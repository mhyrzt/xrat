# Consolidate Local IP Discovery

## Finding

### [Priority: Low] Consolidate local IP discovery into shared helper

**Files involved:**

- `src/support/net.rs:13-15`
- `src/app/commands/runtime_output.rs:22-25`

**Problem:** The same `UdpSocket::bind("0.0.0.0:0")` + `connect("8.8.8.8:80")`

- `local_addr()` trick is implemented in two places: once as a shared helper in
  `support/net.rs` and once inlined in `runtime_output.rs`. Both are concrete
  system calls with no test seam.

**Why this change is needed:** The duplication is unnecessary. When the
`runtime_output.rs` version drifts from the shared version, behavior becomes
inconsistent.

**How to implement it:** Remove the inline duplication and have
`runtime_output.rs` use `support::net::primary_ip()`. Optionally extract a
`LocalIpResolver` trait with a `MockLocalIpResolver` for tests if this function
needs to be testable (currently low value).

**Positive effect on the codebase:** One less piece of duplicated socket logic.
The primary IP resolution is in one place.

**Suggested target architecture:** Keep in `src/support/net.rs` as a concrete
helper. A trait is only warranted if tests need to control the resolved IP.

**Risk / migration notes:** Very low risk. Mechanical replacement of inline code
with a function call.
