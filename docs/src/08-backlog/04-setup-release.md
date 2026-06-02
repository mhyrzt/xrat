# Setup and Release Backlog

This note collects setup, installation, and release-readiness work that should
make xrat easier to adopt outside a development checkout. It is intentionally
planning-only; no implementation is implied by this page.

## 1. Daemon Install and Uninstall Commands

Add first-class commands for installing and removing the xrat daemon service:

```bash
xrat daemon install
xrat daemon uninstall
```

The current daemon command surface covers `start`, `status`, and `stop`, while
the systemd documentation asks users to manually write service files. The goal
is to move that repetitive setup into xrat so a user can install the daemon with
one command and still inspect exactly what was created.

Scope:

- Add CLI parsing for `daemon install` and `daemon uninstall`.
- Prefer user services on Linux (`systemctl --user`) so normal users do not need
  root for everyday installation.
- Generate a service file that starts the daemon in the same runtime root the
  CLI uses, including `XRAT_PATH` when a non-default root is selected.
- Run or print the required `systemctl --user daemon-reload`, `enable`,
  `disable`, and `stop` steps.
- Keep `daemon uninstall` conservative: stop/disable the service and remove the
  generated service file, but do not delete user config, database, logs, or
  imported configs.
- Detect unsupported platforms and return a clear message instead of silently
  doing nothing.

Open decisions:

- Whether `daemon install` should start the daemon immediately or only enable it
  for future logins.
- Whether a `--dry-run` or `--print-service` flag should be added first so users
  can audit the generated service unit.
- Whether macOS launchd support belongs in the first implementation or a later
  platform-specific follow-up.

Definition of done:

- `xrat daemon install` creates the service unit, reloads the manager, and
  enables the service when requested.
- `xrat daemon uninstall` removes only daemon service integration and preserves
  application state.
- CLI parser tests cover both commands.
- Documentation replaces the manual systemd setup path with the command-based
  flow while keeping the generated unit shape visible for troubleshooting.

## 2. Init Command

Add an explicit initialization command:

```bash
xrat init
```

Today xrat resolves its default app root to `$HOME/.config/xrat/` and can create
a minimal `config.toml` as part of normal command bootstrap. `xrat init` should
make that setup deliberate, visible, and safe to run before importing any
configs.

Expected behavior:

- Create the app directory at `$HOME/.config/xrat/` by default.
- Respect the same root override mechanism used elsewhere, including `XRAT_PATH`
  and global `--config`/`--database` flags where applicable.
- Create `config.toml` from a useful default template instead of only an empty
  header.
- Create or migrate the default SQLite database at
  `$HOME/.config/xrat/db.sqlite`.
- Create predictable runtime/log/cache directories if they are part of the
  active layout.
- Be idempotent: running `xrat init` twice should not overwrite an existing
  config or database unless a future explicit force flag is added.
- Print a short summary of paths created and paths already present.

Open decisions:

- Whether `xrat init` should validate required runtime binaries (`xray`,
  `v2ray`, `sing-box`) or leave that to a separate diagnostics command.
- Whether `xrat init --postgres` should exist, or whether PostgreSQL setup
  should stay config-driven.
- Whether a `--force` mode is worth adding, and if so whether it should only
  refresh missing config keys rather than overwrite the full file.

Definition of done:

- A new user can run `xrat init`, then `xrat import ...`, without relying on
  implicit bootstrap side effects.
- Existing users can run `xrat init` safely without losing local settings.
- Tests cover default paths, override paths, existing-file preservation, and
  database migration creation.

## 3. Prerequisites and Installation Documentation

Update user-facing documentation so the first setup path is clear before the
quickstart workflow begins.

Needed documentation:

- Required OS assumptions and current support level for Linux, macOS, and
  Windows.
- Required runtime binaries for the selected engine:
  - `xray` for Xray runtime and real-delay tests.
  - `v2ray` when V2Ray is selected.
  - `sing-box` when sing-box parsing or runtime support is used.
- Rust/Cargo installation path for users building from source.
- Binary installation path once release artifacts are published.
- SQLite default behavior and when PostgreSQL is optional.
- Network permissions and firewall notes for local SOCKS/HTTP inbound ports.
- Where xrat stores state by default:
  - `$HOME/.config/xrat/config.toml`
  - `$HOME/.config/xrat/db.sqlite`
  - `$HOME/.config/xrat/runtime/`
  - logs and GeoIP/MMDB paths when enabled.
- How `XRAT_PATH`, `--config`, and `--database` change those defaults.

Candidate doc changes:

- Add a dedicated installation page under `docs/src/01-getting-started/`.
- Link it before Quickstart in the getting-started flow.
- Update Quickstart so it begins with `xrat init` once that command exists.
- Update the systemd page after `daemon install` exists so manual service files
  become a troubleshooting/reference section rather than the primary setup path.

Definition of done:

- A new user can identify prerequisites, install xrat, initialize state, and run
  the quickstart without reading architecture docs.
- Documentation clearly separates development setup, installed binary setup,
  daemon service setup, and optional PostgreSQL setup.

## 4. Release Planning

Prepare the repository for reliable tagged releases and installable packages.
There is already a release workflow that builds Linux, macOS, and Windows
binaries on `v*` tags. The next step is to make releases more complete and
repeatable.

### Shell Completion

Add generated shell completion support, starting with bash and leaving room for
the other common shells.

Scope:

- Generate bash completion from the Clap command tree so completions stay in
  sync with CLI changes.
- Decide whether generation happens through a hidden/internal command such as
  `xrat completions bash`, a build script, or a packaging-only helper.
- Include completions in release artifacts and Linux packages.
- Document user installation paths for bash completion, especially:
  - per-user installs under `~/.local/share/bash-completion/completions/`
  - system package installs under the distro-specific completion directory.
- Expand later to zsh, fish, and PowerShell once the generation path is stable.

Definition of done:

- A packaged install can provide bash completion without hand-written scripts.
- Completion output updates automatically when commands, flags, or subcommands
  change.
- Documentation includes both packaged and manual completion installation.

### Man Page

Add a generated man page so installed packages have standard terminal
documentation.

Scope:

- Generate an `xrat(1)` man page from the Clap command tree or another
  single-source CLI description.
- Include top-level usage, global flags, subcommands, config path behavior,
  default state paths, and links to full docs.
- Package the man page into the correct platform/package locations where
  applicable.
- Keep detailed feature documentation in mdBook; the man page should be a
  concise installed reference, not a duplicate manual.

Definition of done:

- `man xrat` works after installing a package that supports man pages.
- The generated man page is checked in or produced during release in a
  repeatable way.
- CI catches stale or broken man page generation before release.

Workflow changes to plan:

- Run `cargo fmt`, `cargo test`, and `cargo clippy` before release packaging.
- Build with locked dependencies and fail if the lockfile is stale.
- Produce checksums for every binary/package artifact.
- Use clearer target names such as `x86_64-unknown-linux-gnu`,
  `x86_64-apple-darwin`, and `x86_64-pc-windows-msvc`.
- Consider adding musl Linux builds if static binaries are desired.
- Publish generated release notes from a changelog or GitHub release draft
  instead of a generic one-line note.
- Add docs build validation so release tags do not ship broken documentation.

Essential packages and artifacts to evaluate:

- Raw compressed binaries for Linux, macOS, and Windows.
- Debian package (`.deb`) for common Linux installs.
- RPM package for Fedora/RHEL-style systems.
- Arch package metadata or an AUR-ready package recipe.
- Homebrew tap formula for macOS and Linuxbrew users.
- Bash completion, with later support for zsh, fish, and PowerShell.
- `xrat(1)` man page or equivalent generated CLI reference artifact.
- Optional systemd user unit template for package installs, even if
  `xrat daemon install` remains the preferred path.

Release readiness checklist:

- Version source is clear and matches tag names.
- Release artifacts include license and README where packaging format expects
  them.
- Install docs reference the same artifact names emitted by CI.
- Package installs do not create or overwrite user state; users still run
  `xrat init`.
- Smoke tests verify `xrat --version`, `xrat init`, and basic CLI help from a
  packaged artifact.
