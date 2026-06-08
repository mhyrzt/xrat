# 03.3 Medium, P3: Process reattach via `sysinfo`

**Difficulty:** Medium — 1–2 days, one file plus testing.

**File:** `src/app/runtime_service/reattach/process.rs`

Replace all `/proc/{pid}/exe` and `/proc/{pid}/cmdline` reads with
`sysinfo::System` queries:

```rust
use sysinfo::{Pid, Process, System};

pub(super) struct SystemProcessInspector;

impl ProcessInspector for SystemProcessInspector {
    fn is_running(&self, pid: i64) -> bool {
        xray_runtime::process_is_running(pid)
    }

    fn exec_matches_runtime_engine(&self, context: &AppContext, session_id: i64, pid: i64) -> bool {
        let mut s = System::new();
        s.refresh_process(Pid::from(pid as u32));
        let exe = s.process(Pid::from(pid as u32))
            .and_then(|p| p.exe().map(|p| p.to_path_buf()));
        // ... compare against expected runtime_paths.xray_path / sing_box_path
    }

    fn cmdline_matches_session_config(&self, context: &AppContext, pid: i64, session_id: i64) -> bool {
        let mut s = System::new();
        s.refresh_process(Pid::from(pid as u32));
        let cmd = s.process(Pid::from(pid as u32))
            .map(|p| p.cmd().to_vec());
        // ... check cmdline contains session config path
    }
}
```

**Platform caveats with sysinfo:**

- **macOS:** `proc_pidpath()` works but hardened runtime may need
  `com.apple.security.get-task-allow` entitlement in some configurations.
- **FreeBSD:** `sysctl kern.proc.args` — works in userland, may be restricted
  inside jails.
- **OpenBSD:** `sysinfo` process info is partial; `exe()` may return `None`.
  Fallback: parse `ps -o command= -p <pid>` output.

**Fallback strategy if sysinfo isn't enough for OpenBSD:**

```
macOS:   libc::proc_pidpath() + libc::sysctl(KERN_PROC_ARGS)
FreeBSD: libc::sysctl(KERN_PROC_PATHNAME) + KERN_PROC_ARGS
OpenBSD: kvm_getargv() via kvm interface, or ps(1) parsing
```

**Deps:** Add `sysinfo` (or keep conditional `libc` FFI blocks).

**Verification:**

- Unit test: spawn a known process, verify inspector finds it
- Integration: daemon restart during active session, confirm reattach succeeds
- Run on macOS/FreeBSD/OpenBSD
