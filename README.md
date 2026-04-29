# XRAT

XRAT is a Rust CLI for importing, storing, testing, and running Xray-compatible proxy configurations.

## Current Features

- Import subscription/config lines into SQLite or PostgreSQL.
- List stored configs and subscriptions.
- Test configs for connectivity, latency, and optional download speed.
- Run one stored config as a managed local Xray runtime.
- Show runtime status and stop the active runtime session.

## Development

Common commands:

```bash
cargo build
cargo test -q
cargo fmt
```

Run the CLI locally:

```bash
cargo run -- import <input>
cargo run -- list configs
cargo run -- test
cargo run -- connect <id>
cargo run -- status
cargo run -- disconnect
```

## Documentation

Planning notes live in `docs/plan/`.
