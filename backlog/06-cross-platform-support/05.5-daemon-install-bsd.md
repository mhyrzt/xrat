# 05.5 Medium, P3: Daemon install for FreeBSD/OpenBSD (rc.d)

**Difficulty:** Medium — half day.

**Files:** `src/app/commands/daemon_install.rs`, new `packaging/rc.d/` templates

Add rc.d script generation behind `#[cfg(target_os = "freebsd")]` /
`#[cfg(target_os = "openbsd")]`:

```sh
# packaging/rc.d/xrat-daemon
. /etc/rc.subr

name="xrat_daemon"
rcvar="xrat_daemon_enable"
command="{{EXE}}"
command_args="--config {{XRAT_PATH}}/config.toml daemon run-server"
pidfile="{{XRAT_PATH}}/daemon.pid"

load_rc_config $name
run_rc_command "$1"
```

Commands:

- Enable: `sysrc xrat_daemon_enable=YES`
- Start/stop: `service xrat-daemon start|stop`

**No library needed.**
