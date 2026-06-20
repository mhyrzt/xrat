## xrat v0.9.1

This release improves subscription renaming in the TUI.

### Improvements

- **Compact subscription rename dialog.** The dialog now sizes itself to its
  content, identifies the subscription by reference and current name, and
  prefills a bordered input for fast editing. Save and cancel controls live in
  the bottom border, while rename failures remain visible below the input.

### Upgrade notes

- No new database migrations; safe drop-in upgrade.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.9.0...v0.9.1
