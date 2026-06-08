# Split Test Execution Core From CLI Command Modules

## Finding

### [Priority: High] Split test execution core from CLI command modules

**Files involved:**

- `src/tui/run/tasks/test_batch.rs`
- `src/app/commands/test.rs`
- `src/app/commands/test/bulk`
- `src/app/commands/test/settings`
- `src/cli/test_cmd`

**Problem:** The TUI batch-test task calls
`crate::app::commands::test::run_bulk_for_config_ids_with_progress` and builds a
`crate::cli::TestArgs` value from TUI state. This makes the TUI depend on a CLI
argument type and a CLI command module for a core testing workflow.

**Why this change is needed:** The proxy test pipeline is an application
use-case shared by CLI and TUI. When the TUI constructs CLI args, CLI defaults
and formatting concerns leak into TUI behavior. This coupling makes it harder to
add HTTP or daemon-triggered tests, and it obscures which settings are domain
defaults versus command-line presentation defaults.

**How to implement it:** Move bulk test execution, settings resolution, progress
updates, and result summary generation into `src/app/use_cases/test.rs` or
`src/prober/service.rs`. Define `TestRunRequest`, `TestStagePolicy`,
`TestRunProgress`, and `TestRunSummary` in application code. Convert
`cli::TestArgs` to `TestRunRequest` in the CLI command handler. Convert `TuiApp`
state to `TestRunRequest` in `src/tui/run/tasks/test_batch.rs`. Keep CLI output
sorting, formats, and progress bars in `src/app/commands/test/output`.

**Positive effect on the codebase:** Test behavior can be validated once at the
use-case level, while CLI and TUI tests only need to verify translation and
rendering. New adapters can trigger tests without constructing fake CLI args.

**Suggested target architecture:** Prober modules execute low-level checks; a
test use-case coordinates selection, concurrency, cancellation, persistence, and
progress; adapters render progress and summaries.

**Risk / migration notes:** High value but medium risk because test execution
has many options. Migrate in small steps by first adding a request type that
mirrors current `TestArgs`, then move settings resolution and bulk execution
behind it.
