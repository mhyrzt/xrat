# Easy, P2: Improve TUI log wrapping and scroll navigation

### Status

Draft

### Problem

Long log rows in the TUI logs card can wrap awkwardly and make the table hard to
read. For example, a long event kind/message can split across lines in a way
that hides the relationship between timestamp, level, source, kind, and message.

Current rough output:

```text
2026-06-09 11:26:00  info   runtime       daemon_restart_stale_pid_recove…  Reconnected config 3 after stale runtime
PID on daemon start
```

### Scope

Applies to the logs card first, but the navigation behavior should be consistent
with other scrollable TUI cards such as configs.

### Options

- Wrap long message text inside the message column while keeping fixed metadata
  columns aligned.
- Make the whole card horizontally scrollable when preserving one row per log
  entry is more readable.
- Keep truncation/ellipsis only where the full text can still be reached by
  scrolling or opening details.

### Required hotkeys

Ensure scrollable cards support:

- `PgUp` — page up
- `PgDown` — page down
- `Home` — jump to first row/top of view
- `End` — jump to last row/end of view

### Changes required

- Audit existing TUI scroll state and keymap handling for logs, configs, and
  other card/table views.
- Add or normalize page/top/bottom actions in `src/tui/keymap/`.
- Update the logs card renderer so long values are readable without breaking row
  context.
- Update help/chrome text to show the new navigation keys where relevant.

### Verification

- Keymap tests for `PgUp`, `PgDown`, `Home`, and `End`.
- TUI state tests for top/bottom/page navigation where state is testable.
- Manual verification with long event kinds and messages in the logs card.
