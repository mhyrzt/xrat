# Medium, P2: Add `xrat setup` command to own post-install setup

### Status

Planned

### Goal

Add a first-class `xrat setup` command that performs the post-install setup work
currently scripted inside `install.sh`. The command should be idempotent,
re-runnable, cross-platform, and usable regardless of how the binary was
installed (release archive, `cargo install`, distro package, or manual copy).

After this lands, `install.sh` shrinks to "download/verify/place the binary,
then hand off to `xrat setup`", and the setup logic lives in testable Rust
instead of shell.

### Motivation

- Setup logic in `install.sh` is only reachable through the install script.
  Users who `cargo install xrat`, build from source, or install via a package
  manager get no guided setup.
- The shell implementation is hard to unit test and duplicates platform
  knowledge (daemon install, desktop proxy, OS detection) that already exists in
  Rust under `src/app/commands/`.
- Setup is not re-runnable today: there is no supported way to "finish setup" or
  "repair my install" after the first run.
- Moving setup into the binary keeps per-OS dispatch in one place. The daemon
  install (`src/app/commands/daemon_install.rs`) and desktop proxy
  (`src/app/commands/proxy/`) already branch on OS in Rust; `install.sh`
  re-implements parallel branching in bash.

### Current behavior (install.sh)

`install.sh` performs, in order, after placing the binary:

1. Environment detection: OS, CPU arch, shell (`print_detection`).
2. Dependency checks: `xray` required, `sing-box` optional, with install hints
   (`check_xray`, `check_singbox`).
3. `xrat init` to create the config burrow (`run_init`).
4. Optional `xrat daemon install --start` (`run_daemon_install`).
5. Optional systemd lingering via `loginctl enable-linger` on Linux
   (`run_linger_enable`).
6. PATH check for the install dir (`install_dir_in_path`,
   `show_completion_note`).
7. Man pages, shell completions, and (Linux/XDG only) desktop launcher + icons
   install (`install_extras`, `write_desktop_launcher`,
   `select_desktop_terminal`).
8. Quick-start guide (`show_guide`).

Steps 1-6 and 8 are pure setup orchestration that belong in the binary. Step 7
is split: archive extraction stays in `install.sh`, but generating/placing
completions and man pages can move into setup (the binary already has
`xrat completions <shell>` and `xrat manpage`).

### Proposed CLI

```text
xrat setup [OPTIONS]

  -y, --yes              Non-interactive; accept all recommended defaults
      --no-daemon        Do not install/start the background daemon
      --no-desktop       Skip desktop launcher + icon install (Linux/XDG only)
      --no-completions   Skip shell completion install
      --no-manpages      Skip man page install
      --linger           Enable boot-before-login start (Linux; implies daemon)
      --check            Diagnose only: report what is/!isn't set up, change
                         nothing (exit non-zero if required steps are missing)
      --format <fmt>     For --check: table|json (reuse output.rs conventions)
```

Defaults match `install.sh`: interactive prompts, daemon offered yes, linger
offered no, desktop on (Linux). `--yes` mirrors `install.sh -y`.

### Changes required

- New CLI command file `src/cli/setup.rs` wired through `src/cli/command.rs` and
  `src/cli/root.rs`; add parser tests under `src/cli/tests/`.
- New handler `src/app/commands/setup.rs` (or a `setup/` module if it grows)
  that orchestrates existing services rather than duplicating them:
  - reuse the init flow behind `src/app/commands/init.rs`
  - reuse daemon install behind `src/app/commands/daemon_install.rs`
    (`daemon install --start` equivalent), keeping its per-OS dispatch
  - call into the same completions/manpage generation used by
    `src/app/commands/completions.rs` and `src/app/commands/manpage.rs`
- Add reusable environment-detection helpers (OS, arch, shell, PATH membership)
  under `src/support/` so both `--check` and the guided flow share them. Avoid
  re-shelling `uname`; prefer existing detection already used by
  upgrade/reattach (`sysinfo`, target triple detection in upgrade).
- Per-OS behavior:
  - Linux: systemd user daemon, optional `loginctl enable-linger`, XDG desktop
    launcher + hicolor icons, completion/man dirs under `$XDG_DATA_HOME`.
  - macOS: launchd user agent daemon, `networksetup` proxy already exists; skip
    XDG desktop/icons and linger.
  - BSD: rc.d daemon (root-gated); skip XDG desktop/icons and linger.
- Idempotency: each step detects existing state and reports "already done"
  instead of failing (init already present, daemon already installed,
  completions already current, dir already in PATH).
- `--check` mode emits a structured report of each step's status using the
  shared output helpers in `src/app/commands/output.rs` and existing `--format`
  conventions; no mutations.
- Record setup as a diagnostic event through `src/app/events.rs` (best-effort,
  must not fail the operation) so a setup run appears in `xrat logs`.
- Shrink `install.sh` to: detect platform, download + verify + place binary,
  then `exec xrat setup` (passing through `-y`/`--no-desktop` etc.). Keep the
  ASCII art / branding in the script or move it behind a `--banner` flag on
  setup — decide in review.
- Docs: add `docs/src/02-cli/setup.md`, link it from the CLI index, and update
  `docs/src/01-getting-started/installation.md` to describe `xrat setup` as the
  supported way to (re)run setup after any install method.

### Non-goals

- Downloading/placing the binary itself. That stays in `install.sh` and package
  managers; `setup` assumes the binary is already on disk.
- Installing `xray`/`sing-box`. `setup` only detects them and prints install
  hints, same as today.
- Self-upgrade. That remains `xrat upgrade`.

### Verification

- CLI parser tests in `src/cli/tests/` for every flag and for mutually relevant
  combinations (`--check` rejects mutating flags, `--linger` implies daemon).
- Handler unit tests for idempotency: running setup twice produces the same end
  state and the second run reports steps as already complete.
- `--check` output tests for both human and JSON formats.
- Per-OS dispatch covered by tests where the existing daemon/proxy helpers are
  already testable; gate OS-specific assertions on `cfg`.
- Manual matrix:
  - Linux: fresh `cargo install`, then `xrat setup`; confirm init, daemon,
    optional linger, completions, desktop launcher, PATH hint.
  - macOS: confirm launchd daemon + completions, no XDG desktop/linger.
  - Re-run `xrat setup`; confirm idempotent output.
  - `xrat setup --check --format json` on a half-configured install.

### Open decisions

- Whether `install.sh` should still install completions/man pages from the
  archive, or always delegate to `xrat setup` to generate them from the binary
  (avoids version skew but requires the binary to run during install).
- Whether to keep the playful install banner in shell or move it into the binary
  behind a flag.
- Whether `--check` belongs on `setup` or as a separate `xrat doctor` command;
  if a broader diagnostics command is likely, scope `--check` minimally now.
- How much of `select_desktop_terminal` / `write_desktop_launcher` logic should
  move into Rust versus staying packaging-side.
