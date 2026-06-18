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
    print_detection(color);

    let mut outcomes = Vec::new();

    // Dependencies: xray is required, sing-box is optional.
    let xray = steps::probe_xray();
    print_inline(&xray);
    let xray_missing = xray.status == StepStatus::Missing;
    outcomes.push(xray);
    let singbox = steps::probe_singbox();
    print_inline(&singbox);
    outcomes.push(singbox);

    if xray_missing {
        return Err(crate::app::AppError::InvalidArgument(format!(
            "xray-core is required and was not found on PATH; {}",
            steps::probe_xray()
                .detail
                .unwrap_or_else(|| "install xray-core".to_string())
        )));
    }

    outcomes.push(run_step("init", || steps::apply_init(context)));

    if args.no_daemon {
        outcomes.push(skipped(steps::STEP_DAEMON, "--no-daemon"));
    } else if want_daemon(args) {
        outcomes.push(run_step(steps::STEP_DAEMON, || {
            steps::apply_daemon(context)
        }));
        if args.linger {
            outcomes.push(run_step(steps::STEP_LINGER, steps::apply_linger));
        }
    } else {
        outcomes.push(skipped(steps::STEP_DAEMON, "declined"));
    }

    if args.no_completions {
        outcomes.push(skipped(steps::STEP_COMPLETIONS, "--no-completions"));
    } else {
        outcomes.push(run_step(steps::STEP_COMPLETIONS, steps::apply_completions));
    }

    if args.no_manpages {
        outcomes.push(skipped(steps::STEP_MANPAGES, "--no-manpages"));
    } else {
        outcomes.push(run_step(steps::STEP_MANPAGES, steps::apply_manpages));
    }

    if args.no_desktop {
        outcomes.push(skipped(steps::STEP_DESKTOP, "--no-desktop"));
    } else {
        outcomes.push(run_step(steps::STEP_DESKTOP, desktop::apply));
    }

    let path = steps::probe_path();
    print_inline(&path);
    outcomes.push(path);

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

fn run_step(_name: &str, step: impl FnOnce() -> StepOutcome) -> StepOutcome {
    let outcome = step();
    steps::print_step(outcome.name);
    steps::print_step_result(&outcome);
    outcome
}

fn skipped(name: &'static str, reason: &str) -> StepOutcome {
    let outcome =
        StepOutcome::new(name, StepStatus::Skipped, false).with_detail(reason.to_string());
    steps::print_step(name);
    steps::print_step_result(&outcome);
    outcome
}

fn print_inline(outcome: &StepOutcome) {
    steps::print_step(outcome.name);
    steps::print_step_result(outcome);
}

fn print_detection(color: bool) {
    println!(
        "{}",
        output::format_kv(
            Some("Detected"),
            &[
                ("os", platform::os().to_string()),
                ("arch", platform::arch().to_string()),
                ("shell", platform::detect_shell().name().to_string()),
            ],
            color,
        )
    );
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
