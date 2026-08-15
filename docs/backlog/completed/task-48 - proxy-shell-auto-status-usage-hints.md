---
id: TASK-48
title: 'proxy shell: auto status + usage hints'
status: Done
assignee:
  - '@mahyar'
created_date: '2026-07-31 21:08'
updated_date: '2026-07-31 21:35'
labels:
  - cli
  - proxy-shell
dependencies: []
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
xrat proxy shell enable/disable/toggle should automatically print the current proxy shell status after running, and print how to use the proxy with respect to the user's shell (as a comment or inside --help).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 enable/disable/toggle print xrat proxy shell status after execution
- [x] #2 Usage instructions are printed as a shell comment or shown in --help
- [x] #3 Usage hint reflects user shell (bash/zsh/fish/etc.)
- [x] #4 Tests cover status printing and usage hint generation
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation: enable/disable/toggle now call print_status_stderr (stderr, so stdout script stays eval-safe). Added usage_hint() shell comment prefixed to each script output (bash/zsh: eval, fish: | source) + toggle_on/toggle_off/enable_output/disable_output builders. Added long_about usage notes to CLI subcommands (shown in --help). Refactored print_status -> status_text + print_status_stderr. Added 6 shell unit tests + 1 CLI help test. Docs updated in docs/src/02-cli/proxy.md.

Validation: cargo fmt + clippy --all-targets -D warnings clean; cargo test -q --locked 664 passed (8 proxy_shell unit tests, 6 new CLI parser tests incl. help-text test).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
enable/disable/toggle now print xrat proxy shell status to stderr after emitting the script (stdout stays eval-safe). Each script is prefixed with a shell-aware # usage comment (bash/zsh: eval "", fish: | source), and the same usage note was added to each subcommand's --help. Added unit tests for usage_hint/output builders + a CLI help-text test; updated docs. Verified: full suite + clippy clean.
<!-- SECTION:FINAL_SUMMARY:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Acceptance criteria are satisfied or explicitly updated.
- [x] #2 Relevant tests or checks were run and recorded in the task notes.
- [x] #3 User-facing behavior changes are reflected in docs when applicable.
- [x] #4 Final summary explains what changed and any residual risk.
<!-- DOD:END -->
