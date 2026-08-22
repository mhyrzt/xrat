//! `xrat setup` — post-install orchestration. Runs each setup step idempotently
//! (init, dependency checks, daemon, linger, completions, man pages, desktop,
//! xratui shortcut, PATH) and supports a read-only `--check` diagnostic mode.

mod cores;
mod desktop;
mod report;
mod steps;

use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::events;
use crate::cli::{SetupArgs, SetupFormat};
use crate::support::platform;

use report::{StepOutcome, StepStatus};

pub async fn run(context: &AppContext, args: &SetupArgs) -> crate::app::Result<()> {
    if args.check {
        return check(context, args).await;
    }
    apply(context, args).await
}

async fn check(context: &AppContext, args: &SetupArgs) -> crate::app::Result<()> {
    let mut outcomes = cores::probe_all(context)
        .await
        .iter()
        .map(dependency_check_outcome)
        .collect::<Vec<_>>();
    outcomes.extend([
        steps::probe_init(context),
        steps::probe_daemon(),
        steps::probe_linger(),
        steps::probe_completions(),
        steps::probe_manpages(),
        desktop::probe(),
        steps::probe_tui_shim(),
        steps::probe_path(),
    ]);

    match args.format {
        SetupFormat::Json => println!("{}", report::render_json(&outcomes)?),
        SetupFormat::Table => println!("{}", report::render_table(&outcomes)),
    }

    if report::has_blocking_failure(&outcomes) {
        return Err(crate::app::AppError::InvalidArgument(
            "setup incomplete: required steps are missing".to_string(),
        ));
    }
    Ok(())
}

async fn apply(context: &AppContext, args: &SetupArgs) -> crate::app::Result<()> {
    let color = output::color_enabled();
    print_detection();

    let mut outcomes = Vec::new();

    report::print_section("Dependencies");
    for probe in cores::probe_all(context).await {
        outcomes.push(emit(apply_dependency(context, args, probe).await));
    }

    if report::has_blocking_failure(&outcomes) {
        return Err(crate::app::AppError::InvalidArgument(
            "xray-core is required; install it through the setup prompt or re-run `xrat setup`"
                .to_string(),
        ));
    }

    println!();
    report::print_section("Setup");

    outcomes.push(emit(steps::apply_init(context)));

    if args.no_daemon {
        outcomes.push(emit(skipped(steps::STEP_DAEMON, "--no-daemon")));
    } else if want_daemon(args) {
        outcomes.push(emit(steps::apply_daemon(context)));
        if want_linger(args) {
            outcomes.push(emit(steps::apply_linger()));
        }
    } else {
        outcomes.push(emit(skipped(steps::STEP_DAEMON, "declined")));
    }

    if args.no_completions {
        outcomes.push(emit(skipped(steps::STEP_COMPLETIONS, "--no-completions")));
    } else {
        outcomes.push(emit(steps::apply_completions()));
    }

    if args.no_manpages {
        outcomes.push(emit(skipped(steps::STEP_MANPAGES, "--no-manpages")));
    } else {
        outcomes.push(emit(steps::apply_manpages()));
    }

    if args.no_desktop {
        outcomes.push(emit(skipped(steps::STEP_DESKTOP, "--no-desktop")));
    } else {
        outcomes.push(emit(desktop::apply()));
    }

    outcomes.push(emit(steps::apply_tui_shim()));

    outcomes.push(emit(steps::probe_path()));

    println!();
    println!("{}", output::success("Setup complete.", color));

    record_event(context, &outcomes).await;
    Ok(())
}

fn dependency_check_outcome(probe: &cores::CoreProbe) -> StepOutcome {
    let status = if probe.missing() {
        StepStatus::Missing
    } else if probe.outdated() {
        StepStatus::UpdateAvailable
    } else {
        StepStatus::Done
    };
    StepOutcome::new(probe.kind.name(), status, probe.kind.required()).with_detail(probe.detail())
}

async fn apply_dependency(
    context: &AppContext,
    args: &SetupArgs,
    probe: cores::CoreProbe,
) -> StepOutcome {
    let required = probe.kind.required();
    let status_without_change = if probe.missing() {
        StepStatus::Missing
    } else if probe.outdated() {
        StepStatus::UpdateAvailable
    } else {
        StepStatus::AlreadyDone
    };
    let Some(release) = probe.latest.as_ref().ok() else {
        return StepOutcome::new(probe.kind.name(), status_without_change, required)
            .with_detail(probe.detail());
    };
    if !probe.missing() && !probe.outdated() {
        return StepOutcome::new(probe.kind.name(), status_without_change, required)
            .with_detail(probe.detail());
    }

    let should_change = if args.yes {
        unattended_dependency_change(probe.kind, probe.missing())
    } else {
        let prompt = dependency_prompt(&probe, release);
        output::confirm_default(prompt, !probe.missing() || probe.kind.unattended_default())
            .unwrap_or(false)
    };
    if !should_change {
        return StepOutcome::new(probe.kind.name(), status_without_change, required)
            .with_detail(probe.detail());
    }

    let progress_enabled = args.format == SetupFormat::Table;
    match cores::install(context, probe.kind, release, progress_enabled).await {
        Ok(installed) => {
            let mut detail = format!(
                "{} (v{}; managed)",
                installed.binary_path.display(),
                installed.version
            );
            if let Some(warning) = installed.cli_link_warning {
                detail.push_str(&format!("; {warning}"));
            }
            StepOutcome::new(probe.kind.name(), StepStatus::Done, required).with_detail(detail)
        }
        Err(error) => {
            StepOutcome::new(probe.kind.name(), StepStatus::Failed, required).with_detail(error)
        }
    }
}

fn unattended_dependency_change(kind: cores::CoreKind, missing: bool) -> bool {
    !missing || kind.unattended_default()
}

fn dependency_prompt(probe: &cores::CoreProbe, release: &cores::CoreRelease) -> String {
    if probe.missing() {
        return format!(
            "Install {} v{} as a user-local managed core?",
            probe.kind.name(),
            release.version
        );
    }
    let current = probe
        .version
        .as_ref()
        .map(|version| format!("v{version}"))
        .unwrap_or_else(|| "the installed version".to_string());
    if probe.managed {
        format!(
            "Upgrade {} {current} to v{}?",
            probe.kind.name(),
            release.version
        )
    } else {
        format!(
            "Keep the external {} untouched and adopt managed v{} instead of {current}?",
            probe.kind.name(),
            release.version
        )
    }
}

fn want_daemon(args: &SetupArgs) -> bool {
    if args.yes || args.linger {
        return true;
    }
    output::confirm("Install and start the background daemon?").unwrap_or(false)
}

/// Whether to enable boot-before-login start. `--linger` forces it; otherwise
/// (Linux, interactive only) prompt, defaulting to no.
fn want_linger(args: &SetupArgs) -> bool {
    if args.linger {
        return true;
    }
    if args.yes || platform::os() != "linux" {
        return false;
    }
    output::confirm("Enable boot-before-login start (systemd lingering)?").unwrap_or(false)
}

fn emit(outcome: StepOutcome) -> StepOutcome {
    report::print_outcome(&outcome);
    outcome
}

fn skipped(name: &'static str, reason: &str) -> StepOutcome {
    StepOutcome::new(name, StepStatus::Skipped, false).with_detail(reason.to_string())
}

fn print_detection() {
    let color = output::color_enabled();
    report::print_section("Environment");

    let mut rows = vec![
        ("os", platform::os_pretty()),
        ("arch", platform::arch().to_string()),
        ("shell", platform::detect_shell().name().to_string()),
    ];
    if let Some(terminal) = desktop::detected_terminal() {
        rows.push(("terminal", terminal));
    }

    for (key, value) in rows {
        println!(
            "  {:<width$}  {}",
            key,
            output::style_text(&value, output::Style::Dim, color),
            width = report::NAME_WIDTH,
        );
    }
    println!();
}

async fn record_event(context: &AppContext, outcomes: &[StepOutcome]) {
    let detail = serde_json::to_string(outcomes).ok();
    events::record(
        &context.db,
        events::LEVEL_INFO,
        events::SOURCE_SETUP,
        "setup_completed",
        "Setup completed",
        None,
        None,
        detail,
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unattended_setup_installs_recommended_missing_cores() {
        assert!(unattended_dependency_change(cores::CoreKind::Xray, true));
        assert!(unattended_dependency_change(cores::CoreKind::SingBox, true));
        assert!(!unattended_dependency_change(cores::CoreKind::V2Ray, true));
    }

    #[test]
    fn unattended_setup_updates_any_installed_core() {
        for kind in cores::CORE_KINDS {
            assert!(unattended_dependency_change(kind, false));
        }
    }
}
