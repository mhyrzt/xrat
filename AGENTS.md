# Repository Guidelines

## Project Structure & Module Organization

`src/` contains the Rust application code. Keep responsibilities separated:

- `src/cli/` defines Clap command trees, flags, and CLI parsing tests under
  `src/cli/tests/`.
- `src/app/` contains app runtime bootstrap and command handlers. Keep command
  logic under `src/app/commands/` and keep lifecycle concerns in
  `src/app/daemon/`, `src/app/runtime_service/`, and `src/xray/process_mgmt/`.
- `src/app/context/`, `src/app/config/`, and `src/app/paths/` contain
  application bootstrap, path resolution, and command-specific runtime settings.
- `src/config/` contains import, parsing, normalization, and protocol link
  support. Keep external input parsing separate from runtime config generation.
- `src/xray/` contains Xray parsing, generated runtime config builders, and
  process management.
- `src/singbox/` contains sing-box parsing/translation support.
- `src/prober/` contains connection test runners for TCP, ICMP, download,
  upload, and real-delay flows.
- `src/db/` contains database wiring, schema helpers, records, and repositories,
  split across `src/db/database/`, `src/db/record/`, and `src/db/repository/`.
- `src/model/` contains shared domain types.
- `src/server/` contains HTTP API routes, auth, response, and server state.
- `src/tui/` contains terminal UI state, views, keymaps, and data adapters.
- `src/support/` contains small shared helpers for decode, GeoIP, cancellation,
  network, time, and URL handling.
- `migrations/sqlite/` and `migrations/postgres/` hold ordered SQL migrations.
- `docs/src/` holds user-facing documentation.
- `packaging/systemd/` holds user-service templates used by
  `xrat daemon install`; `packaging/desktop/` holds desktop entry and icon
  packaging assets.
- `install.sh` installs release archives from GitHub and runs optional first-run
  setup.
- `.github/workflows/` contains CI and release automation, including musl
  release builds, generated man pages/completions, GitHub releases, and
  crates.io publishing.
- `testdata/` holds local fixtures such as GeoIP data.

## Build, Test, and Development Commands

- `cargo build` — compile the project.
- `cargo fmt` — format Rust code.
- `cargo clippy --all-targets -- -D warnings` — run the same lint gate used by
  CI.
- `cargo test -q --locked` — run the test suite quietly with the locked
  dependency graph.
- `cargo run -- <command>` — run the CLI locally, for example:
  - `cargo run -- init`
  - `cargo run -- import <input>`
  - `cargo run -- list`
  - `cargo run -- parse <config-id>`
  - `cargo run -- test <config-id>`
  - `cargo run -- scan`
  - `cargo run -- connect <config-id>`
  - `cargo run -- disconnect`
  - `cargo run -- serve`
  - `cargo run -- tui`
  - `cargo run -- status`
  - `cargo run -- daemon install --start`
  - `cargo run -- geoip status`
  - `cargo run -- manpage --output <dir>`
  - `cargo run -- completions <shell>`

Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test -q --locked` before committing.

## Coding Style & Naming Conventions

Use standard Rust formatting via `cargo fmt` (4-space indentation, rustfmt
defaults). Prefer small modules over large files, and split by capability when
files begin mixing CLI parsing, command orchestration, and domain logic.

Naming:

- files/modules: `snake_case`
- functions: `snake_case`
- structs/enums: `PascalCase`
- CLI flags: long, explicit names such as `--database`, `--selected-only`, and
  `--include-geoip`

Avoid one-letter variable names. Avoid inline comments unless they explain
non-obvious intent or constraints.

## Testing Guidelines

Use Rust’s built-in test framework with `#[test]` and `#[tokio::test]`.

- Keep tests close to the code they validate.
- Prefer focused unit tests for parser/config normalization, CLI parsing, DB
  repositories, and runtime lifecycle transitions.
- Add regression tests when fixing parsing, dedup, scanner, or runtime-session
  edge cases.
- For CLI changes, add or update parser tests in `src/cli/tests/` when behavior
  can be validated without starting external services.
- For repository changes, prefer database tests that exercise both SQLite and
  Postgres paths when the existing helpers make that practical.
- Name tests descriptively, e.g. `parses_list_config_filters`.

## Commit & Pull Request Guidelines

Follow conventional commit style seen in history:

- `feat: add scanner command with cf_scan persistence`
- `feat: complete managed runtime service`
- `refactor: split modules into subfiles`
- `test: expand runtime lifecycle coverage`

Keep commits focused and descriptive. PRs should summarize behavior changes,
mention schema or CLI changes, and include example commands/output when
relevant.

## Architecture Notes

Keep `src/main.rs` thin and route behavior through `src/cli` and `src/app`.

For new CLI behavior, usually add:

- a new or extended command file in `src/cli/`
- a matching handler under `src/app/commands/`
- repository/model updates in `src/db/` when persistence is required
- documentation under `docs/src/02-cli/` or the relevant feature page when user
  behavior changes
- packaging or installer updates when behavior changes generated service
  templates, desktop assets, release extras, or first-run installation flow

Design constraints from current implementation direction:

- Do not persist full root-level Xray JSON in the database; generate runtime
  config on demand from stored normalized data.
- Keep managed runtime lifecycle behavior explicit and observable (start/status/
  stop flow, persisted runtime session state).
- Keep parser and runtime-generation concerns decoupled so parse/test/scan flows
  can evolve without rewriting CLI glue.
- Keep CLI parsing, command orchestration, persistence, and external process
  control in separate layers. Avoid command handlers that directly parse raw
  subscription formats or manually assemble database rows when reusable services
  already exist.
- Prefer typed domain records and repository methods over passing raw JSON or
  loosely structured maps between layers.

## Agent Behavioral Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Merge with
project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial
tasks, use judgment.

### Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them instead of picking silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop, name what is confusing, and ask.

### Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- Do not add features beyond what was asked.
- Do not add abstractions for single-use code.
- Do not add flexibility or configurability that was not requested.
- Do not add error handling for impossible scenarios.
- If a change grows far beyond the requested scope, pause and simplify.

Ask: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Do not improve adjacent code, comments, or formatting unless required.
- Do not refactor things that are not broken.
- Match existing style, even if you would do it differently.
- If you notice unrelated dead code, mention it instead of deleting it.

When changes create orphans:

- Remove imports, variables, functions, and tests that your changes made unused.
- Do not remove pre-existing dead code unless asked.

Every changed line should trace directly to the user's request.

### Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" means write tests for invalid inputs, then make them pass.
- "Fix the bug" means write or identify a reproduction, then make it pass.
- "Refactor X" means preserve behavior and run relevant tests before finishing.

For multi-step tasks, state a brief plan:

```text
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria such as "make
it work" require clarification.

These guidelines are working if diffs contain fewer unnecessary changes, fewer
rewrites are needed due to overcomplication, and clarifying questions come
before implementation rather than after mistakes.
