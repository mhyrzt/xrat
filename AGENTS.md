# Repository Guidelines

## Project Structure & Module Organization

`src/` contains the Rust application code. Keep responsibilities separated:

- `src/cli/` defines Clap commands, flags, and CLI parsing tests.
- `src/app/` contains app behavior, runtime bootstrap, input loading, and
  command handlers.
- `src/db/` contains SQLite connection code, SQLx migrations, repository
  functions, and DB-facing models.
- `src/parser/` parses and normalizes subscription/config lines into domain
  models.
- `src/model/` contains shared domain types.
- `src/support/` contains small shared helpers such as decoding.
- `migrations/` stores SQL migrations; keep schema changes in ordered files like
  `0001_init.sql`.
- `plan/` holds implementation notes and phased design docs.

## Build, Test, and Development Commands

- `cargo build` — compile the project.
- `cargo test -q` — run the test suite quietly.
- `cargo fmt` — format Rust code.
- `cargo run -- <command>` — run the CLI locally, for example:
  - `cargo run -- import <input>`
  - `cargo run -- list configs`

Run `cargo fmt` and `cargo test -q` before committing.

## Coding Style & Naming Conventions

Use standard Rust formatting via `cargo fmt` (4-space indentation, rustfmt
defaults). Prefer small modules over large files. Keep parsing, CLI definition,
and command execution in separate folders.

Naming:

- files/modules: `snake_case`
- functions: `snake_case`
- structs/enums: `PascalCase`
- CLI flags: long, explicit names such as `--database`, `--selected-only`

Avoid one-letter variable names and avoid adding inline comments unless truly
necessary.

## Testing Guidelines

Use Rust’s built-in test framework with `#[test]` and `#[tokio::test]`. Keep
tests close to the code they validate. Prefer focused unit tests for parser, CLI
parsing, and DB behavior. Name tests descriptively, e.g.
`parses_list_config_filters`.

## Commit & Pull Request Guidelines

Follow conventional commit style seen in history:

- `feat: add default app paths`
- `refactor: split parser into modules`
- `test: add parser regression cases`

Keep commits focused and descriptive. PRs should summarize behavior changes,
mention schema or CLI changes, and include example commands/output when
relevant.

## Architecture Notes

Keep `src/main.rs` thin. New CLI behavior should usually add:

- a new file in `src/cli/`
- a matching handler in `src/app/commands/`

Do not persist full root-level Xray JSON in the database; generate runtime
config when needed.
