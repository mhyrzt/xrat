## xrat v0.19.1

This patch release cleans up accumulated runtime logs and corrects subscription
config counts.

### Fixes

- **Automatic runtime log retention.** XRAT keeps stdout/stderr logs for the 10
  newest completed sessions and removes older completed-session logs at startup
  and before connections or replacements. Xray, V2Ray, and sing-box logs are
  covered. Starting, running, and stopping sessions are protected, and cleanup
  failures do not block normal operations. Fixes #3.
- **Accurate subscription counts.** Deleted configs no longer inflate
  subscription config counts.
- **Documentation.** Refreshed the TUI demo and corrected its colors.

### Upgrade notes

- No database migration or configuration change is required.
- Retention also applies to existing logs whose sessions remain in the database.
  Older completed-session logs are permanently deleted; copy any diagnostics you
  need before starting the upgraded application.
- Retention does not cap active log sizes or remove `daemon.log`, generated JSON
  configs, custom engine logs, or logs without matching database sessions.

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.19.0...v0.19.1
