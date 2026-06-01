# Phase 2.5 CLI Restructure

## Goal

Move XRAT from the current single-purpose CLI shape into a subcommand-based CLI
that follows:

- `xrat COMMAND [ARGS] [FLAGS]`

This phase is about CLI structure and command ergonomics, not full runtime
orchestration yet.

The purpose is to make the app easier to grow while keeping the existing
persistence work reusable.

## Why This Phase Exists

The current CLI is centered around one main flow:

- `xrat <input>`

That was acceptable while the app only imported subscription text into SQLite,
but it does not scale well now that the project already has:

- stored configs
- config lifecycle flags
- connection test history
- runtime session persistence
- app-level paths and config files

Once the app has multiple user actions such as import, list, select, enable,
disable, connect, and disconnect, the CLI should be command-first rather than
input-first.

## Scope Boundary

This document is only for the CLI foundation work needed right now.

Phase 2.5 should cover:

- moving the CLI from input-first to command-first
- keeping shared/global flags such as `--database` and `--config`
- exposing already-implemented persistence flows through clean commands
- adding basic read/list ergonomics on top of the current database layer
- separating CLI definition from command execution so bootstrap code stays small

Phase 2.5 should not fully implement commands whose real behavior belongs to
later phases.

Those commands may be named here for direction, but their actual implementation
should live with the phase that introduces the underlying capability.

## Target CLI Shape

The desired shape is:

- `xrat COMMAND [ARGS] [FLAGS]`

Examples of the long-term command surface:

- `xrat add <config-uri>`
- `xrat import <input>`
- `xrat list`
- `xrat show <id>`
- `xrat select <id>`
- `xrat enable <id>`
- `xrat disable <id>`
- `xrat delete <id>`
- `xrat restore <id>`
- `xrat status`
- `xrat test <id>`
- `xrat connect <id>`
- `xrat disconnect`

This structure makes XRAT behave more like a normal CLI tool and gives each
action a clear home.

However, not all of these commands belong to Phase 2.5.

## CLI Design Principles

### 1. Command-first structure

The first positional unit after `xrat` should be a verb-like subcommand.

This is better than the current format because it:

- makes intent obvious at a glance
- avoids overloading one positional argument with too much meaning
- keeps help output readable
- allows future commands without breaking the overall CLI model

### 2. Shared global flags

Some flags apply to the whole app rather than one specific command.

Examples:

- `--database`
- `--config`

These should be global/shared flags available across commands.

Purpose:

- `--database`: override the SQLite database path
- `--config`: override the config file path

These flags should integrate with the existing app path behavior:

- use `XRAT_PATH` if present
- otherwise use `$HOME/.config/xrat`
- allow the user to override the specific file paths when needed

### 3. Command-specific flags and args

Each command should own its own inputs and options.

Examples:

- `import` should accept the subscription input source
- `list` may later support filters such as deleted, enabled-only, active-only,
  or selected-only
- `test` may later support timeout or mode flags
- `connect` may later support runtime overrides such as ports or profile options

This prevents the top-level CLI from becoming crowded and keeps related options
close to the action they affect.

## Recommended Clap Structure

The CLI should be modeled roughly as:

- a top-level `Cli` struct
- global/shared flags on `Cli`
- a `Command` enum for subcommands
- dedicated argument structs per command where useful

Conceptually:

```rust
#[derive(Parser)]
pub struct Cli {
    #[arg(long, global = true)]
    pub database: Option<PathBuf>,

    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Add(AddArgs),
    Import(ImportArgs),
    List(ListArgs),
    Show(IdArgs),
    Select(IdArgs),
    Enable(IdArgs),
    Disable(IdArgs),
    Delete(IdArgs),
    Restore(IdArgs),
    Status,
    Test(IdArgs),
    Connect(IdArgs),
    Disconnect,
}
```

This does not need to be implemented all at once, but the shape should support
that direction from the start.

## What Belongs In Phase 2.5

Phase 2.5 should focus only on commands backed by persistence work that already
exists:

- `add`
- `import`
- `list`

These commands are enough to establish the CLI shape and prove that:

- global flags work correctly
- subcommand routing works correctly
- persisted data can be written and read through the CLI
- command execution can live outside `main.rs`

## Structural Outcome

Phase 2.5 should leave the codebase with a clear split between command
definition and command execution:

- `src/cli/`
  - Clap structs, enums, and parsing tests
- `src/app/commands/`
  - command handlers for implemented commands
- `src/app/runtime.rs`
  - runtime/bootstrap context shared by command handlers
- `src/main.rs`
  - thin entrypoint only

This keeps later phases from turning either `src/cli.rs` or `src/main.rs` into
oversized files.

## Commands Deferred To Their Respective Phases

The commands below should be implemented later, together with the capability
they depend on:

- `show`
  - may be added soon as a small persistence-oriented command, but it is not
    required for the CLI restructure itself
- `select`
- `enable`
- `disable`
- `delete`
- `restore`
  - these are config lifecycle commands and can be grouped with the phase where
    config management UX is formalized
- `status`
  - should land once runtime/session state is part of normal user flow
- `test`
  - belongs with the connection-testing phase
- `connect`
- `disconnect`
  - belong with the Xray runtime phase

## Suggested Initial Command Set

The first command set should focus on already-implemented persistence behavior.

### `add`

Purpose:

- add one single config directly into storage without treating it as a
  subscription batch import

This is useful for:

- manually pasting one config URI
- adding one server quickly from the terminal
- distinguishing one-off additions from subscription imports

Examples:

- `xrat add 'vless://...'`
- `xrat add 'ss://...'`
- `xrat add 'trojan://...'`

Expected behavior:

- accept one config URI or one single config line
- parse it using the same normalization/parser pipeline already used elsewhere
- persist it into `configs`
- either create a dedicated non-subscription source record or store it through a
  manual/raw-text source classification

Possible future flags:

- `--name`
- `--select`
- `--enable`

### `import`

Purpose:

- read subscription input from URL, file, or raw text
- parse it
- persist normalized configs into SQLite

This is the current app behavior and should become:

- `xrat import <input>`

Possible future flags:

- `--name`
- `--replace`
- `--source-kind`

### `list`

Purpose:

- read back persisted configs
- show them in a user-friendly list

Recommended initial CLI shape:

- `xrat list configs`
- `xrat list subscriptions`
- allow `subs` as a short alias for subscriptions
- allow `nodes` as a short alias for configs
- treat "configs" here as stored nodes/profiles

This should likely use the current config query methods already present in the
database layer.

Possible future flags:

- `--all`
- `--deleted`
- `--enabled-only`
- `--active-only`
- `--selected-only`
- `--subscription <id>`

Initial implementation now supports:

- `xrat list configs --all`
- `xrat list configs --deleted`
- `xrat list configs --enabled-only`
- `xrat list configs --active-only`
- `xrat list configs --selected-only`
- `xrat list configs --subscription <id>`
- `xrat list subscriptions --kind <url|file|raw-text>`
- `xrat list subs`
- `xrat list nodes`

## Deferred Command Notes

### `show`

Useful for inspecting one persisted row in detail, but it is not necessary to
complete the command-structure refactor.

### `select`

Useful once we formalize config-management UX, especially around chosen/default
nodes.

### `enable` / `disable`

These should arrive with the broader config lifecycle management slice rather
than being treated as CLI-structure work.

### `delete` / `restore`

Purpose:

- soft-delete a config
- restore a previously deleted config

Input:

- config id

These should not physically remove rows.

### `status`

Purpose:

- show current app state at a glance

Potential contents:

- selected config
- active config
- latest runtime session
- maybe latest test result later

This command is especially useful once connect/disconnect behavior is added.

## Commands Better Added Slightly Later

These fit the new CLI structure well, but they depend on runtime/test execution
rather than only persistence.

### `test`

Purpose:

- run a real connection test for one config
- persist the result in `connection_tests`

Later it may support:

- quick TCP-only tests
- real-delay tests
- timeouts

### `connect`

Purpose:

- generate runtime config from a stored profile
- launch the Xray process
- mark config/session state appropriately

Later it may support:

- mixed port override
- background mode
- log level

### `disconnect`

Purpose:

- stop the current runtime session
- update `runtime_sessions`
- clear active runtime state if needed

## Relationship To Existing Work

This phase is enabled by the persistence work that is already done.

The following pieces already exist and should now be consumed by the CLI:

- `configs` table and repository methods
- config lifecycle methods:
  - selection
  - activation
  - enable/disable
  - soft delete
  - restore
- `connection_tests` repository methods
- `runtime_sessions` repository methods
- app home and default file paths

So Phase 2.5 is not about inventing new storage; it is about exposing the
existing storage through a proper user-facing command model.

## Shared Flags

The following shared flags should exist at the top level.

### `--database`

Purpose:

- override the default SQLite database path

Default behavior without override:

- `XRAT_PATH/db.sqlite`
- or `$HOME/.config/xrat/db.sqlite`

### `--config`

Purpose:

- override the default config file path

Default behavior without override:

- `XRAT_PATH/Config.toml`
- or `$HOME/.config/xrat/Config.toml`

These flags should be treated as global flags rather than repeated separately in
every command.

## Detailed Implementation Plan

### Step 1. Refactor the top-level CLI into subcommands

Change:

- from a top-level input argument
- to a top-level command enum

Minimum first implementation:

- `import`
- `add`

This should preserve current behavior while establishing the new structure.

Initial implementation notes:

- shared flags stay at the top level:
  - `--database`
  - `--config`
- `import` remains the batch/subscription-oriented path
- `add` is the manual single-config path
- `add` should reject inputs that expand into zero or multiple configs
- the runtime/bootstrap path resolution should remain shared regardless of
  command

### Step 2. Move current import behavior under `import`

Current behavior:

- parse one input source
- normalize
- import to SQLite

New behavior:

- `xrat import <input>`

No change in persistence logic should be required beyond command dispatch.

### Step 3. Add `add`

Use the existing parsing and persistence pipeline, but with a command intended
for one config rather than a batch or subscription source.

This gives the app a better manual-entry flow and avoids overloading `import`
for both one-off and batch use cases.

Recommended initial behavior:

- accept one config URI/string argument
- parse it through the same normalization and dedup pipeline
- require exactly one parsed node
- persist it using `source_kind = raw_text`
- print a small import-style summary so the user can confirm what was stored

### Step 4. Add `list`

Use existing DB query methods to show stored configs.

This is likely the first command that makes the new persistence visible to the
user.

### Step 5. Add lifecycle commands

Commands:

- `select`
- `enable`
- `disable`
- `delete`
- `restore`

These are thin CLI wrappers around repository methods that already exist.

### Step 6. Add `show` and `status`

These improve observability and make the app more usable before full runtime
control is added.

### Step 7. Add `test`, `connect`, and `disconnect`

These commands should come after the command structure is stable and after
runtime behavior is better defined.

## Suggested Completion Criteria

Phase 2.5 can be considered complete after:

1. XRAT uses subcommands rather than a single top-level input argument
2. shared flags such as `--database` and `--config` are global
3. the current import flow is available under `xrat import`
4. a manual single-config flow is available under `xrat add`
5. at least one read command such as `list` is available
6. at least one lifecycle command such as `select` or `enable` is available
7. help output is clear enough that users can discover commands without reading
   source code

## Current Execution Order

To keep the rollout small and safe, implementation should start in this order:

1. refactor the CLI to `xrat COMMAND [ARGS] [FLAGS]`
2. move existing logic to `xrat import <input>`
3. add `xrat add <config-uri>`
4. verify help output, parsing behavior, and database persistence
5. continue with read commands such as `list`
6. split CLI definitions and command handlers into folders before more commands
   are added

## Notes

- this phase should avoid overengineering command trees too early
- a minimal but extensible command layout is better than implementing every
  planned command immediately
- command names should stay short and conventional
- global flags should remain few and focused
- per-command flags should be added only where they improve actual usage

## Implementation Review Notes

Current status: Phase 2.5 is not complete.

The command-first CLI foundation is mostly in place:

- `src/cli/root.rs` defines a top-level `Cli` with a `Command` subcommand enum.
- shared flags such as `--database` and `--config` are global.
- `src/cli/command.rs` exposes `import`, `add`, and `list`.
- `src/app/commands/` contains separate command handlers for implemented
  commands.
- `src/main.rs` remains a thin bootstrap and dispatch entrypoint.
- `xrat list configs`, `xrat list nodes`, `xrat list subscriptions`, and
  `xrat list subs` are implemented with basic filters.

Blocking gap:

- Phase 2.5 completion criteria require at least one lifecycle command such as
  `select` or `enable`.
- The current CLI command enum does not expose `select`, `enable`, `disable`,
  `delete`, or `restore`.
- `src/app/commands/mod.rs` has no dispatch path for lifecycle command handlers.
- Repository lifecycle methods already exist, but there is no user-facing CLI
  wrapper for them yet.

Required work before this phase can move to `DONE.md`:

1. Add at least one lifecycle command, preferably the small complete set:
   `select`, `enable`, `disable`, `delete`, and `restore`.
2. Add matching command handlers under `src/app/commands/`.
3. Wire the handlers to the existing repository lifecycle methods.
4. Add CLI parsing tests for the new lifecycle command(s).
5. Add a regression test or command-level test that verifies the lifecycle
   operation changes persisted config state as expected.

Validation performed during review:

- `cargo test -q cli::tests` passed.

## Completion blockers

**Reviewed: 2026-06-01**
**Resolved: 2026-06-01**

All blockers have been resolved:

1. **Lifecycle commands added to CLI** - Added `select`, `enable`, `disable`, `delete`, `restore`, and `show` commands to `src/cli/command.rs` and `src/cli/lifecycle.rs`. Command handlers implemented in `src/app/commands/lifecycle.rs`.

2. **`show` command implemented** - Added `show` command with `--json` flag for detailed config inspection.

3. **Completion criteria item 6 met** - All lifecycle commands (`select`, `enable`, `disable`, `delete`, `restore`) are now available via CLI.

4. **CLI parsing tests added** - Added 8 new CLI parsing tests for lifecycle commands in `src/cli/tests/cases/core_cases.rs`.
