---
id: TASK-18
title: Audit Production unwrap() And expect() Panics
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
ordinal: 24
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/1-foundation/24-audit-production-panics.md`

# Audit Production unwrap() And expect() Panics

## Finding

### [Priority: Medium] Audit and reduce production unwrap()/expect() panics

**Files involved:**

- `src/app/` (78 occurrences)
- `src/db/` (18 occurrences)
- `src/config/` (17 occurrences)
- `src/server/` (6 occurrences)
- `src/support/` (4 occurrences)

**Problem:** There are ~123 `unwrap()`/`expect()` calls in non-test production
code. Each is a panic path that no test exercises and that aborts the process (or
a daemon/TUI/server task) at runtime when the assumed invariant does not hold.
The heaviest concentration is in `src/app/` (78), which spans command handlers,
runtime services, and daemon supervision — exactly the long-running paths where a
panic is most disruptive (daemon abort, TUI freeze, dropped IPC connection).

**Why this change is needed:** Panics in adapter and use-case code turn
recoverable conditions (missing field, parse failure, lock poisoning, absent
optional value) into crashes instead of surfaced `AppError`s. They also hide
error-handling behavior from tests: a `Result` path can be asserted, an `unwrap`
cannot. Reducing them improves crash safety and makes failure handling testable —
which directly supports the port and use-case work that wants to assert failure
scenarios.

**How to implement it:** Triage every occurrence into one of three buckets:

- **Recoverable** — convert to `?` with an appropriate layered error variant
  (pairs with `23-split-apperror-by-layer`). Most `app/` and `config/` cases.
- **Provable invariant** — keep, but use `expect("why this cannot fail")` with a
  message documenting the invariant (e.g. a regex compiled from a literal, a map
  key just inserted).
- **Test-only assumption leaking into production** — restructure so the invariant
  is encoded in the type (ties into `25-newtype-ids` for ID parsing).

Add a clippy gate incrementally (`-W clippy::unwrap_used` /
`clippy::expect_used`) scoped to already-cleaned modules so new panics do not
regress. Do not flip the gate repo-wide until the audit is done.

**Positive effect on the codebase:** Fewer crash paths in the daemon, server, and
TUI. Error handling becomes explicit and testable. The remaining `expect`s carry
documented invariants instead of silent assumptions.

**Suggested target architecture:** Adapters and use-cases return layered errors;
panics remain only for genuinely unreachable invariants and are documented with
an `expect` message; a scoped clippy lint prevents regressions.

**Risk / migration notes:** Low risk per change but high volume. Do it
module-by-module, not in one sweep, and pair the `app/` cleanup with the relevant
use-case extraction (`01`–`05`) so converted errors flow into the right layered
type. Add or extend tests for each newly-introduced error path.
<!-- SECTION:DESCRIPTION:END -->
