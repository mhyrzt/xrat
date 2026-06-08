# 04.4 Medium, P2: Daemon install for macOS (launchd)

**Difficulty:** Medium — half day.

**Files:** `src/app/commands/daemon_install.rs`, new `packaging/launchd/`
templates

Add launchd plist support behind `#[cfg(target_os = "macos")]`:

```xml
<!-- packaging/launchd/xrat-daemon.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>xrat-daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{{EXE}}</string>
        <string>--config</string>
        <string>{{XRAT_PATH}}/config.toml</string>
        <string>daemon</string>
        <string>run-server</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{{XRAT_PATH}}/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>{{XRAT_PATH}}/daemon.log</string>
</dict>
</plist>
```

Commands:

- Install: `launchctl bootstrap gui/$(id -u) /path/to/xrat-daemon.plist`
- Uninstall: `launchctl bootout gui/$(id -u)/xrat-daemon`
- Start/stop: `launchctl kickstart` / `launchctl kill`

Template rendering is already factored into `render_service()` — just add a
parallel `generate_daemon_plist()` and dispatch on `#[cfg]`.

**No library needed.**
