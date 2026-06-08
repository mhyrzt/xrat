# Easy, P2: Desktop icon shows in DE taskbar

### Status

Planned

### Goal

When launched from app menu/search, xrat should show the xrat icon in the
taskbar/dock instead of the terminal emulator icon.

### Current behavior

`packaging/desktop/xrat.desktop` uses `Terminal=true`, so DE launches inside a
terminal window and the taskbar icon belongs to that terminal.

### Changes required

- Decide approach:
  - document as known DE limitation, or
  - implement terminal-specific wrapper/WM_CLASS strategy.
- Keep `StartupNotify=true` for launch feedback.
- Document supported terminals/DE behavior matrix if wrapper approach is chosen.

### Possible root cause

xrat is a TUI app with terminal-owned window identity; `.desktop` metadata does
not control taskbar icon when terminal emulator owns the window class.

### Verification

- Launch from app menu in target DE(s) and validate icon behavior matches the
  selected approach (fix or documented limitation).
