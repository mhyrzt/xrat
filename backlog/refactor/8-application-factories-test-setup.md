# Centralize Application Factories And Test Setup

## Finding

### [Priority: Medium] Centralize application factories and test setup

**Files involved:**

- `src/main.rs`
- `src/app/context.rs`
- `src/server/mod.rs`
- `src/server/tests/mod.rs`
- `src/app/commands/connect.rs`
- `src/app/commands/disconnect.rs`
- `src/app/commands/status/mod.rs`

**Problem:** There is a basic `AppContext::build` and `build_router`, but test
setup is duplicated across command and server tests. Multiple tests manually
create temp roots, database configs, runtime paths, and `AppContext` values.
Command tests for connect, disconnect, and status repeat nearly identical
`test_context` helpers.

**Why this change is needed:** Duplicated setup makes tests noisy and
inconsistent. It also discourages adding tests for new use-cases because
creating a valid app context requires copying boilerplate.

**How to implement it:** Add a shared test support module with builders such as
`TestAppBuilder`, `TestContext`, `TestDatabase`, and `TestRouter`. Provide
defaults for temp paths, SQLite database setup, seeded config nodes, runtime
paths, and server state. For production, add explicit factories such as
`build_app_context`, `build_router_from_context`, `build_daemon_runner`, and
`build_cli_runner` so composition is centralized.

**Positive effect on the codebase:** Tests become shorter, setup behavior stays
consistent, and new architecture services can be validated with less
boilerplate.

**Suggested target architecture:** Production factories wire concrete
dependencies; test factories wire temporary or fake dependencies; individual
tests focus on behavior.

**Risk / migration notes:** Low risk. Start by deduplicating command test
context setup, then server/router fixtures. Avoid changing production behavior
during the first pass.
