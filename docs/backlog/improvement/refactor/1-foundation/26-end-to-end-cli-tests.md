# Add End-To-End CLI Test Harness

## Finding

### [Priority: Medium] Add black-box end-to-end CLI tests

**Files involved:**

- `tests/` (new top-level integration test directory)
- `src/cli/tests/` (existing inline parser tests — stay as-is)
- `Cargo.toml` (`[dev-dependencies]`: `assert_cmd`, `predicates`, `tempfile`)

**Problem:** There is no top-level `tests/` directory. All ~88 test modules are
inline `#[cfg(test)]` units close to the code they validate. That is correct for
parser/normalization/repository units, but it leaves the actual CLI wiring —
argument parsing → command dispatch → context build → repository → output —
untested as a whole. A regression in how `main.rs`/`src/cli` routes a command to
its handler, or in exit codes and stdout format, is not caught by any inline
test.

**Why this change is needed:** The architecture goal is thin adapters over shared
use-cases. As use-cases get extracted (`01`–`05`) and adapters get rewired, the
risk shifts from "is the logic correct" (covered by units) to "is the command
wired to the right use-case and rendering the right output". Black-box tests pin
that contract. They also give a safety net for the larger refactors in this
backlog: run the same CLI flow before and after and assert identical output.

**How to implement it:** Add `tests/` integration tests using `assert_cmd` to
invoke the built binary against a temp `XRAT` home (`tempfile`), asserting exit
codes and stdout/stderr with `predicates`. Cover the core lifecycle end-to-end:
`init` → `import <fixture>` → `list` (table + `--format json`) → `show config` →
`delete config` → `purge`. Reuse the seeding/fixture helpers from
`08-application-factories-test-setup` so unit and e2e tests share setup. Keep
these tests network- and daemon-free; gate anything requiring xray/sing-box
binaries behind a feature or skip.

**Positive effect on the codebase:** Catches CLI-wiring and output-format
regressions that inline unit tests structurally cannot. Gives the use-case
extraction work a behavior-preserving harness. Documents the supported CLI flows
by example.

**Suggested target architecture:** Inline `#[cfg(test)]` modules cover units;
`tests/` covers the binary as a black box over a temp home; both draw fixtures
from the shared test-support builders.

**Risk / migration notes:** Low risk — additive, no production change. Start with
the read-only flows (`init`, `list`, `show`) which need no external binaries,
then add mutation flows. Keep e2e tests fast and deterministic so they can stay in
the default `cargo test` run.
