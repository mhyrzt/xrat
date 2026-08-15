---
id: TASK-47
title: 'proxy shell: select protocol on enable'
status: Done
assignee:
  - '@mahyar'
created_date: '2026-07-31 21:08'
updated_date: '2026-07-31 21:25'
labels:
  - cli
  - proxy-shell
dependencies: []
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
xrat proxy shell enable should accept a trailing flag or positional arg indicating which proxy protocol to use: http, socks5, or socks5h. Applies to the enable subcommand (last positional/flag).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 xrat proxy shell enable supports selecting http, socks5, or socks5h protocol
- [x] #2 Protocol chosen via a last positional arg or a flag on the enable subcommand
- [x] #3 Default behavior preserved when protocol not specified
- [x] #4 CLI parser tests cover each protocol value and the default
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented protocol selection: added ProxyShellProtocol enum (http/socks5/socks5h), trailing positional arg on enable, URL selection in proxy_urls (forced scheme vs default fallback). Added shell unit tests + CLI parser tests in src/cli/tests/cases/core_cases/proxy.rs. Updated docs/src/02-cli/proxy.md.

Validation: cargo fmt + clippy --all-targets -D warnings clean; cargo test -q --locked 657 passed (7 proxy_shell unit tests + 5 new CLI parser tests).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added optional trailing protocol arg (http|socks5|socks5h) to xrat proxy shell enable. proxy_urls now forces scheme when protocol given (requires matching inbound) and keeps default fallback behavior otherwise. Added shell unit tests, CLI parser tests in src/cli/tests/cases/core_cases/proxy.rs, and updated docs/src/02-cli/proxy.md. Verified: full test suite + clippy clean.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
