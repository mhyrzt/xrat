# Phase 8: Native GeoLite2 MMDB Downloader

## Goal

Replace `scripts/download_geolite2_mmdb.sh` with a first-class `xrat geoip`
command family that downloads and manages GeoLite2 MMDB files natively from
Rust, while preserving the existing layout (`$XRAT_PATH/geoip/...`) and the
`GeoLite2-Country|City|ASN` edition set the script already supports.

By the end of this phase, XRAT should be able to:

- download one or more GeoLite2 editions into the resolved geoip directory
- report the resolved geoip directory and the on-disk presence state per edition
- refresh the locally cached files on demand
- integrate the same path resolution as the rest of the app (`XRAT_PATH` or
  `~/.config/xrat`)
- keep the existing default `geoip/GeoLite2-{Country,City,ASN}.mmdb` paths that
  `[testing.geoip]` already reads from
- keep the shell script working for one release as a thin wrapper, then be
  removed

## Why This Phase Exists

XRAT already supports GeoIP enrichment of test results through `[testing.geoip]`
and `src/support/geoip.rs`, but the only way to obtain GeoLite2 files today is a
hand-maintained bash script. The script:

- is a separate build/run artifact the user must discover and remember
- only handles single-edition downloads
- does not report which editions are already present or where they live
- cannot be invoked from the TUI, the HTTP API, or the daemon supervisor
- has no automated tests because it lives outside the Rust crate

Pulling the downloader into the CLI gives the rest of the project a stable
service to build on:

- TUI can offer a "Download GeoIP" action that calls the same service
- documentation only has to teach one command
- the resolver for the geoip directory becomes a single Rust function that the
  rest of the app can reuse (e.g. the future `[geo]` asset work, real-MMDB
  tests, and a possible auto-update scheduler)
- CI can run the downloader via `cargo run` instead of `bash scripts/...`

The script is kept as a thin wrapper for one release to keep the migration
non-breaking. It then becomes redundant and can be deleted.

## Current Starting Point

Relevant building blocks already present in the codebase:

- `XRAT_PATH` resolution lives in `src/app/app_paths.rs` and is consumed by
  `RuntimePaths` in `src/app/context/paths.rs`
- `[testing.geoip]` settings already canonicalize the default MMDB paths as
  `geoip/GeoLite2-Country.mmdb`, `geoip/GeoLite2-City.mmdb`, and
  `geoip/GeoLite2-ASN.mmdb` (see `src/app/config/defaults.rs:60-62`)
- GeoIP lookups use `maxminddb` in `src/support/geoip.rs` and only need valid
  MMDB files at the configured paths
- `reqwest` is already a dependency with `rustls`, `blocking`, and `socks`
  features enabled
- `indicatif` is already a dependency and used for progress bars in the
  `xrat test` bulk path (`src/app/commands/test/bulk/bulk_executor/progress.rs`)
- `tempfile` is already a dependency and is the standard way to do crash-safe
  downloads in this codebase
- `clap` subcommand pattern is already established by `xrat daemon`,
  `xrat proxy`, and others

The current script (`scripts/download_geolite2_mmdb.sh`):

- hard-codes the P3TERX mirror at
  `https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/<EDITION>.mmdb`
- validates the edition against `GeoLite2-Country|City|ASN`
- downloads into a temp dir, verifies the file is non-empty, and atomically
  installs it at `${XRAT_PATH}/geoip/<EDITION>.mmdb` with mode `0644`
- exposes `XRAT_PATH` and `GEOIP_EDITION` as overrides

The native implementation should keep that contract for the first slice and add
the rest on top of it.

## Scope Boundary

Phase 8 should cover:

- `xrat geoip download [flags]` to fetch one or more editions
- `xrat geoip update` as a convenience alias that downloads every supported
  edition
- `xrat geoip path` to print the resolved geoip directory
- `xrat geoip status` to show which editions are present, missing, or stale
- a reusable Rust function that resolves the geoip directory from `RuntimePaths`
- atomic, crash-safe file replacement using a temp file in the same directory
  plus `rename`
- indicatif progress bar during download (matching the style used in `xrat test`
  bulk runs)
- edition parsing that accepts both `GeoLite2-Country` (full) and the short
  `country|city|asn` form, with the full form as the canonical display name
- a `Justfile` recipe and CI smoke that exercise the new command
- a deprecation note in the shell script that points at `xrat geoip download`,
  followed by removal in a later release

Phase 8 should not yet cover:

- SHA256 verification (P3TERX mirror does not publish hashes; can be added later
  if a verifying mirror is chosen)
- auto-update scheduling from the daemon (lives behind a future
  `geo.auto_update` config, see `PHASE_3p5` and `[geo]` schema notes)
- Xray `geosite.dat` / Xray-style `geoip.dat` downloads (different asset family,
  different source; out of scope for this phase)
- a TUI panel/button (Phase 6 work, can call into the same service when the TUI
  background task work reaches it)
- an HTTP API endpoint (the existing `/health` family is the wrong shape for a
  one-shot download; can be added later if needed)

## CLI Entry

Add a new top-level `geoip` subcommand with a small `Subcommand` enum:

```bash
xrat geoip download [flags]
xrat geoip update
xrat geoip path
xrat geoip status
```

### `xrat geoip download`

| Flag               | Description                                                                                                                                                                                                  |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `--edition <name>` | Edition to download. Repeatable. Accepts `GeoLite2-Country`, `GeoLite2-City`, `GeoLite2-ASN`, or the short forms `country`, `city`, `asn`. Defaults to `GeoLite2-Country` when omitted, matching the script. |
| `--all`            | Download every supported edition (`Country`, `City`, `ASN`). Overrides `--edition`.                                                                                                                          |
| `--output <dir>`   | Override the target directory. Defaults to `$XRAT_PATH/geoip`, or `~/.config/xrat/geoip` when `XRAT_PATH` is unset.                                                                                          |
| `--force`          | Re-download even if the file already exists. Default is to skip with a clear message.                                                                                                                        |
| `--url <template>` | Override the URL template. The string `{edition}` is replaced with the canonical edition name. Defaults to `https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/{edition}.mmdb`.                 |
| `--timeout <secs>` | HTTP request timeout in seconds. Defaults to `60`.                                                                                                                                                           |
| `--quiet`          | Suppress progress bar output. Default prints a per-edition bar to stderr.                                                                                                                                    |

Examples:

```bash
xrat geoip download
xrat geoip download --edition GeoLite2-City
xrat geoip download --all
xrat geoip download --edition GeoLite2-City --output ./testdata/xrat/geoip --force
xrat geoip download --url https://mirror.example.com/{edition}.mmdb --edition GeoLite2-ASN
```

### `xrat geoip update`

Convenience subcommand equivalent to `xrat geoip download --all --force`. Same
flags as `download` minus `--edition`/`--all`. Intended for the "Just refresh
everything" workflow and for CI smoke tests.

### `xrat geoip path`

Prints the resolved geoip directory and exits. Example:

```bash
$ xrat geoip path
/home/mahyar/.config/xrat/geoip
```

Does not require database access, so it can be used to script around the
location. Honors the same `--output` override as `download` only for symmetry;
the default is always the resolved app path.

### `xrat geoip status`

Reports each supported edition's on-disk presence and size, plus the resolved
geoip directory. Example:

```bash
$ xrat geoip status
geoip dir: /home/mahyar/.config/xrat/geoip
GeoLite2-Country.mmdb  present   72.4 MiB
GeoLite2-City.mmdb     missing   -
GeoLite2-ASN.mmdb      present   18.1 MiB
```

Sizes are reported in human-readable units. No network access. Exits with
non-zero status when at least one edition is missing if `--strict` is passed,
otherwise zero.

## Module Layout

Recommended initial layout, matching the existing
`src/app/commands/{daemon,proxy}` pattern:

```text
src/cli/
  geoip.rs              # GeoIpArgs + GeoIpAction + edition parsing tests
src/app/commands/
  geoip/
    mod.rs              # dispatch + shared helpers
    download.rs         # HTTP download + atomic write + progress bar
    update.rs           # thin wrapper around download --all --force
    path.rs             # print resolved geoip dir
    status.rs           # per-edition presence/size report
    edition.rs          # Edition enum + parse/display + tests
```

Shared resolver:

```text
src/app/
  paths/geoip.rs        # resolve_geoip_dir(runtime_paths) -> PathBuf
                        # geoip_path(runtime_paths, edition) -> PathBuf
```

These helpers should be reusable by:

- the TUI when it adds a "Download GeoIP" background action
- a future daemon auto-update scheduler
- the existing `[testing.geoip]` path validation in
  `src/app/commands/test/settings/resolve.rs` (no behavior change there, but the
  helpers become the canonical source of truth for the resolved paths)

## Behavior

### Path resolution

The geoip directory must come from the same source of truth as the rest of the
app. The resolver must:

- use `$XRAT_PATH/geoip` when `XRAT_PATH` is set
- otherwise use `~/.config/xrat/geoip` (matching `app_paths::resolve`)
- honor `--output` only for the duration of one command invocation
- create the directory if it does not exist (matching the script's `mkdir -p`)

### Edition parsing

- canonical names: `GeoLite2-Country`, `GeoLite2-City`, `GeoLite2-ASN`
- short aliases: `country`, `city`, `asn` (case-insensitive)
- anything else: `AppError::InvalidArgument` with a list of valid values
- display always uses the canonical form so the file on disk matches what
  `[testing.geoip]` already expects

### Download

- HTTP GET against `<url template>` with `{edition}` replaced
- async via `reqwest` (no `blocking` feature use; `tokio` is already wired)
- follow redirects (GitHub releases redirect to S3; default `reqwest` behavior
  already follows up to 10)
- write to a `tempfile::NamedTempFile::new_in(&geoip_dir)` so the rename is
  atomic on the same filesystem
- on success: `persist()` to `<geoip_dir>/<Edition>.mmdb` with mode `0644`
- on HTTP error or non-2xx: surface the status code and the URL in the error
  message, drop the temp file, do not leave a half-written file on disk
- on empty response (0 bytes): distinct error matching the script's "download
  failed or returned empty file" message
- when the destination file already exists and `--force` is not set: skip with
  `skipped: GeoLite2-Country.mmdb (already present, use --force to redownload)`.
  Exit code is still zero for the skipped edition but the final summary
  distinguishes downloaded / skipped / failed counts

### Progress bar

- one `indicatif::ProgressBar` per edition, written to stderr
- template mirrors the test bulk progress style:
  `{spinner:.green} {edition} [{bar:32.cyan/blue}] {bytes}/{total_bytes} {msg}`
- hidden when `--quiet` is set or stderr is not a TTY
- finishes with `done` on success or `failed: <reason>` on error

### `update` semantics

Equivalent to:

```bash
xrat geoip download --all --force
```

with the same other flags inherited. The exit code is zero only when every
edition downloads successfully; otherwise the highest non-zero exit code is
returned, and a summary of failed editions is printed.

### `status` semantics

- walks the resolved geoip directory
- for each canonical edition name, reports present/missing and size
- does not validate MMDB format; that is `[testing.geoip]`'s job
- does not require network access

## Error Handling

Reuse `AppError` and add a focused variant rather than scattering strings:

```rust
#[error("geoip download failed for {edition} from {url}: {reason}")]
GeoipDownload { edition: String, url: String, reason: String },
```

This keeps the existing `From<reqwest::Error>` for callers that do not need the
per-edition context. `GeoipDownload` is the surfaced form when the per-edition
context matters.

Specific failure modes:

- unsupported edition: `AppError::InvalidArgument` with the list of valid values
- geoip directory not creatable: `AppError::Io` from the underlying error
- HTTP non-2xx, network error, empty body: `AppError::GeoipDownload` with the
  edition, the URL, and the reason
- atomic rename failure: `AppError::Io`; the temp file is cleaned up via
  `NamedTempFile`'s `Drop`

## Implementation Slices

### P8.1 Resolver and CLI Scaffold

Goal: route the new command into the CLI without breaking the script.

Tasks:

- [ ] Add `src/app/paths/geoip.rs` with `resolve_geoip_dir` and `geoip_path_for`
      helpers
- [ ] Add `src/cli/geoip.rs` with `GeoIpArgs`, `GeoIpAction`, and edition
      parsing
- [ ] Register `GeoIp(GeoIpArgs)` in `src/cli/command.rs` and re-export
- [ ] Add `src/app/commands/geoip/mod.rs` and `path.rs` that print the resolved
      directory
- [ ] Wire dispatch in `src/app/commands/mod.rs`
- [ ] Add focused tests in `src/cli/tests/` for the CLI parser and the resolver

Acceptance:

- [ ] `xrat geoip path` prints the resolved geoip directory
- [ ] `XRAT_PATH=/tmp/foo xrat geoip path` prints `/tmp/foo/geoip`
- [ ] `xrat geoip --help` lists the subcommands

### P8.2 Single-Edition Download

Goal: replace the script for the most common path (one edition, default
location).

Tasks:

- [ ] Add `src/app/commands/geoip/download.rs` with async download + atomic
      write + indicatif progress
- [ ] Validate the destination directory is creatable before the HTTP call
- [ ] Add `--edition`, `--output`, `--force`, `--url`, `--timeout`, `--quiet`
      flags
- [ ] Add `AppError::GeoipDownload` variant
- [ ] Add unit tests: - edition parser accepts both forms - URL builder
      substitutes `{edition}` correctly - destination path builder joins
      `geoip_dir` and the canonical edition filename - empty-body detection does
      not require a real HTTP call (factor the check)

Acceptance:

- [ ] `xrat geoip download` produces `$XRAT_PATH/geoip/GeoLite2-Country.mmdb`
- [ ] `xrat geoip download --edition GeoLite2-City` produces the City file
- [ ] re-running without `--force` skips and reports `skipped`
- [ ] `xrat geoip download --url https://invalid.example/{edition}.mmdb` exits
      non-zero and prints the URL in the error

### P8.3 Multi-Edition and `update`

Goal: make "refresh all the geoip files" a one-liner.

Tasks:

- [ ] Add `--all` flag to `download`
- [ ] Allow `--edition` to be repeated
- [ ] Add `src/app/commands/geoip/update.rs` that delegates to
      `download --all --force`
- [ ] Run editions concurrently with `JoinSet` so multiple downloads overlap
      without exceeding a small bounded fan-out
- [ ] Print a final summary: `downloaded=N skipped=M failed=K`
- [ ] Add tests for the multi-edition dispatch and the summary reducer

Acceptance:

- [ ] `xrat geoip update` downloads Country, City, and ASN in one run
- [ ] summary line matches the per-edition outcomes
- [ ] no temp files remain in the geoip directory after a successful run
- [ ] one edition failing does not abort the others; the summary reports the
      failure and the exit code is non-zero

### P8.4 `status` and `path` Polish

Goal: complete the read-only subcommands so users can introspect without
downloading.

Tasks:

- [ ] Add `src/app/commands/geoip/status.rs` with the per-edition table
- [ ] Add `--strict` flag to `status` for CI use
- [ ] Human-readable size formatting (KiB/MiB/GiB) using a small helper
- [ ] Tests for the formatter and the missing/present logic

Acceptance:

- [ ] `xrat geoip status` lists every supported edition with present/missing and
      size
- [ ] `xrat geoip status --strict` exits non-zero when any edition is missing

### P8.5 Test Surface and CI Wiring

Goal: cover the new code with tests and wire the command into CI.

Tasks:

- [ ] Unit tests for edition parsing, URL building, atomic write, skip behavior
- [ ] Integration-style test using a local `httpmock`/`wiremock` server for the
      download path that does not hit the network
- [ ] Real-network smoke gated behind `XRAT_GEOIP_DOWNLOAD_LIVE=1`, skipped by
      default
- [ ] Update `Justfile`: - `geoip-download` recipe points at
      `cargo run -- geoip download` - `geoip-download-testdata` and
      `geoip-download-testdata-all` recipes are preserved (they remain useful
      for keeping `testdata/xrat/geoip/` populated)
- [ ] Add a `geoip-path` and `geoip-status` recipe for convenience

Acceptance:

- [ ] `cargo test -q geoip::` passes
- [ ] full `cargo test -q` still passes
- [ ] `just geoip-download` and `just geoip-update` work end-to-end

### P8.6 Documentation and Script Deprecation

Goal: teach the new command and retire the script.

Tasks:

- [ ] Add `docs/src/02-cli/geoip.md` with the subcommand reference
- [ ] Add `docs/src/03-features/geoip.md` describing the asset layout and the
      downloader (or extend the existing `testing.md` GeoIP section with a link
      to the new command)
- [ ] Add `docs/src/08-backlog/01-plan/README.md` roadmap entry for Phase 8
- [ ] Update `README.md` GeoIP section to lead with `xrat geoip download` and
      note that the shell script is deprecated
- [ ] Add a deprecation comment at the top of
      `scripts/download_geolite2_mmdb.sh` that prints a warning and shells out
      to `xrat geoip update` when present, otherwise keeps the existing behavior
- [ ] In a follow-up release: delete the script and the warning wrapper

Acceptance:

- [ ] running the old `./scripts/download_geolite2_mmdb.sh` prints a deprecation
      notice but still succeeds
- [ ] mdbook build succeeds
- [ ] `cargo fmt` and `cargo test -q` pass

## Documentation

Update when the phase starts:

- `docs/src/SUMMARY.md` to include the new `geoip` CLI page and feature page
- `docs/src/02-cli/geoip.md` for the command reference
- `docs/src/03-features/geoip.md` (new) or extend
  `docs/src/03-features/testing.md` for the GeoIP enrichment story
- `README.md` GeoIP section to use the new command
- `Justfile` to keep the existing recipes working through the new command

## Open Questions

1. **Command shape** — `xrat geoip <subcommand>` (this plan) versus a flat
   `xrat geoip-download` and `xrat geoip-update`. The subcommand shape matches
   the existing `xrat daemon` and `xrat proxy` families and gives room for
   `status`, `path`, and a future `verify`/`update-schedule` without re-plumbing
   clap. Worth confirming before the slices land.

2. **Default edition** — keep the script's `GeoLite2-Country` default for
   `download` (matches what users expect today), and treat `--all` as the "I
   want everything" path. Alternative: make `download` default to `--all` so a
   fresh user gets a working geoip directory with one command.

3. **TUI integration** — should the TUI Phase 6 work add a "Download GeoIP"
   action in the Sources view (or a new "GeoIP" view), or should it stay
   CLI-only and only surface the path/status in the Diagnostics view? If added,
   it should call the same `geoip::download` service via a background task so
   the UI stays responsive.

4. **Daemon auto-update** — Phase 3.5 already mentions `geo.auto_update` and
   `geo.update_interval_hours` in `src/app/config/defaults.rs`. This phase
   should not implement the scheduler, but the slice plan should keep room for a
   future `geoip::run_scheduled_update` entry point that the daemon can call.

5. **Mirror handling** — the `--url` flag is enough for an escape hatch, but
   should the command also accept a list of mirrors with fallback? Probably out
   of scope for v1; revisit if a real failure mode is reported.

6. **SHA256 verification** — P3TERX does not publish checksums for the mirror
   assets. If the project ever switches to MaxMind's official downloads (which
   require an account/license key) or another verifying mirror, add a
   `--sha256 <hex>` flag and a parallel test that rejects mismatched downloads.

7. **Concurrency limit** — `--all` already enables parallel downloads via
   `JoinSet`. Should there be a `--jobs <N>` flag, or is "all three editions in
   parallel" small enough that no flag is needed? Default to unbounded for v1
   and add the flag only if a user reports a problem.

8. **Script removal timing** — the script is kept as a thin wrapper for one
   release. Confirm the removal release now so the deprecation comment can name
   it.

## Completion Criteria

Phase 8 can be considered complete when:

1. `xrat geoip download` produces the same files the old script produced in the
   same locations, with the same default edition.
2. `xrat geoip update` downloads all three editions in one run and reports a
   per-edition summary.
3. `xrat geoip path` and `xrat geoip status` work without network access.
4. The shell script still works, prints a deprecation notice, and delegates to
   the new command when `xrat` is on `PATH`.
5. `cargo fmt` and `cargo test -q` pass, including the new `geoip::` tests.
6. `Justfile` recipes `geoip-download`, `geoip-download-testdata`, and
   `geoip-download-testdata-all` still work through the new command.
7. mdbook builds and the new pages are linked from `SUMMARY.md`.
8. The resolver helper is reusable: a unit test exercises both `$XRAT_PATH`
   override and the `~/.config/xrat` default.

## Out of Scope

- Xray `geosite.dat` / `geoip.dat` asset family (different sources, may be
  covered by a separate `xrat geoasset` command later)
- MaxMind's licensed direct downloads (account/license key handling is a much
  larger surface and lives outside the open mirror workflow)
- Auto-update scheduling from the daemon (already deferred to a future phase,
  see `PHASE_3p5`)
- TUI panel/button for the downloader (Phase 6 work; can call the same service)
- HTTP API endpoint (no obvious use case for a one-shot download)
- Windows-specific path handling beyond what `RuntimePaths` already does
