# Move Config Lifecycle Mutations Out Of CLI Commands

## Finding

### [Priority: High] Move config lifecycle mutations out of CLI commands

**Files involved:**

- `src/app/commands/lifecycle.rs`
- `src/app/commands/resolve.rs`
- `src/tui/run/tasks/commands.rs`
- `src/db/repository/configs`

**Problem:** The CLI lifecycle command handler contains business rules for
enable, disable, soft delete, hard delete, subscription delete, restore, and
show. It resolves identifiers, validates deleted/enabled state, calls
repositories, decides whether operations are no-ops, and prints user-facing
messages in the same functions.

**Why this change is needed:** These lifecycle operations are core application
behavior, not CLI behavior. If the TUI, daemon, or future management HTTP API
need the same mutations, they must duplicate state checks, no-op rules, and
error wording or call a CLI module that prints to stdout. That weakens
testability and makes mutation semantics inconsistent across adapters.

**How to implement it:** Introduce `ConfigLifecycleService` with methods such as
`enable_config`, `disable_config`, `delete_config`, `delete_subscription`, and
`restore_config`. Use typed request structs containing resolved or raw
identifiers, hard/soft delete flags, and confirmation decisions. Return typed
outcomes such as `Changed`, `AlreadyEnabled`, `DeletedConfig`, or
`SubscriptionDeleted` instead of printing. Keep confirmation prompts and output
rendering in `src/app/commands/lifecycle.rs`. Reuse the service from TUI command
tasks and future Axum mutation handlers.

**Positive effect on the codebase:** Lifecycle state transitions become
consistent and directly unit-testable. New interfaces can perform mutations
without depending on CLI args or stdout.

**Suggested target architecture:** Domain/application lifecycle service owns
state transition rules; repositories perform persistence; adapters perform
confirmation, authorization, and presentation.

**Risk / migration notes:** Medium risk because lifecycle commands affect
persisted state. Add focused service tests for deleted, already-enabled,
already-disabled, hard-delete, and subscription-delete paths before replacing
CLI internals.
