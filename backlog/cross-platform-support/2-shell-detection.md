# 02.2 Easy, P3: Shell detection via `sysinfo`

**Difficulty:** Easy — 30 minutes, one file.

**File:** `src/app/commands/proxy/shell.rs:239-244`

Replace `/proc/{ppid}/comm` with cross-platform
[`sysinfo`](https://crates.io/crates/sysinfo):

```rust
fn parent_process_name() -> Option<String> {
    let ppid = std::os::unix::process::parent_id();
    let mut system = sysinfo::System::new();
    system.refresh_process(sysinfo::Pid::from_u32(ppid));
    let p = system.process(sysinfo::Pid::from_u32(ppid))?;
    Some(p.name().to_string_lossy().to_string())
}
```

`sysinfo` covers macOS (via `proc_pidpath`), FreeBSD (via `sysctl`), OpenBSD
(via `kvm`), and Linux.

Since this is best-effort (already returns `None` gracefully on failure), there
is no regression risk.

**Deps:** Add `sysinfo` to `Cargo.toml`.

**Verification:** `cargo test` + manual `xrat proxy shell status` on each
target.
