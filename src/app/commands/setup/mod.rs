//! `xrat setup` — post-install orchestration. Runs each setup step idempotently
//! (init, dependency checks, daemon, linger, completions, man pages, desktop,
//! PATH) and supports a read-only `--check` diagnostic mode.

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
        return check(context, args);
    }
    apply(context, args).await
}

fn check(context: &AppContext, args: &SetupArgs) -> crate::app::Result<()> {
    let outcomes = vec![
        steps::probe_xray(),
        steps::probe_singbox(),
        steps::probe_init(context),
        steps::probe_daemon(),
        steps::probe_linger(),
        steps::probe_completions(),
        steps::probe_manpages(),
        desktop::probe(),
        steps::probe_path(),
    ];

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

    // Dependencies: xray is required, sing-box is optional.
    report::print_section("Dependencies");
    let xray = emit(steps::probe_xray());
    let xray_missing = xray.status == StepStatus::Missing;
    outcomes.push(xray);
    outcomes.push(emit(steps::probe_singbox()));

    if xray_missing {
        return Err(crate::app::AppError::InvalidArgument(
            "xray-core is required but was not found on PATH; install xray-core and re-run `xrat setup`"
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

    outcomes.push(emit(steps::probe_path()));

    println!();
    println!("{}", output::success("Setup complete.", color));

    record_event(context, &outcomes).await;
    Ok(())
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
