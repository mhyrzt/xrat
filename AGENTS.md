# Repository Guidelines

## Project Structure & Module Organization

`src/` contains the Rust application code. Keep responsibilities separated:

- `src/cli/` defines Clap command trees, flags, and CLI parsing tests under
  `src/cli/tests/`.
- `src/app/` contains app runtime bootstrap and command handlers. Keep command
  logic under `src/app/commands/` (for example `parse/` and `test/`) and keep
  runtime lifecycle concerns in `src/app/runtime/` and
  `src/app/runtime_service/`.
- `src/db/` contains database wiring and repositories, split across
  `src/db/database/`, `src/db/model/`, and `src/db/repository/`.
- `src/config/` and `src/config/xray/` contain runtime config generation
  builders and protocol/transport mapping.
- `src/model/` contains shared domain types.
- `src/singbox/` contains sing-box parsing/translation support.
- `src/tester/` contains connection test runners (for example download and real
  delay flows).
- `src/support/` contains small shared helpers.
- `migrations/sqlite/` and `migrations/postgres/` hold ordered SQL migrations.
- `docs/plan/` and `docs/validation/` hold phased plans, parity notes, and
  implementation checklists.

## Build, Test, and Development Commands

- `cargo build` — compile the project.
- `cargo test -q` — run the test suite quietly.
- `cargo fmt` — format Rust code.
- `cargo run -- <command>` — run the CLI locally, for example:
  - `cargo run -- import <input>`
  - `cargo run -- parse <config_id>`
  - `cargo run -- test <config_id>`
  - `cargo run -- scan`
  - `cargo run -- runtime status`

Run `cargo fmt` and `cargo test -q` before committing.

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

Design constraints from current implementation direction:

- Do not persist full root-level Xray JSON in the database; generate runtime
  config on demand from stored normalized data.
- Keep managed runtime lifecycle behavior explicit and observable (start/status/
  stop flow, persisted runtime session state).
- Keep parser and runtime-generation concerns decoupled so parse/test/scan flows
  can evolve without rewriting CLI glue.
