---
id: TASK-8
title: 12.05 Windows release and upgrade
status: To Do
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-07-05 14:44'
labels:
  - legacy-import
  - feature
  - cross-platform
milestone: m-0
dependencies: []
priority: medium
ordinal: 1205
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/feature/cross-platform-support/12-05-windows-release-and-upgrade.md`

# 12.05 Windows release and upgrade

**Difficulty:** Large, P3.

Windows support requires release artifacts before install and self-upgrade can
work reliably.

## Current state

- `.github/workflows/release.yml` builds Linux musl and macOS archives.
- Release packaging always creates `.tar.gz` files.
- `src/app/commands/upgrade/release.rs` detects Linux and macOS targets only.
- Upgrade extraction shells out to `tar`, checksum verification shells out to
  `sha256sum`, and installation assumes the current binary can be overwritten.

## Target behavior

- Release CI builds Windows archives for:
  - `x86_64-pc-windows-msvc`
  - `aarch64-pc-windows-msvc`, if the dependency graph and runner support it
- Windows archives are `.zip` files containing `xrat.exe`, `LICENSE`, and
  `README.md`.
- `SHASUMS256.txt` includes both `.tar.gz` and `.zip` artifacts.
- `xrat upgrade` detects Windows target triples and downloads the matching
  `.zip`.
- Windows self-upgrade stages the new executable and completes replacement
  without trying to overwrite the running `.exe` in place.

## Implementation notes

- Use Rust-native checksum and archive extraction where practical so upgrade
  does not depend on external `sha256sum`, `tar`, or PowerShell tools.
- Keep Unix archive names stable.
- Implement Windows binary replacement with a staged file and rename-on-exit or
  helper process flow. A running `.exe` cannot be overwritten directly.
- Keep post-upgrade database migrations after the new binary is installed or
  staged successfully.
- Update release notes guidance only if the workflow behavior changes for
  maintainers.

## Tests and verification

- Add unit tests for target detection and archive-name selection.
- Add extraction/checksum tests for `.zip` and `.tar.gz` paths.
- Add Windows CI build coverage for release targets before publishing archives.
- On Windows, verify `xrat upgrade --version <tag>` from an older release.

## Completion criteria

- GitHub releases publish Windows `.zip` artifacts and checksums.
- `xrat upgrade` works from Windows without external Unix tools.
- Existing Linux and macOS upgrade behavior remains compatible with published
  archive names.
<!-- SECTION:DESCRIPTION:END -->
