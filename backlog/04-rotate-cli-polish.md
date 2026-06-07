# 04. Medium-hard, P1: `xrat rotate` CLI polish — enable/disable, actionable info, progress

### Status

Planned

### Goal

Polish the `xrat rotate` subcommands for consistency, user feedback, and
visibility into long-running operations.

### Changes required

1. **Rename subcommands**: change `xrat rotate start` → `xrat rotate enable` and
   `xrat rotate stop` → `xrat rotate disable`.
2. **Actionable info on enable**: when the user runs `xrat rotate enable`, show:
   ```
   INFO State is volatile and resets to config defaults on daemon restart.
   ```
   Also show **which config file** controls rotation settings and **how** to
   make the change permanent (i.e. which key to set in which config file).
3. **Progress for blocking operations**: `xrat rotate now` is a blocking task.
   Show live logs of what xrat is doing in real time. If the operation involves
   bulk sub-tasks (e.g. testing many configs), render a **progress bar** so the
   user can see how far along it is.

### Verification

- CLI parser tests for `enable`/`disable` subcommands.
- Integration test: `xrat rotate enable` output contains the info message, the
  config file path, and instructions.
- Integration test: `xrat rotate now` shows progress for bulk operations.
- Manual: confirm `xrat rotate disable` stops rotation.
