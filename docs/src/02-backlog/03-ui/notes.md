# XRAT UI Facilitator Notes

This folder prototypes the future Ratatui interface in HTML/CSS before the Rust
TUI implementation.

## Current Prototype

- `index.html` models a full control-deck screen for configs, subscriptions,
  testing, imports, runtime status, and keybindings.
- `styles.css` defines a warm terminal visual direction with high-contrast
  panels, table state colors, progress UI, and mobile fallbacks.

## Ratatui Mapping

- Top bar: app title, current runtime engine, local mixed port, process state.
- Left panel: mode selector and filters matching `list configs` / `test` flags.
- Main table: `ConfigRecord` rows with protocol, address, port, network, flags,
  and latest connection-test summary.
- Detail panel: focused config metadata and actions for activate, test, select,
  import, and search.
- Lower panels: bulk test progress, import/add drawer feedback, and subscription
  list.
- Footer: global keybindings.

## Data Sources

- Config rows map to `configs` plus latest `connection_tests` by `config_id`.
- Subscription panel maps to `subscriptions` with config counts.
- Runtime pill maps to latest `runtime_sessions` row.
- Test progress maps to bulk `test` execution state.

## Interaction Draft

- `j/k` or arrows: move focus in current list.
- `tab`: cycle panels.
- `enter`: activate focused config.
- `space`: toggle selected state.
- `t`: test focused config or current filtered set.
- `i`: open import drawer.
- `/`: search/filter configs.
- `q`: quit or close modal.

## Config Deletion UX

XRAT should support both soft delete and hard delete, with the user choosing the
behavior explicitly.

- Soft delete should be the default destructive-looking action in Phase 6 UI.
- Soft-deleted configs stay in the database with history intact and can be shown
  through a deleted/archived filter.
- Restore should be available for soft-deleted configs.
- Hard delete should be labeled as `Purge` or `Hard delete`, require a stronger
  confirmation, and explain that the config row is permanently removed.
- The UI should show deleted state in the table/detail panel once the DB schema
  exposes `is_deleted`/`deleted_at` or equivalent fields.
- Phase 5 API list/detail responses should surface deleted-state metadata once
  available, but mutation endpoints can remain deferred.

Suggested keybinding direction:

- `d`: soft delete focused config.
- `r`: restore focused soft-deleted config.
- `D`: hard delete/purge focused config after confirmation.
