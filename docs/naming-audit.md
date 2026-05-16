# Naming & Structure Improvement Audit

> Reviewed: 2026-05-16 · Scope: `src/` directory · 25 findings across 3 severity levels

---

## HIGH -- Confusing or Misleading

### 1. Duplicate `node_from_record` Function

**Locations:**
- `src/app/runtime_service/helpers.rs`
- `src/app/commands/test/model/mod.rs`

**Problem:** Identical function defined in two separate modules. Both convert `ConfigRecord` to `Node`. Will drift over time.

**Fix:** Extract to a single shared location (`src/db/model/configs.rs` or `src/model/conversion.rs`).

---

### 2. Duplicate `looks_like_url` Function

**Locations:**
- `src/app/input/source.rs`
- `src/app/import.rs`

**Problem:** Same helper defined twice in the same crate.

**Fix:** Move to `src/support/url.rs` or have `import.rs` call `input::source::looks_like_url`.

---

### 3. Duplicate `now_epoch_seconds` / `now_string` (4+ copies)

**Locations:**
- `src/app/daemon/supervisor/mod.rs`
- `src/app/daemon/supervisor/handlers/health.rs`
- `src/app/daemon/supervisor/handlers/runtime/runtime_lifecycle/mod.rs`
- `src/app/runtime_service/connect/replace_flow/candidate.rs`
- `src/app/runtime_service/helpers.rs` (`now_string`)

**Problem:** Same timestamp utility reimplemented at least 5 times.

**Fix:** Create `src/support/time.rs` with `now_epoch_seconds() -> u64` and `now_string() -> String`.

---

### 4. Confusing `src/config/xray/` vs `src/xray/config/`

**Locations:**
- `src/config/xray/` -- Xray JSON config *parsing* (deserialization into Rust types)
- `src/xray/config/` -- Xray runtime config *generation* (building JSON from `Node`)

**Problem:** The naming hierarchy is inverted. A reader expects `config/xray/` to be a subset of `config/` (general config parsing), but it is actually Xray-specific deserialization. Meanwhile `xray/config/` is about generation, not configuration.

**Fix:**
- `src/config/xray/` → `src/xray/parsing/` (parses Xray JSON into Rust types)
- `src/xray/config/` → keep as-is or rename to `src/xray/config_gen/` (generates runtime configs)

---

### 5. Three "paths" Modules

**Locations:**
- `src/app/path.rs` -- `AppPaths` (root dir, database path, config file path)
- `src/app/config/paths.rs` -- `PathSettings` (TOML config struct for path overrides)
- `src/app/runtime/paths.rs` -- `RuntimePaths` (resolved runtime paths including binary paths)

**Problem:** Singular vs plural, overlapping naming. Searching for "where are paths resolved?" is ambiguous.

**Fix:**
- `src/app/path.rs` → `src/app/app_paths.rs`
- `src/app/config/paths.rs` → `src/app/config/path_settings.rs`
- `src/app/runtime/paths.rs` → keep as-is

---

## MEDIUM -- Inconsistent Patterns

### 6. Singular vs Plural Module Naming

**Examples:**
- `src/model/node.rs` (singular) vs `src/db/model/configs.rs` (plural)
- `src/db/model/import.rs` (singular) vs `src/db/model/runtime_sessions.rs` (plural)
- `src/cli/test/` (singular) vs `src/cli/tests/` (plural) -- siblings with confusingly similar names

**Fix:** Adopt convention: model files are singular (`node.rs`, `protocol.rs`), repository/collection files are plural (`configs.rs`, `subscriptions.rs`). Rename `src/cli/test/` → `src/cli/test_cmd/` to distinguish from `tests/`.

---

### 7. `test.rs` / `test/` File-Directory Collision

**Location:** `src/app/commands/test.rs` + `src/app/commands/test/`

**Problem:** `test.rs` is not just a module declaration -- it also contains `run_rotation_bulk_tests`, mixing concerns.

**Fix:** Move `run_rotation_bulk_tests` into `test/bulk/rotation.rs`. Make `test.rs` a pure module declaration file.

---

### 8. `database.rs` / `database/` Collision

**Location:** `src/db/database.rs` + `src/db/database/`

**Problem:** `database/imports.rs` is poorly named -- it defines the `Database` struct, it doesn't "import" anything.

**Fix:** Rename `src/db/database/imports.rs` → `src/db/database/types.rs`.

---

### 9. `generate_parse_config` Confusing Name

**Location:** `src/singbox/config/mod.rs`

**Problem:** Sounds like it "generates a parse config" but actually generates a sing-box config for parsing/diagnostics purposes.

**Fix:** Rename to `generate_singbox_probe_config`.

---

### 10. Vague `helpers.rs`

**Location:** `src/app/runtime_service/helpers.rs`

**Contents:** `now_string()`, `node_from_record()`, `connect_host_for_bind_host()`

**Problem:** "helpers" tells the reader nothing. These are a time utility, a record-to-node converter, and a bind-to-connect host mapper.

**Fix:** Split into specific modules or move to shared locations (see items 1, 3).

---

### 11. Redundant `process_impl.rs`

**Location:** `src/xray/process/process_impl.rs`

**Problem:** `_impl` suffix is redundant inside the `process` module.

**Fix:** Rename to `src/xray/process/spawn.rs` or `src/xray/process/handle.rs`.

---

### 12. Misleading `facade/`

**Location:** `src/db/repository/facade/`

**Problem:** It just re-exports functions from sibling modules. Not a facade pattern.

**Fix:** Rename to `src/db/repository/api.rs` or inline re-exports into `src/db/repository/mod.rs` and eliminate the directory.

---

### 13. `config/support.rs` vs `support/` Name Collision

**Locations:**
- `src/config/support.rs` -- URL query parsing, JSON field extraction
- `src/support/` -- top-level module with `decode.rs` and `geoip.rs`

**Problem:** Two different "support" modules at different hierarchy levels.

**Fix:** Rename `src/config/support.rs` → `src/config/parsing_helpers.rs` or `src/config/url_helpers.rs`.

---

### 14. "Runtime" Overloaded 5 Ways

**Modules:**
| Module | What it does |
|---|---|
| `src/app/runtime.rs` | `AppContext` and `RuntimePaths` (application bootstrap) |
| `src/app/runtime_service.rs` | Managed proxy lifecycle (connect/disconnect/replace) |
| `src/app/config/runtime/` | TOML settings for local inbounds (socks, http, rotation) |
| `src/xray/runtime/` | Xray process spawning and signal handling |
| `src/singbox/runtime.rs` | sing-box process spawning |

**Problem:** The word "runtime" means 5 different things.

**Fix:**
- `src/app/runtime.rs` → `src/app/context.rs`
- `src/app/runtime_service.rs` → keep as-is (clearest usage)
- `src/app/config/runtime/` → `src/app/config/proxy.rs` or `src/app/config/inbounds/`
- `src/xray/runtime/` → `src/xray/process_mgmt/`
- `src/singbox/runtime.rs` → `src/singbox/process_mgmt.rs`

---

### 15. Unclear `server/` vs `supervisor/` Boundary

**Locations:**
- `src/app/daemon/server/` -- IPC transport (Unix socket, client, request/response types)
- `src/app/daemon/supervisor/` -- Event loop, state machine, handlers

**Problem:** "server" does not clearly convey "IPC transport layer."

**Fix:** Rename `src/app/daemon/server/` → `src/app/daemon/ipc/`.

---

## LOW -- Could Be Clearer

### 16. Generic `model/` in Test Command

**Location:** `src/app/commands/test/model/`

**Contents:** `TestOutputParts`, `TestStatus`, `node_from_record`

**Fix:** Rename to `src/app/commands/test/output_types.rs` or `src/app/commands/test/result_model.rs`.

---

### 17. Two "model" Modules

**Locations:**
- `src/model/` -- Domain types (`Node`, `Protocol`, `NodeDedupKey`)
- `src/db/model/` -- Database record types (`ConfigRecord`, `RuntimeSessionRecord`)

**Fix:** Rename `src/db/model/` → `src/db/record/` or `src/db/dto/`.

---

### 18. Awkward `defaults_impl.rs`

**Locations:**
- `src/app/config/runtime/defaults_impl.rs`
- `src/app/config/testing/defaults_impl.rs`

**Problem:** `_impl` suffix used to avoid name collision with a `defaults` module.

**Fix:** Rename to `default_values.rs`.

---

### 19. Verb-Named Directory `generate/`

**Location:** `src/xray/config/generate/`

**Fix:** Rename to `src/xray/config/generator/` or `src/xray/config/builder/`.

---

### 20. Deeply Nested Test Directories

**Pattern:** `src/cli/tests/cases/runtime_parse_cases/proxy_cases.rs`

**Problem:** `cases/` intermediate directory and `_cases` suffix repeated at every level adds noise.

**Fix:** Flatten to `src/cli/tests/runtime_parse/proxy.rs`.

---

### 21. `RuntimeStatusLabel` Inconsistent Variants

**Location:** `src/app/runtime_service/types.rs`

```rust
pub enum RuntimeStatusLabel {
    Degraded,
    Persisted(RuntimeSessionStatus),  // wraps another enum
    Stale,
    StaleReconciled,
    Stopped,
}
```

**Problem:** `Persisted` is a container, not a label. Other variants are simple.

**Fix:** Rename enum to `RuntimeSessionDisplay` or `RuntimeSummary`.

---

### 22. Vague `runners/`

**Location:** `src/app/commands/test/bulk/runners/`

**Fix:** Rename to `src/app/commands/test/bulk/executor/` (note: sibling `execution/` exists at `test/` level, so `bulk_executor/` or `bulk_ops/` avoids collision).

---

### 23. Vague `Serve` Variant

**Location:** `src/cli/daemon.rs` -- `DaemonAction::Serve`

**Problem:** "Serve" is vague -- serve what? It's the internal IPC server loop.

**Fix:** Rename to `DaemonAction::RunServer` or `DaemonAction::IpcListen`.

---

### 24. Unclear `entrypoints/`

**Location:** `src/app/commands/test/entrypoints/`

**Contents:** `run`, `run_ping_loop`, `print_latest_run_summary`

**Fix:** Rename to `src/app/commands/test/handlers/`.

---

### 25. Deep Nesting `connect/replace_flow/`

**Location:** `src/app/runtime_service/connect/replace_flow/`

**Problem:** Replace is conceptually a sibling of connect (both are session lifecycle actions), not a sub-operation.

**Fix:** Flatten to `src/app/runtime_service/replace.rs` as a sibling of `connect.rs`.

---

## Summary Table

| # | Severity | Issue | Suggested Fix |
|---|---|---|---|
| 1 | HIGH | Duplicate `node_from_record` | Extract to shared module |
| 2 | HIGH | Duplicate `looks_like_url` | Extract to `support/url.rs` |
| 3 | HIGH | Duplicate `now_epoch_seconds`/`now_string` (5 copies) | Extract to `support/time.rs` |
| 4 | HIGH | `config/xray/` vs `xray/config/` confusion | Rename to `xray/parsing/` |
| 5 | HIGH | Three "paths" modules | `app_paths.rs`, `path_settings.rs` |
| 6 | MEDIUM | Singular/plural inconsistency | Adopt convention |
| 7 | MEDIUM | `test.rs` / `test/` collision | Move function to submodule |
| 8 | MEDIUM | `database/imports.rs` misnamed | Rename to `types.rs` |
| 9 | MEDIUM | `generate_parse_config` confusing | Rename to `generate_singbox_probe_config` |
| 10 | MEDIUM | Vague `helpers.rs` | Split into specific modules |
| 11 | MEDIUM | Redundant `process_impl.rs` | Rename to `spawn.rs` |
| 12 | MEDIUM | Misleading `facade/` | Rename to `api.rs` or inline |
| 13 | MEDIUM | `config/support.rs` vs `support/` | Rename to `parsing_helpers.rs` |
| 14 | MEDIUM | "Runtime" overloaded 5 ways | Disambiguate (see details) |
| 15 | MEDIUM | Unclear `server/` vs `supervisor/` | Rename `server/` to `ipc/` |
| 16 | LOW | Generic `model/` in test | Rename to `output_types.rs` |
| 17 | LOW | Two "model" modules | Rename `db/model/` to `db/record/` |
| 18 | LOW | Awkward `defaults_impl.rs` | Rename to `default_values.rs` |
| 19 | LOW | Verb-named `generate/` dir | Rename to `generator/` |
| 20 | LOW | Deeply nested test dirs | Flatten structure |
| 21 | LOW | `RuntimeStatusLabel` inconsistent | Rename to `RuntimeSessionDisplay` |
| 22 | LOW | Vague `runners/` | Rename to `bulk_executor/` |
| 23 | LOW | Vague `Serve` variant | Rename to `RunServer` |
| 24 | LOW | Unclear `entrypoints/` | Rename to `handlers/` |
| 25 | LOW | Deep `connect/replace_flow/` | Flatten to sibling |

## Additional Findings (Post-Scan)

### 26. MEDIUM -- `src/app/path.rs` / `src/app/path/` File-Directory Collision

**Location:** `src/app/path.rs` + `src/app/path/tests.rs`

**Problem:** Same pattern as item 7 (`test.rs` / `test/`). The file is not a pure module declaration.

**Fix:** Rename `src/app/path.rs` → `src/app/app_paths.rs` and move tests into `src/app/app_paths/tests.rs`.

---

### 27. LOW -- `src/db/repository/configs/import_list/` Unclear Purpose

**Location:** `src/db/repository/configs/import_list/`

**Contents:** `import.rs`, `query.rs`

**Problem:** "import_list" is ambiguous -- is it importing a list, or listing imports?

**Fix:** Rename to `src/db/repository/configs/import_ops/`.

---

### 28. LOW -- `src/app/daemon/server/` Submodule Inconsistency

**Location:** `src/app/daemon/server/`

**Submodules:** `bridge/`, `client/`, `responses/`, `serve/`

**Problem:** Mixed naming conventions -- `bridge` (role), `client` (actor), `responses` (data), `serve` (verb).

**Fix:** After renaming to `ipc/` (item 15), standardize submodules: `transport/`, `client/`, `responses/`, `handler/`.

---

### 29. LOW -- `src/tester/` Module Purpose Unclear

**Location:** `src/tester/`

**Contents:** `download/`, `icmp/`, `real_delay/`, `tcp/`, `upload/`

**Problem:** "tester" is generic. This module handles node performance measurement (delay, throughput, connectivity).

**Fix:** Rename to `src/prober/` or `src/measurer/`.

---

### 30. LOW -- Deeply Nested `connect_status_cases/connect_case/`

**Location:** `src/app/runtime_service/tests/connect_status_cases/connect_case/`

**Problem:** Redundant `_cases` suffix repeated, with singular/plural collision (`connect_status_cases` contains `connect_case`).

**Fix:** Flatten to `src/app/runtime_service/tests/connect_status/lifecycle.rs` and `src/app/runtime_service/tests/connect_status/rejection.rs`.

---

## Progress Tracker

| # | Severity | Status | Notes |
|---|---|---|---|
| 1 | HIGH | [x] done | Duplicate `node_from_record` |
| 2 | HIGH | [x] done | Duplicate `looks_like_url` |
| 3 | HIGH | [x] done | Duplicate `now_epoch_seconds`/`now_string` |
| 4 | HIGH | [x] done | `config/xray/` vs `xray/config/` |
| 5 | HIGH | [x] done | Three "paths" modules |
| 6 | MEDIUM | [ ] pending | Singular/plural inconsistency |
| 7 | MEDIUM | [ ] pending | `test.rs` / `test/` collision |
| 8 | MEDIUM | [ ] pending | `database/imports.rs` misnamed |
| 9 | MEDIUM | [ ] pending | `generate_parse_config` confusing |
| 10 | MEDIUM | [ ] pending | Vague `helpers.rs` |
| 11 | MEDIUM | [ ] pending | Redundant `process_impl.rs` |
| 12 | MEDIUM | [ ] pending | Misleading `facade/` |
| 13 | MEDIUM | [ ] pending | `config/support.rs` vs `support/` |
| 14 | MEDIUM | [x] done | "Runtime" overloaded 5 ways |
| 15 | MEDIUM | [ ] pending | Unclear `server/` vs `supervisor/` |
| 16 | LOW | [ ] pending | Generic `model/` in test |
| 17 | LOW | [ ] pending | Two "model" modules |
| 18 | LOW | [ ] pending | Awkward `defaults_impl.rs` |
| 19 | LOW | [ ] pending | Verb-named `generate/` dir |
| 20 | LOW | [ ] pending | Deeply nested test dirs |
| 21 | LOW | [ ] pending | `RuntimeStatusLabel` inconsistent |
| 22 | LOW | [ ] pending | Vague `runners/` |
| 23 | LOW | [ ] pending | Vague `Serve` variant |
| 24 | LOW | [ ] pending | Unclear `entrypoints/` |
| 25 | LOW | [ ] pending | Deep `connect/replace_flow/` |
| 26 | MEDIUM | [x] done | `path.rs` / `path/` collision |
| 27 | LOW | [ ] pending | `import_list/` unclear |
| 28 | LOW | [ ] pending | `server/` submodule inconsistency |
| 29 | LOW | [ ] pending | `tester/` module purpose unclear |
| 30 | LOW | [ ] pending | Deep `connect_status_cases/connect_case/` |

**Summary:** 7/30 complete · 0 HIGH · 9 MEDIUM · 14 LOW

---

## Implementation Checklist

Use this checklist when working through the audit. Complete items in priority order (HIGH → MEDIUM → LOW).

### Phase 1: Deduplication (Items 1-3)

- [x] 1.1 Create `src/support/time.rs` with `now_epoch_seconds()` and `now_string()`
- [x] 1.2 Update all 5 call sites to use `support::time::*`
- [x] 1.3 Remove duplicate definitions
- [x] 1.4 Create `src/support/url.rs` with `looks_like_url()`
- [x] 1.5 Update `src/app/input/source.rs` and `src/app/import.rs` imports
- [x] 1.6 Remove duplicate definition
- [x] 1.7 Decide shared location for `node_from_record` (`src/db/model/configs.rs` or `src/model/conversion.rs`)
- [x] 1.8 Move function, update both call sites
- [x] 1.9 Remove duplicate definition
- [x] 1.10 Run `cargo build && cargo test -q`

### Phase 2: Disambiguate "Runtime" (Item 14)

- [x] 2.1 Rename `src/app/runtime.rs` → `src/app/context.rs`
- [x] 2.2 Update all imports referencing `app::runtime`
- [x] 2.3 Rename `src/app/config/runtime/` → `src/app/config/proxy/`
- [x] 2.4 Update all imports referencing `app::config::runtime`
- [x] 2.5 Rename `src/xray/runtime/` → `src/xray/process_mgmt/`
- [x] 2.6 Update all imports referencing `xray::runtime`
- [x] 2.7 Rename `src/singbox/runtime.rs` → `src/singbox/process_mgmt.rs`
- [x] 2.8 Update all imports referencing `singbox::runtime`
- [x] 2.9 Run `cargo build && cargo test -q`

### Phase 3: Config vs Xray Boundary (Item 4)

- [x] 3.1 Rename `src/config/xray/` → `src/xray/parsing/`
- [x] 3.2 Update all imports referencing `config::xray`
- [x] 3.3 Run `cargo build && cargo test -q`

### Phase 4: Paths Modules (Items 5, 26)

- [x] 4.1 Rename `src/app/path.rs` → `src/app/app_paths.rs`
- [x] 4.2 Rename `src/app/path/` → `src/app/app_paths/`
- [x] 4.3 Rename `src/app/config/paths.rs` → `src/app/config/path_settings.rs`
- [x] 4.4 Update all imports
- [x] 4.5 Run `cargo build && cargo test -q`

### Phase 5: Daemon IPC Boundary (Items 15, 28)

- [ ] 5.1 Rename `src/app/daemon/server/` → `src/app/daemon/ipc/`
- [ ] 5.2 Rename `ipc/bridge/` → `ipc/transport/`
- [ ] 5.3 Rename `ipc/serve/` → `ipc/handler/`
- [ ] 5.4 Update all imports
- [ ] 5.5 Run `cargo build && cargo test -q`

### Phase 6: Remaining Medium Items (6-13, 22, 24, 26)

- [ ] 6.1 Adopt singular/plural convention (item 6)
- [ ] 6.2 Resolve `test.rs` / `test/` collision (item 7)
- [ ] 6.3 Rename `database/imports.rs` → `types.rs` (item 8)
- [ ] 6.4 Rename `generate_parse_config` (item 9)
- [ ] 6.5 Split `runtime_service/helpers.rs` (item 10)
- [ ] 6.6 Rename `process_impl.rs` → `spawn.rs` (item 11)
- [ ] 6.7 Inline or rename `facade/` (item 12)
- [ ] 6.8 Rename `config/support.rs` (item 13)
- [ ] 6.9 Rename `runners/` → `bulk_executor/` (item 22)
- [ ] 6.10 Rename `entrypoints/` → `handlers/` (item 24)
- [ ] 6.11 Run `cargo build && cargo test -q`

### Phase 7: Low Items (16-21, 23, 25, 27, 29, 30)

- [ ] 7.1 Rename test `model/` (item 16)
- [ ] 7.2 Rename `db/model/` → `db/record/` (item 17)
- [ ] 7.3 Rename `defaults_impl.rs` (item 18)
- [ ] 7.4 Rename `generate/` → `generator/` (item 19)
- [ ] 7.5 Flatten test directories (item 20)
- [ ] 7.6 Rename `RuntimeStatusLabel` (item 21)
- [ ] 7.7 Rename `Serve` variant (item 23)
- [ ] 7.8 Flatten `connect/replace_flow/` (item 25)
- [ ] 7.9 Rename `import_list/` (item 27)
- [ ] 7.10 Rename `tester/` (item 29)
- [ ] 7.11 Flatten `connect_status_cases/` (item 30)
- [ ] 7.12 Run `cargo build && cargo test -q`

---

## Recommended Priority Order

1. **Deduplication first** (items 1-3): Eliminate copy-paste code to prevent drift
2. **Disambiguate "runtime"** (item 14): Highest cognitive load reduction
3. **Clarify config vs xray boundary** (item 4): Prevents future confusion
4. **Rename paths modules** (item 5): Quick win, high discoverability improvement
5. **Rename daemon server → ipc** (item 15): Clear architectural boundary
6. **Remaining medium items**: Incremental improvements
7. **Low items**: Polish when touching related code
