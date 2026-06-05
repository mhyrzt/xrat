# Backlog

## Move Import Ingestion Out of the TUI

The TUI currently exposes an import modal from the Sources tab. Technically it
can import raw config links, but that creates `raw_text` source rows that cannot
be refreshed, copied, or shared like real subscription sources. This makes the
Sources tab model less clear.

Proposed direction:

- Remove the TUI import modal and the `i` import key from the Sources tab.
- Keep source refresh actions in the TUI for already-known, refreshable sources.
- Handle all new ingestion explicitly through CLI commands:
  - `xrat import <input>` for subscriptions, files, raw link lists, and other
    supported import inputs.
  - `xrat add <link>` for adding exactly one config link.
- Update TUI empty states, help text, and docs to point users to the CLI import
  commands instead of offering in-app import.
- Ensure the Sources tab only presents operations that make sense for stored
  sources: inspect, rename, refresh, delete, copy/QR when a source URL exists.

Success criteria:

- No TUI action opens an import modal.
- TUI docs and help no longer advertise import from the TUI.
- Empty Configs/Sources states show the relevant CLI command.
- Refresh/copy/QR behavior remains available for existing URL-backed sources.

## Reconcile Removed Configs on Subscription Refresh

Refreshing a subscription currently upserts configs returned by the provider but
preserves old configs that no longer appear in the refreshed subscription. That
leaves stale provider-removed configs attached to the source.

Proposed direction:

- Treat subscription refresh as source reconciliation, not additive import.
- After parsing the refreshed subscription, purge configs currently attached to
  that subscription whose dedup keys are absent from the new provider payload.
- Keep upsert behavior for configs that are still present.
- Keep explicit source deletion behavior unchanged: deleting a source still
  deletes the source and all of its configs.
- Decide whether manual `xrat import <url>` should use the same reconciliation
  behavior when the URL maps to an existing subscription.

Success criteria:

- Refreshing a URL-backed subscription removes configs that disappeared from the
  provider payload.
- Existing configs still present in the provider payload are updated normally.
- Tests cover SQLite and Postgres paths where practical.
- TUI refresh status reports both imported/updated and purged counts.

## Implement Automatic Subscription Refresh

Subscriptions can be refreshed manually from the TUI, but there is no automatic
subscription refresh/update scheduler yet. Users should be able to opt in to
periodic refresh so URL-backed subscriptions stay current without manual `r` /
`R` actions.

Proposed direction:

- Add config for subscription auto-refresh, for example:
  - `[subscriptions].auto_refresh = false`
  - `[subscriptions].refresh_interval_hours = 24`
- Add a daemon-managed scheduler that refreshes URL-backed subscriptions when
  auto-refresh is enabled.
- Reuse the same refresh/import path as manual TUI refresh, including provider
  removal reconciliation once that backlog task is implemented.
- Skip non-refreshable sources such as raw-text/manual imports and sources with
  no URL.
- Record refresh start/success/failure through app events so `xrat logs` and
  the TUI can report what happened.

Success criteria:

- URL-backed subscriptions refresh automatically when enabled.
- Refresh intervals are respected across daemon restarts.
- Failed subscription refreshes are logged and do not crash the daemon.
- Manual TUI refresh remains available and uses the same reconciliation logic.
- Docs explain the difference between manual refresh and automatic refresh.

## Refresh Subscriptions Before Proxy Rotation

Proxy rotation is implemented and can run automatically through the daemon, but
it does not refresh URL-backed subscriptions before choosing a replacement
candidate. The intended rotation flow should use the freshest available provider
configs before testing and reconnecting.

Desired flow:

1. Current runtime config becomes unhealthy, timer rotation fires, or user
   triggers rotation.
2. Refresh URL-backed subscriptions.
3. Reconcile provider-removed configs according to the subscription refresh
   behavior.
4. Test eligible enabled configs.
5. Pick the passing config with the lowest real-delay latency.
6. Replace the runtime with the selected config on the same local inbound ports.

Proposed direction:

- Add a setting such as `[runtime.rotation].refresh_subscriptions = true`.
- Before automatic timer/health rotation, refresh all URL-backed subscriptions.
- Decide whether manual `xrat proxy rotate` should refresh by default or expose
  an explicit flag such as `--refresh`.
- Keep non-refreshable sources out of this path.
- Ensure rotation status/events report refresh failures separately from candidate
  test failures.
- Consider changing manual rotation without `--config-id` to run the same fresh
  candidate test pass as automatic rotation, instead of relying primarily on
  latest persisted test results.

Success criteria:

- Rotation can refresh subscriptions before candidate selection.
- Refreshed configs are included in the rotation test pass.
- Removed provider configs are not selected after refresh reconciliation.
- Failures are logged without leaving the old runtime stopped unnecessarily.
- Docs describe the full rotation flow and the manual/automatic differences.
