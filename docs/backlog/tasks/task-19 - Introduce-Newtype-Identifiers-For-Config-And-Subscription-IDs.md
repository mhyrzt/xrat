---
id: TASK-19
title: Introduce Newtype Identifiers For Config And Subscription IDs
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-2
dependencies: []
priority: medium
ordinal: 25
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/25-newtype-ids.md`

# Introduce Newtype Identifiers For Config And Subscription IDs

## Finding

### [Priority: Medium] Replace raw ID primitives with newtype wrappers

**Files involved:**

- `src/model/`
- `src/db/record/`
- `src/db/repository/configs/`
- `src/db/repository/subscriptions.rs`
- `src/app/commands/resolve.rs`
- `src/app/commands/lifecycle.rs`
- `src/server/routes/configs.rs`
- `src/tui/data/configs.rs`

**Problem:** Config and subscription identifiers are passed around as raw
primitives (`i64` database row ids, `String` prefix/identifier tokens) through
repositories, command handlers, resolvers, HTTP routes, and TUI rows. Nothing in
the type system distinguishes a config id from a subscription id, or a resolved
numeric id from an unresolved user-supplied prefix string.

**Why this change is needed:** Raw primitive ids invite argument-swap bugs (a
function taking `(config_id: i64, subscription_id: i64)` accepts them in either
order), and they make the repository-trait signatures planned in
`01-config-query-use-cases`, `02-config-lifecycle-service`, and the
`ConfigRepository` port self-documenting only by parameter name. They also blur
the resolve step: `app/commands/resolve.rs` turns a user-supplied prefix into a
concrete id, but both ends are `String`/`i64`, so "resolved" vs "unresolved" is
not visible at call sites.

**How to implement it:** Add small newtypes in `src/model/` such as `ConfigId`,
`SubscriptionId`, and a `ConfigRef` (raw user token before resolution). Derive
the usual traits (`Copy`/`Clone`, `Eq`, `Hash`, `Display`, `serde`,
`sqlx::Type`) so they pass through repositories, DTOs, and TUI rows with no extra
mapping. Resolution (`resolve.rs`) takes a `ConfigRef` and returns a `ConfigId`,
making the transition explicit. Migrate signatures bottom-up: repositories first,
then use-cases, then adapters.

**Positive effect on the codebase:** Eliminates a class of id-mixup bugs at
compile time. Makes the upcoming repository/use-case trait signatures
(`01`, `02`, `06`, `07`) readable without comments. Encodes the resolve step in
the type system, shrinking a source of `unwrap()`s flagged in
`24-audit-production-panics`.

**Suggested target architecture:** Domain ids are newtypes owned in `src/model/`;
repositories, use-cases, DTOs, and view models all speak in newtypes; only the
SQL layer and CLI parse boundary convert to/from primitives.

**Risk / migration notes:** Low risk, mechanical, but touches many signatures. Do
it before or alongside `01`/`06` so the new use-case and read-model signatures
adopt newtypes from the start rather than being migrated twice. Keep `From`/`Into`
conversions at the SQL and CLI edges to localize the churn.
<!-- SECTION:DESCRIPTION:END -->
