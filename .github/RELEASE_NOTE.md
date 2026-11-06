## xrat v0.35.1

This release adds complete Xray JSON profiles, automatic daemon connection
restore, and more reliable runtime status reporting.

### Xray JSON profiles

- Import and run complete Xray JSON configurations while preserving the full
  profile in the database.
- Use these profiles through the CLI, TUI, and daemon. Operations that cannot
  safely manage a raw profile are rejected explicitly.
- Profile data is included in database backup and restore.

### Daemon startup

- Restore the previously active connection when the daemon starts after a
  normal shutdown or reboot.
- Preserve an intentional manual disconnect and retry startup when the network
  is not ready yet.

### Runtime status

- Report the active runtime session even when a historical record has a later
  wall-clock timestamp, keeping process and TUN health aligned with the running
  connection.

### Upgrade notes

- No database migrations are required.
- Windows support remains experimental; automated checks do not replace
  interactive Windows and actual TUN traffic acceptance work.

Thanks to [@yarreg](https://github.com/yarreg) for reporting these behaviors in
[#289](https://github.com/mhyrzt/xrat/issues/289),
[#290](https://github.com/mhyrzt/xrat/issues/290), and
[#291](https://github.com/mhyrzt/xrat/issues/291).

**Full Changelog**: https://github.com/mhyrzt/xrat/compare/v0.35.0...v0.35.1
