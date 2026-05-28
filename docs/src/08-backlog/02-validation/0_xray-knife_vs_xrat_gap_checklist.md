# xray-knife vs xrat Gap Checklist (File-by-File)

This checklist compares `xray-knife` (Go, `../xray-knife`) with the current
`xrat` codebase (Rust, this repo), focusing on the QA areas in
`../xray-knife/QA/*.md`.

Status legend:

- **MATCHED**: Behavior exists in both projects (possibly different
  implementation).
- **PARTIAL**: Similar behavior exists but scope/semantics differ.
- **MISSING**: Present in xray-knife but not currently implemented in xrat.
- **DIFFERENT BY DESIGN**: xrat intentionally uses a different architecture.

---

## 1) Parse + Validate Config

| xray-knife file(s)                                             | xrat file(s)                                                                                                                                                                                                                      | Current gap status | Notes / action                                                                                                                  |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------- | ---- | ----------------------------------------------------------------------------- |
| `pkg/core/factory.go`                                          | `src/cli/parse.rs`, `src/app/commands/parse.rs`                                                                                                                                                                                   | **MATCHED**        | xrat parse flow supports `--engine auto                                                                                         | xray | sing-box`; auto routes by scheme (hy2/hysteria2 -> sing-box, others -> xray). |
| `cmd/parse/parse.go`                                           | `src/cli/parse.rs`, `src/app/commands/parse.rs`                                                                                                                                                                                   | **MATCHED**        | xrat now has dedicated `parse` command with positional input, `--file`, and `--stdin` modes for non-persistent diagnostics.     |
| `cmd/parse/parse.go` (`--json` path)                           | `src/app/commands/parse.rs`, `src/xray/config/*`, `src/singbox/config.rs`                                                                                                                                                         | **MATCHED**        | `xrat parse --json` renders runtime preview JSON for both xray and sing-box parse paths.                                        |
| `pkg/core/xray/*.go`, `pkg/core/singbox/*.go` protocol parsers | `src/config/protocols/vless.rs`, `src/config/protocols/vmess.rs`, `src/config/protocols/ss.rs`, `src/config/protocols/trojan.rs`, `src/config/protocols/http.rs`, `src/config/protocols/socks5.rs`, `src/config/protocols/hy2.rs` | **PARTIAL**        | xrat covers xray-family parsers and hy2/hysteria2 for sing-box parse path; broader sing-box protocol matrix remains incomplete. |
| N/A (normalization mostly per parser)                          | `src/config/normalize.rs`                                                                                                                                                                                                         | **MATCHED**        | xrat normalizes runtime-relevant fields (`network`, ws/grpc path defaults, host inference).                                     |

### Checklist tasks for this area

- [x] Add optional `parse` command in xrat for diagnostics (single
      link/file/stdin parity).
- [x] Decide whether to support parse-to-xray-json export (`--json` equivalent).
- [x] Decide if sing-box/hysteria2 is in scope for xrat, then add parser + model
      support.

---

## 2) Storage + Persistence

| xray-knife file(s)                                                        | xrat file(s)                                                                        | Current gap status      | Notes / action                                                                                                                                                        |
| ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cmd/root.go` (default DB path init)                                      | `src/cli/root.rs`, `src/app/config/paths.rs`, `src/app/config/database.rs`          | **PARTIAL**             | Both support DB path resolution; paths/layout conventions differ.                                                                                                     |
| `database/queries.go` (`UpsertSubscriptionConfigs`)                       | `src/db/repository/configs.rs`, `src/db/repository/subscriptions.rs`                | **MATCHED**             | Both do durable storage with upsert semantics. xrat upserts by canonical `dedup_key`; xray-knife upserts by `config_link`.                                            |
| `database/migrations/0001_initial_schema.up.sql`                          | `migrations/sqlite/0001_init.sql`, `migrations/postgres/0001_init.sql`              | **DIFFERENT BY DESIGN** | Schema model differs: xray-knife has `subscription_configs`, `http_test_runs/results`, `cf_scan_results`; xrat has `configs`, `connection_tests`, `runtime_sessions`. |
| `database/queries.go` (`CreateHttpTestRun`, `InsertHttpTestResultsBatch`) | `src/db/repository/connection_tests.rs`                                             | **MATCHED**             | Both persist run-grouped test history (`http_test_runs/results` vs `connection_test_runs/connection_tests`).                                                          |
| `database/queries.go` (`UpsertCfScanResultsBatch`)                        | `src/db/repository/cf_scan_results.rs`, `migrations/*/0011_add_cf_scan_results.sql` | **PARTIAL**             | xrat now has scanner result persistence schema/repository; scanner command/runtime integration still missing.                                                         |

### Checklist tasks for this area

- [x] If run-history parity is required, add `test_runs` table + FK from
      `connection_tests`.
- [x] If scanner parity is required, add `cf_scan_results` schema and
      repository.
- [x] Decide if dedup key should remain canonical-struct (`xrat`) or raw-link
      key (`xray-knife` style).

---

## 3) Connection Testing + Ping

| xray-knife file(s)                                                | xrat file(s)                                                                     | Current gap status | Notes / action                                                                                                                                                      |
| ----------------------------------------------------------------- | -------------------------------------------------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pkg/http/examiner.go` (`ExamineConfig`)                          | `src/app/commands/test.rs`, `src/tester/real_delay.rs`, `src/tester/download.rs` | **PARTIAL**        | Both evaluate config health. xray-knife focuses on HTTP probe metrics (delay/ttfb/connect/http code). xrat includes ICMP+TCP gating plus proxy real-delay/download. |
| `cmd/http/http.go` (batch test, flags)                            | `src/cli/test.rs`, `src/app/commands/test.rs`                                    | **MATCHED**        | Both support batch testing knobs and filtering.                                                                                                                     |
| `cmd/http/http.go` (`--ping` continuous mode)                     | `src/app/commands/test/handlers/ping.rs`                                         | **MATCHED**        | xrat implements continuous HTTP ping loop with Ctrl+C summary stats and persisted `ping_loop` run grouping.                                                         |
| `pkg/http/examiner.go` (`semi-passed`, `timeout`, `broken`, etc.) | `src/app/commands/test.rs` status model                                          | **PARTIAL**        | Status vocabulary differs; xrat uses `passed/failed/skipped` style from staged checks.                                                                              |
| `pkg/http/examiner.go` speed/IP-info enrichment                   | `src/tester/download.rs`                                                         | **PARTIAL**        | xrat supports download Mbps, not full IP trace info fields as primary result model.                                                                                 |
| `cmd/net/icmp.go`, `cmd/net/tcp.go`                               | `src/tester/icmp.rs`, `src/tester/tcp.rs`                                        | **MATCHED**        | Equivalent test primitives exist in xrat.                                                                                                                           |

### Checklist tasks for this area

- [x] Add optional `test --ping` (continuous HTTP probe loop) in xrat.
- [x] Decide whether to add `ttfb`, `connect_ms`, `http_status`, `egress_ip`,
      `location` columns.
- [ ] Align/translate status taxonomy if cross-tool comparability is needed.

---

## 4) Engine Selection Logic (xray vs sing-box)

Planning note:

- Runtime-engine direction items in this section should be tracked alongside
  `docs/plan/PHASE_4.md` and `docs/plan/PHASE_4.5.md` because they directly
  affect managed-runtime architecture and future supervisor/reattach behavior.

| xray-knife file(s)                  | xrat file(s)                                                                           | Current gap status      | Notes / action                                                                                             |
| ----------------------------------- | -------------------------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------------------------------------------------------- |
| `pkg/core/factory.go` (auto select) | `src/app/config/runtime.rs`, `src/app/commands/test.rs` (`resolve_engine_binary_path`) | **DIFFERENT BY DESIGN** | xrat selects runtime binary from config (`xray` or `v2ray`), not protocol-based multi-engine auto-routing. |
| `pkg/core/singbox/*`                | N/A                                                                                    | **MISSING**             | No sing-box runtime integration in xrat.                                                                   |
| `pkg/core/xray/*`                   | `src/xray/*`, `src/config/xray/*`                                                      | **PARTIAL**             | xrat is xray-focused runtime path; architecture differs from xray-knife core abstraction.                  |

### Checklist tasks for this area

- [ ] Confirm whether engine parity includes sing-box support.
- [ ] If yes, add engine abstraction layer (similar to xray-knife `Core`) in
      xrat.
- [ ] Add per-protocol engine compatibility matrix + CLI `auto` mode.

---

## 5) Auto-Rotating Proxy

Planning note:

- Runtime lifecycle, reconnect, and supervision-adjacent items here are upstream
  inputs for `docs/plan/PHASE_4.md` and `docs/plan/PHASE_4.5.md`.
- Full rotation strategy still belongs to later parity phases, but any process
  ownership/reconciliation behavior must stay consistent with Phase 4/4.5
  decisions.

| xray-knife file(s)                                                      | xrat file(s)                                                                            | Current gap status | Notes / action                                                                                                                                                                             |
| ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `pkg/proxy/service.go` (rotation loop, health checks, blacklist, drain) | `src/app/runtime_service.rs`, `src/app/daemon/supervisor/`, `src/app/commands/proxy.rs` | **PARTIAL**        | xrat has daemon-owned rotation scheduler with timer/health/manual triggers, cooldown policy, and make-before-break replacement. Missing: durable rotation events, blacklist/strike policy. |
| `cmd/proxy/proxy.go`                                                    | `src/cli/proxy.rs`, `src/app/commands/proxy.rs`                                         | **PARTIAL**        | `xrat proxy start\|status\|rotate\|stop` exists. Missing: netns/sysproxy/chain features.                                                                                                   |
| `pkg/proxy/netns/*`, `pkg/proxy/sysproxy/*`                             | N/A                                                                                     | **MISSING**        | Namespace/system proxy management features are not present in xrat.                                                                                                                        |
| `pkg/proxy/chain.go` (multi-hop chain)                                  | N/A                                                                                     | **MISSING**        | No outbound chain/multi-hop orchestration in xrat.                                                                                                                                         |

### Checklist tasks for this area

- [x] Define new `proxy` command and config contract in `src/cli/` and
      `src/app/commands/`.
- [x] Add rotation strategy (batch test, score, switch, blacklist, cooldown).
- [ ] Add persistence model for rotation state/events if required.

---

## 6) Powerful IP Scanner

| xray-knife file(s)                                         | xrat file(s)                                                                        | Current gap status | Notes / action                                                                                           |
| ---------------------------------------------------------- | ----------------------------------------------------------------------------------- | ------------------ | -------------------------------------------------------------------------------------------------------- |
| `cmd/cfscanner/cfscanner.go`                               | `src/cli/scan.rs`, `src/app/commands/scan.rs`                                       | **PARTIAL**        | xrat has basic scanner command, but not cfscanner feature depth/parity.                                  |
| `pkg/scanner/scanner.go`                                   | `src/app/commands/scan.rs`                                                          | **PARTIAL**        | Basic sequential TCP probing exists; no CIDR expansion/concurrent worker pool yet.                       |
| `database/queries.go` (`CfScanResult`, upsert/load resume) | `src/db/repository/cf_scan_results.rs`, `migrations/*/0011_add_cf_scan_results.sql` | **PARTIAL**        | Schema/repository parity exists; basic scanner command flow exists, but resume semantics remain missing. |
| `cmd/cfscanner/realityscanner.go`                          | N/A                                                                                 | **MISSING**        | No reality-specific scanner flow in xrat.                                                                |

### Checklist tasks for this area

- [ ] Decide scanner scope: latency-only vs latency+speedtest+proxy-assisted
      scan.
- [x] Add schema (`cf_scan_results`) + repository persistence path.
- [ ] Add command UX (`--subnets`, `--resume`, `--speedtest`, `--config`,
      `--save-db`).
- [x] Add baseline scanner command flow (`--ips`/`--file` + persisted results +
      `--history`).

---

## 7) Deduplication Mechanisms

| xray-knife file(s)                                   | xrat file(s)                                              | Current gap status | Notes / action                                                                                     |
| ---------------------------------------------------- | --------------------------------------------------------- | ------------------ | -------------------------------------------------------------------------------------------------- |
| `cmd/subs/subscription.go` (`RemoveDuplicate`)       | `src/config/mod.rs` (`HashSet` dedup)                     | **MATCHED**        | Both remove duplicate configs in-memory before/while ingesting.                                    |
| `pkg/http/httptester.go` (`DeduplicateLinks`)        | `src/config/mod.rs`, `src/model/node_dedup_key.rs`        | **PARTIAL**        | xray-knife dedups raw strings; xrat dedups canonicalized semantic key fields.                      |
| `database/queries.go` (`ON CONFLICT(config_link)`)   | `src/db/repository/configs.rs` (`ON CONFLICT(dedup_key)`) | **MATCHED**        | Both enforce DB-level uniqueness with upsert.                                                      |
| `cmd/cfscanner/cfscanner.go` (`finalResultsMap[ip]`) | `src/app/commands/scan.rs` (`BTreeSet` IP dedup)          | **PARTIAL**        | Scanner IP-level dedup exists via `BTreeSet` before probing. DB `UNIQUE(ip)` upsert also enforced. |
| N/A                                                  | `migrations/*/0003_canonical_config_dedup_key.sql`        | **xrat strength**  | xrat has explicit dedup key migration/versioning (`v1`) for deterministic uniqueness.              |

### Checklist tasks for this area

- [x] Keep canonical dedup key as source of truth (recommended).
- [ ] If scanner is added, include per-IP final result dedup map + DB unique
      key.

---

## Cross-Cutting CLI/File Map

| xray-knife command file      | xrat nearest file                                                              | Gap                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------------------------- |
| `cmd/parse/parse.go`         | `src/cli/parse.rs`, `src/app/commands/parse.rs`                                | Parse UX parity is present (`parse` with file/stdin/input + `--json`). |
| `cmd/http/http.go`           | `src/cli/test.rs`, `src/app/commands/test.rs`                                  | Mostly present; ping loop implemented. Some metric fields differ.      |
| `cmd/proxy/proxy.go`         | `src/cli/proxy.rs`, `src/app/commands/proxy.rs`                                | `xrat proxy start\|status\|rotate\|stop` exists with daemon rotation.  |
| `cmd/cfscanner/cfscanner.go` | `src/cli/scan.rs`, `src/app/commands/scan.rs`                                  | Baseline scanner exists; advanced cfscanner parity remains missing.    |
| `cmd/subs/*.go`              | `src/cli/import.rs`, `src/app/import.rs`, `src/db/repository/subscriptions.rs` | Conceptually present, different UX/storage model.                      |
| `cmd/net/*.go`               | `src/tester/icmp.rs`, `src/tester/tcp.rs`                                      | Primitive checks present inside test pipeline.                         |

---

## Prioritized Implementation Backlog (if parity is desired)

1. **P0 - Decide product direction**
   - [ ] Confirm whether xrat should remain xray-runtime-focused or pursue
         xray-knife parity (sing-box, scanner, rotating proxy).

2. **P1 - Testing parity uplift**
   - [x] Add continuous ping mode to `test` command.
   - [x] Add optional extended test metrics (`ttfb`, `connect_ms`,
         `http_status`, `egress_ip`, `location`).

3. **P1 - Parse UX parity**
   - [x] Add `parse` command (single/file/stdin) and optional `--json` runtime
         config preview.

4. **P2 - Proxy rotation subsystem**
   - [x] Introduce `proxy` command + rotation scheduler + health/cooldown.
   - [ ] Add durable rotation event history and blacklist/strike policy.
   - [ ] Add netns/sysproxy/chain features if required.

5. **P2 - CF scanner subsystem**
   - [x] Add baseline scanner command with IP dedup and persistence.
   - [ ] Add advanced scanner parity (CIDR expansion, resume, bounded
         concurrency, speedtest, proxy-assisted/reality modes).

6. **P3 - Engine abstraction**
   - [ ] Add engine plugin/trait abstraction if sing-box support is required.

---

## Summary

- xrat already has strong foundations for **parsing, canonical dedup, DB
  persistence, staged testing, managed runtime sessions, and auto-rotating
  proxy**.
- Remaining feature gaps versus xray-knife QA are: **sing-box auto engine
  routing, advanced cfscanner parity, durable rotation events, and
  netns/sysproxy/chain features**.
- Continuous HTTP ping mode and proxy rotation scheduler are now implemented.
- If strict parity is the goal, implement remaining P2/P3 items above in order.
