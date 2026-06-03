# Contributing

## Prerequisites

- Rust stable toolchain (`rustup toolchain install stable`)
- [`just`](https://github.com/casey/just) — task runner
- Docker — for PostgreSQL integration tests
- [`mdbook`](https://rust-lang.github.io/mdBook/) — for local docs preview
  (optional)

## Local Setup

```sh
git clone <repo>
cd xrat
cargo build
```

Run the full CI pipeline locally before pushing:

```sh
just ci       # cargo fmt --check + clippy + locked tests
```

Or individually:

```sh
just fmt      # format Rust, Markdown, SQL
just lint     # cargo clippy --all-targets -- -D warnings
just test     # cargo test -q --locked
```

## PostgreSQL Tests

Some repository tests can verify against a real Postgres backend:

```sh
just postgres-up
just test-postgres
just postgres-down
```

## Commit Style

Follow conventional commits as seen in history:

```
feat: add scanner command with cf_scan persistence
fix: resolve clippy warnings in runtime session state
refactor(tui): split oversized tasks module by capability
test: expand runtime lifecycle coverage
docs: add CLI reference for geoip command
chore: remove obsolete shell script
ci: reuse ci.yml in release via workflow_call
```

- Subject ≤ 72 characters, imperative mood
- Use scope in parentheses when the change is module-specific: `feat(tui):`,
  `refactor(db):`
- No period at end of subject
- Body only when the _why_ is non-obvious from the diff

## Pull Requests

- One concern per PR. Mix of unrelated changes gets asked to split.
- Mention schema or CLI changes explicitly in the description.
- Include example commands/output when user-facing behavior changes.
- CI must be green: `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test -q --locked`.

## Coding Style

- 4-space indentation, `rustfmt` defaults — enforced by `cargo fmt`
- `snake_case` for files, modules, functions; `PascalCase` for structs/enums
- Long `--explicit-flag-names` for CLI flags
- No one-letter variable names
- No comments unless the _why_ is non-obvious to a future reader
- Prefer small, focused modules over large mixed files

## Adding New CLI Behavior

For a new command or flag, usually touch:

1. `src/cli/` — Clap command definition and parser tests under `src/cli/tests/`
2. `src/app/commands/` — command handler
3. `src/db/` — repository/model updates if persistence is required
4. `docs/src/02-cli/` — user-facing documentation

## Testing

- Keep tests adjacent to the code they validate
- Unit tests for parsers, config normalization, CLI parsing, DB repos, runtime
  lifecycle
- Add regression tests when fixing parsing, dedup, scanner, or session edge
  cases
- CLI tests in `src/cli/tests/` for behavior that doesn't require external
  services
- DB tests should exercise SQLite and Postgres paths where helpers exist
- Name tests descriptively: `parses_list_config_filters`,
  `rejects_duplicate_import`

## Architecture Constraints

- Do not persist full root-level Xray JSON; generate runtime config on demand
- Keep CLI parsing, command orchestration, persistence, and process control in
  separate layers
- Keep parser and runtime-generation concerns decoupled
- Prefer typed domain records over raw JSON or loosely structured maps between
  layers
- Keep managed runtime lifecycle explicit and observable

## Docs

```sh
just docs     # serves docs at http://localhost:3000
```

Docs live in `docs/src/`.
