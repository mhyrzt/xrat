# Small, P2: Clean up TUI hotkeys for subscriptions and API sharing

### Status

Planned

### Goal

Make subscription refresh available from anywhere in the TUI and replace the
current API sharing hotkeys with bindings that do not conflict with common view
actions or feel tied to the Subscriptions tab. While updating the help/keymap
docs, make search cancellation behavior explicit and align it with common TUI
expectations. Also tighten the help modal so it uses fewer rows and only shows
useful, accurate hints.

### Current behavior

- Subscription refresh is only bound in the Sources view:
  - `r` refreshes the focused subscription.
  - `R` refreshes all subscriptions.
- In the Configs view, `R` restarts the runtime. This means `R` changes meaning
  by tab and makes refresh-all unavailable outside Sources.
- API sharing is also only bound in the Sources view:
  - `u` opens the API subscription QR modal.
  - `U` copies the API subscription URL.
- The help modal lists these under API, but the bindings only work when the
  Sources view is active.
- Search help currently highlights `Ctrl+U` for clearing search input, but does
  not make the expected cancel/exit-search behavior prominent.
- The Log help section includes `[ ] Cycle log tabs`, which should be removed
  from the modal.
- Navigation help uses separate rows for related movement keys and mixes "move
  row" with "scroll" wording.

### Changes required

- Add a global refresh-all subscription binding that works from Configs,
  Sources, and any focused panel, including log/detail/runtime focus.
- Keep focused-subscription refresh scoped to Sources unless there is a clear
  focused subscription in the current view.
- Move API sharing off `u`/`U` to a less surprising binding under an API chord.
- Ensure runtime controls remain easy to use and do not conflict with global
  subscription refresh.
- Keep `Esc` as the primary way to cancel/exit search mode.
- Keep `Ctrl+U` for clearing the current search input while staying in search
  mode.
- Remove the `[ ] Cycle log tabs` row from the Log section of the help modal.
- Group related Navigation bindings to reduce modal height, for example:
  - `j, Down / k, Up`: scroll down/up.
  - `PgUp / PgDn`: page up/down.
  - `Home / End`: jump to top/bottom.
- Use "scroll" consistently for row/log/detail movement labels instead of mixing
  "move row" and "scroll".
- Update the help modal so every listed binding works in the contexts described.
- Update keymap tests to cover the global refresh binding, the new API bindings,
  and the removed/conflicting old bindings.

### Proposed keymap

- `u`: update all subscriptions from any main TUI context.
  - Rationale: `u` reads as update and is easier to trigger than a chord for a
    frequent global action.
  - This replaces the current Sources-only `R` refresh-all binding.
- `r`: refresh focused subscription, Sources view only.
  - Rationale: focused refresh only has an obvious target in the Subscriptions
    table.
- `R`: restart runtime, Configs view only.
  - Rationale: keep the existing runtime mnemonic and remove the cross-tab
    conflict where `R` can mean either restart or refresh-all.
- `a q`: show API subscription QR from any main TUI context.
- `a c`: copy API subscription URL from any main TUI context.
  - Rationale: `a` is an API leader, while `q` and `c` name the two API share
    actions. These keys do not consume single-letter view actions and keep API
    sharing available without tying it to the Subscriptions tab.
- Search mode:
  - `Esc`: cancel/exit search and return to normal TUI navigation.
  - `Ctrl+U`: clear search input and remain in search mode.
  - Rationale: `Esc` is the conventional modal cancel key; `Ctrl+U` is better
    understood as line/input clearing.

Do not keep `u`/`U` as API aliases. `u` should have one global meaning: update
subscriptions.

### Implementation plan

1. Extend chord handling so the API leader `a` is available from all main TUI
   views, not only the Configs view.
2. Map `a q` to `OpenQrApiUrl` and `a c` to `CopyApiUrl`.
3. Map plain `u` to `RefreshAllSources` globally after modal/search handling.
4. Remove the Sources-only `U` refresh/API conflict by dropping `U` as an API
   shortcut and no longer using `R` for refresh-all.
5. Keep Sources-only `r` as `RefreshFocusedSource`.
6. Update the help modal:
   - Navigation: group related movement keys and use "scroll" wording.
   - Subscriptions: `u` update all, `r` refresh focused.
   - API: `a q` show API QR, `a c` copy API link.
   - Runtime: `R` restart.
   - Search: `Esc` cancel search, `Ctrl+U` clear search.
   - Log: remove `[ ] Cycle log tabs`.
7. Add or update keymap tests for each changed binding and for the old bindings
   returning `None` or their new action as appropriate.

### Verification

- Unit tests in `src/tui/keymap/tests/` cover:
  - refresh-all subscriptions from Configs, Sources, Logs, and non-table focus.
  - runtime restart still works from Configs.
  - API QR/copy bindings work from every main view.
  - old `u`/`U` API bindings no longer trigger API actions.
  - old Sources-only `R` refresh-all binding no longer triggers refresh-all.
  - `Esc` exits search mode and `Ctrl+U` clears search input without exiting.
  - help modal snapshots/text expectations, if present, reflect the grouped
    Navigation rows and removed Log tab row.
- Manual:
  - Open the TUI, switch between tabs and focused panels, and confirm the
    subscription refresh-all shortcut works everywhere.
  - Confirm the help modal matches actual behavior.

### Open decisions

- Whether refresh-focused should gain a global binding that targets the source
  for the currently focused config, or stay Sources-only.
