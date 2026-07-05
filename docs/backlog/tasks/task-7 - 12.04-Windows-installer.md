---
id: TASK-7
title: 12.04 Windows installer
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
ordinal: 1204
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/feature/cross-platform-support/12-04-windows-installer.md`

# 12.04 Windows installer

**Difficulty:** Medium, P3.

Windows needs a native PowerShell install path instead of extending the POSIX
`install.sh` script.

## Current state

- `install.sh` is POSIX shell and is intentionally Unix-focused.
- Release archives are `.tar.gz` and contain Unix-style binaries today.
- `xrat setup` owns first-run initialization after the binary is installed.

## Target behavior

- Add `install.ps1` for Windows users.
- Detect `x86_64` and `aarch64` Windows hosts and select the matching
  `*-pc-windows-msvc` release archive.
- Download the requested version or latest release from GitHub.
- Verify `SHASUMS256.txt` with `Get-FileHash -Algorithm SHA256`.
- Expand the `.zip` archive, install `xrat.exe`, and add the install directory
  to the user's `PATH` when requested.
- Optionally run `xrat setup` and optionally register the Windows Service.

## Implementation notes

- Keep PowerShell flags aligned with existing installer intent: noninteractive
  install, version selection, install directory override, and setup skip/setup
  yes behavior where practical.
- Use user-writable defaults such as a directory under `%LOCALAPPDATA%` unless
  the caller requests a system install path.
- Do not require administrator privileges unless the user asks for service
  registration or a protected install directory.
- Print concise next steps when PATH changes require opening a new shell.

## Tests and verification

- Add a syntax check with PowerShell in CI when possible.
- Verify checksum mismatch fails before extraction.
- Verify install into a temp directory with a locally staged release archive.
- On Windows, verify install, reinstall, PATH update, setup skip, setup run, and
  service registration options.

## Completion criteria

- Windows users can install a release without Git, Rust, or POSIX shell tools.
- Installer docs include the PowerShell command and security expectations.
- `install.sh` remains Unix-only and unchanged in behavior.
<!-- SECTION:DESCRIPTION:END -->
