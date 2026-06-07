# 05. Medium-hard, P2: Background activity indicators in TUI

### Status

Planned

### Goal

Replace the current zero-feedback update behavior with animated, running- state
indicators so the user can see at a glance that sources/subscriptions are being
reloaded and the runtime card reflects current activity.

### Priority items

1. **Better indicator for sources update in TUI.** When updating sources
   (subscriptions) from the TUI, show an animated spinner or progress indicator,
   consistent with other background-task indicators elsewhere in the TUI.
2. **Runtime card activity.** When the runtime is checking, reloading, or
   transitioning, the Runtime card should show a spinner or pulse animation
   similar to other live-state indicators.
3. Also when update completed in chrome bar middle (bottom bar) show message
   (subscription name || ref updated) or if it was all update show (All
   subscriptions updated!) or other proper message to show how many failed and
   remove the message after sometime Apply the same pattern used by other
   background tasks in the TUI so the indicator style is consistent.

---

### Sub-items

#### 5.1 Medium, P1: Dedicated `xrat update` CLI command

**Status:** Planned

Add `xrat update [SUBS_REF...]` as a dedicated command for refreshing
subscriptions/sources, instead of relying solely on TUI or implicit triggers.

Changes required:

- New CLI command `xrat update` that triggers subscription updates.
- If optional subscription references are given, update only those; otherwise
  update all.
- Extract the shared update/subscription-reload logic from the TUI into a
  reusable command handler under `src/app/commands/` so both the CLI and TUI use
  the same path.
- Wire the new handler into the existing subscription-update infrastructure
  rather than duplicating it.

Verification:

- CLI parser tests for `xrat update` with and without refs.
- Integration test: confirm running `xrat update` triggers updates and results
  are reflected in `xrat list subscriptions`.
- Confirm the TUI sources update still works after extracting shared logic.

---

#### 5.2 Easy, P2: `xrat list subscriptions` missing Updated At column

**Status:** Planned

Add an 'Updated At' (or 'Last Update') column to the `xrat list subscriptions`
output so the user can see when each subscription was last refreshed without
needing to inspect individual records.

Changes required:

- Add the column to the table/tsv/json/csv output in the list-subscriptions
  command handler.
- Use a human-readable relative or absolute timestamp consistent with other list
  output formats.

Verification:

- CLI parser/output tests for the new column and formatting.

---

#### 5.3 Easy, P2: TUI rename Sources to Subscriptions

**Status:** Planned

Rename "Sources" to "Subscriptions" throughout the TUI for consistency with the
CLI naming convention.

Changes required:

- Update TUI view labels, tab names, and any user-facing strings that refer to
  "Sources" to use "Subscriptions" instead.

Verification:

- Manual TUI verification: confirm all UI panels, tabs, and labels say
  "Subscriptions".
- Update any tests that reference old label strings.

---

#### 5.4 Easy, P2: Update docs for UI/CLI changes

**Status:** Planned

Update documentation under `docs/src/` to reflect user-facing changes from items
5.1–5.3: the new `xrat update` command, the `list subscriptions` column
addition, and the TUI naming change.

Changes required:

- Add `xrat update` to the CLI reference docs.
- Update list-subscriptions output documentation for the new column.
- Update TUI docs to use "Subscriptions" instead of "Sources".

Verification:

- Confirm `mdbook build` succeeds.
- Confirm rendered docs match actual CLI and TUI behavior.
