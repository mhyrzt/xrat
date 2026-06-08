# Add Clipboard Port

## Finding

### [Priority: Low] Add a clipboard abstraction for TUI share

**Files involved:**

- `src/tui/run/tasks/share.rs:6,83-103`

**Problem:** `arboard::Clipboard::new()` + `set_text()` is called directly in
the TUI share task. The clipboard is a system GUI dependency that fails in
headless environments and cannot be mocked.

**Why this change is needed:** The TUI share test either requires a display
server or is skipped. An abstraction would let tests verify copy behavior
without a clipboard backend.

**How to implement it:** Introduce a `Clipboard` trait with a `copy_text`
method. Provide a `ArboardClipboard` production adapter and a `MockClipboard`
test adapter that stores the last copied text in memory.

**Positive effect on the codebase:** TUI share feature becomes testable in
headless CI. Clipboard failures (common in SSH sessions) can be handled
gracefully with a fallback that logs the text instead of crashing.

**Suggested target architecture:** `Clipboard` port in `src/support/` or
`src/tui/ports/`. Injected into the TUI share task.

**Risk / migration notes:** Very low risk. Minor feature, small surface area.
