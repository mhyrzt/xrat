---
id: TASK-37
title: Add DNS Resolver Port
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
ordinal: 17
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/improvement/refactor/3-ports/17-dns-resolver-port.md`

# Add DNS Resolver Port

## Finding

### [Priority: Low] Add a DNS resolver port for probers

**Files involved:**

- `src/prober/tcp/check.rs:11`
- `src/prober/icmp/mod.rs:43`

**Problem:** `tokio::net::lookup_host` is called directly in the TCP and ICMP
probers. DNS failures, slow resolution, and empty results cannot be simulated in
tests without real DNS queries.

**Why this change is needed:** Network probe tests that exercise DNS failure
paths currently require either real DNS that happens to fail or invasive test
infrastructure. Extracting a port makes probe tests more reliable and adds the
ability to assert probe behavior under DNS errors.

**How to implement it:** Introduce a `DnsResolver` trait with a single method.
Provide a `TokioDnsResolver` production adapter and a `MockDnsResolver` test
adapter. Inject into prober constructors.

**Positive effect on the codebase:** Probe tests can inject canned IP results or
simulate `NXDOMAIN` errors. The change is small and contained.

**Suggested target architecture:** `DnsResolver` port in `src/prober/` or
`src/support/`. Injected into TCP and ICMP prober services.

**Risk / migration notes:** Low risk. Two call sites, simple replacement.
<!-- SECTION:DESCRIPTION:END -->
