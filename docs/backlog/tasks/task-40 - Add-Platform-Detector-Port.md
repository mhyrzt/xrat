---
id: TASK-40
title: Add Platform Detector Port
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
labels:
  - legacy-import
  - improvement
  - refactor
milestone: m-4
dependencies: []
priority: medium
ordinal: 20
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/20-platform-detector-port.md`

# Add Platform Detector Port

## Finding

### [Priority: Low] Add a platform detector port for OS/arch detection

**Files involved:**

- `src/app/commands/upgrade/release.rs`
- `src/app/commands/proxy/desktop.rs`
- `src/prober/icmp/mod.rs`

**Problem:** `cfg!(target_os)` checks and `std::env::consts::ARCH` are scattered
across upgrade binary URL construction, desktop proxy gsettings gating, and ICMP
ping flag selection. These compile-time and env-based checks cannot be
overridden in tests.

**Why this change is needed:** Testing platform-specific behavior (e.g., upgrade
on a non-Linux target, ICMP on macOS) requires running the test on that
platform. A trait would let tests simulate different platforms, but the value is
low since platform checks are essentially constants.

**How to implement it:** Introduce a `PlatformDetector` trait returning OS and
architecture enums. Provide a `HostPlatformDetector` production adapter. Add a
`MockPlatformDetector` for tests that need to verify cross-platform behavior.

**Positive effect on the codebase:** Upgrade URL selection, desktop proxy
gating, and ICMP command selection become testable on any host platform.

**Suggested target architecture:** `PlatformDetector` port in `src/support/`.
Injected into upgrade, desktop proxy, and ICMP services.

**Risk / migration notes:** Very low risk. Low value since platform rarely
changes in a test session. Consider deferring until there is an explicit need to
test cross-platform behavior on a single host.
<!-- SECTION:DESCRIPTION:END -->
