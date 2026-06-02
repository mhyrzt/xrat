# CI Improvements Backlog

The current CI is working and should be treated as the baseline, not as broken
infrastructure. This page only tracks optional improvements for speed, coverage,
release confidence, and maintainability.

## Current Baseline

### Main CI

`.github/workflows/ci.yml` runs on pushes to `main`/`master` and on pull
requests.

Current checks:

- Check out the repository.
- Install stable Rust with `clippy` and `rustfmt`.
- Run `cargo fmt --all --check`.
- Run `cargo clippy --all-targets --all-features`.
- Run `cargo test -q`.

This is a good minimal gate for day-to-day Rust changes.

### Docs

`.github/workflows/docs.yml` runs when docs or the docs workflow change, and can
also run manually.

Current behavior:

- Installs mdBook and mdbook-mermaid.
- Restores the custom Mermaid assets after installing Mermaid support.
- Builds the book.
- Deploys to GitHub Pages.

### Release

`.github/workflows/release.yml` runs on `v*` tags.

Current behavior:

- Builds release binaries on Linux, macOS, and Windows.
- Uploads per-platform artifacts.
- Publishes a GitHub Release with the built artifacts.

## 1. CI Speed and Caching

Add caching once dependency/build time becomes noticeable.

Scope:

- Cache Cargo registry, Cargo git sources, and target artifacts.
- Prefer a maintained Rust cache action or a small explicit cache setup.
- Keep cache keys tied to `Cargo.lock`, Rust toolchain, OS, and build profile.
- Avoid making cache misses or stale caches difficult to debug.

Definition of done:

- CI remains functionally identical when the cache is cold.
- Warm runs are measurably faster.
- Cache behavior is documented in the workflow comments or backlog follow-up.

## 2. Cross-Platform Test Matrix

Expand test coverage beyond Ubuntu when platform behavior becomes important.

Scope:

- Add a matrix for Linux, macOS, and Windows test jobs.
- Keep the existing Ubuntu job as the fastest/default signal.
- Decide whether macOS/Windows should run on every PR or only on release/main
  branch events to control CI cost and queue time.
- Pay special attention to path handling, config directory resolution, process
  management, and CLI parsing behavior.

Definition of done:

- Platform-specific failures are caught before release.
- Matrix jobs are named clearly enough to identify the failing platform.
- The workflow stays readable and does not obscure the simple Rust check flow.

## 3. Locked and Reproducible Checks

Tighten reproducibility without changing normal developer workflow.

Scope:

- Consider `cargo test --locked` in CI so lockfile drift is caught early.
- Consider `cargo clippy --locked --all-targets --all-features`.
- Keep release builds using `cargo build --release --locked`.
- Add a clear failure mode when `Cargo.lock` needs to be updated.

Definition of done:

- Pull requests fail when dependency changes are not reflected in `Cargo.lock`.
- The release workflow and normal CI agree on dependency resolution.

## 4. Clippy Strictness

Decide whether CI should deny Clippy warnings.

Scope:

- Evaluate adding `-- -D warnings` to the Clippy step.
- If enabled, apply it after the codebase is already clean so the change is not
  mixed with unrelated cleanup.
- Keep any allowed lints explicit in code or project config.

Definition of done:

- CI prevents new Clippy warnings once the policy is adopted.
- The policy is documented so contributors know warnings are treated as build
  failures.

## 5. Docs Validation in CI

The docs workflow already builds mdBook for docs changes. Consider whether the
main CI or release workflow should also validate docs.

Scope:

- Add docs build validation to release tags so published binaries are not
  shipped with broken docs.
- Optionally add docs validation to pull requests that touch both code and docs.
- Avoid running heavy docs setup on every code-only PR unless there is a clear
  benefit.

Definition of done:

- Release tags fail if docs cannot be built.
- Docs validation remains path-aware or otherwise cheap enough for routine work.

## 6. Release Workflow Gates

Keep the release workflow focused on packaging, but make sure it cannot publish
artifacts that skipped essential checks.

Scope:

- Require the same Rust checks as main CI before publishing a release.
- Decide whether to call reusable workflows or duplicate a small set of release
  checks inline.
- Add artifact checksum generation.
- Add smoke checks against built binaries, such as `xrat --version` and
  `xrat --help`.
- Add generated shell completion and man page validation once those artifacts
  exist.

Definition of done:

- Tagged releases fail before publishing when basic checks fail.
- Release artifacts are built, smoke-tested, checksummed, and then published.

## 7. Security and Dependency Maintenance

Add maintenance jobs only when they are useful and not noisy.

Scope:

- Consider `cargo audit` or another advisory scanner.
- Consider Dependabot for GitHub Actions and Cargo dependencies.
- Decide how advisory failures should be handled for transitive dependencies
  that cannot be fixed immediately.
- Keep permissions minimal for every workflow.

Definition of done:

- Dependency/security checks produce actionable failures.
- Workflow permissions remain scoped to the jobs that need them.

## Suggested Order

1. Add release smoke tests and checksums.
2. Add Cargo caching if CI time becomes painful.
3. Add `--locked` to normal CI checks.
4. Add docs validation to release tags.
5. Expand to a platform test matrix when cross-platform support becomes a
   release goal.
6. Add Clippy `-D warnings` once the lint policy is ready.
