---
id: TASK-1
title: 'High, P1: Preserve and faithfully regenerate unknown config parameters'
status: Done
assignee: []
created_date: '2026-07-05 14:43'
updated_date: '2026-08-15 00:23'
labels:
  - legacy-import
  - bugfix
dependencies: []
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Legacy path: `docs/backlog/bugfix/config-roundtrip-fidelity.md`

# High, P1: Preserve and faithfully regenerate unknown config parameters

### Status

Planned

### Motivation

A REALITY VLESS subscription failed every probe with `Empty "realitySettings"`
and later `empty "password"`. Two root causes, both the same class of bug:

1. The runtime generator (`src/xray/config/`) is a hand-coded subset. It set
   `security = "reality"` but emitted no `realitySettings`, and it had no field
   for the REALITY public key, so Xray rejected the outbound.
2. `node_from_record` (`src/db/record/configs.rs`) reconstructed nodes from
   typed DB columns only and hardcoded `extensions: None`, dropping `pbk`,
   `sid`, `fp`, `flow`, `mode`, and `alpn`. Probes then built REALITY settings
   with an empty public key.

Both were fixed for REALITY specifically (re-parse `raw_config` to recover
extensions; add `RealitySettings`). But the underlying design — an allowlist of
known parameters plus a hand-written generator — guarantees the next unusual
config breaks the same way, usually silently. This task makes config handling
**round-trip faithful**: anything a link carries is preserved and either
regenerated correctly or rejected with a clear reason, never silently dropped.

### Current behavior (the gaps)

Parsers keep only a fixed set of fields and discard the rest:

- `src/config/protocols/vless.rs` collects a fixed allowlist:
  `["fp", "alpn", "mode", "flow", "pbk", "sid", "spx"]`. Everything else
  (`headerType`, `serviceName`, `seed`, `host` variants, future keys) is lost.
- `src/config/protocols/trojan.rs` and `vmess.rs` set `extensions: None`. Trojan
  over REALITY/xhttp would break exactly like the VLESS case did. VMess drops
  `aid`/`scy`/`type`/`fp`/`alpn`.
- gRPC `serviceName` is read from the `path` query, not the `serviceName`/`spx`
  parameter that real gRPC links use, so gRPC service names are often wrong.

The runtime generator only covers a subset of transports and security options:

- `src/xray/config/stream.rs` generates: `tcp` (http header from `path`), `ws`
  (path + `Host`), `grpc` (serviceName from `path`), `xhttp` (host, path, mode),
  `tls` (serverName, fingerprint, alpn), and now `reality`.
- Not generated at all: `kcp`/mKCP, `httpupgrade`, QUIC/hysteria. A node with
  one of these networks gets a `streamSettings` with `network` set but no
  matching settings object, which Xray may reject or mis-handle.
- Missing per-transport fields: ws custom headers and early-data, grpc
  `multiMode`/`idleTimeout`, xhttp `extra`/headers, tls `allowInsecure`, mKCP
  `seed`/`header`, sockopt entirely.

When a parameter is unsupported, nothing tells the user it was ignored — the
config simply fails or, worse, connects with the wrong settings.

### xray-core stream settings reference (target coverage)

`streamSettings` fields and the settings objects we should be able to round-trip
(client side):

- `network`: `raw`/`tcp`, `kcp`, `ws`, `grpc`, `xhttp` (splithttp, covers HTTP/2
  and HTTP/3), `httpupgrade`. The standalone `http`/h2 transport and
  domainsocket are removed; `quic` is REALITY-incompatible.
- `security`: `none`, `tls`, `reality`.
- `tlsSettings`: `serverName`, `alpn`, `fingerprint`, `allowInsecure`.
- `realitySettings` (client): `serverName`, `fingerprint`, `publicKey`,
  `shortId`, `spiderX`, `show`.
- `rawSettings`/`tcpSettings`: `header` (http obfuscation).
- `kcpSettings`: `mtu`, `tti`, `uplinkCapacity`, `downlinkCapacity`,
  `congestion`, `readBufferSize`, `writeBufferSize`, `header`, `seed`.
- `wsSettings`: `path`, `host`, `headers`, `heartbeatPeriod`.
- `grpcSettings`: `serviceName`, `multiMode`, `idleTimeout`.
- `xhttpSettings`: `path`, `host`, `mode`, `extra`.
- `httpupgradeSettings`: `path`, `host`, `headers`.
- `sockopt`: `tcpFastOpen`, `tproxy`, `domainStrategy`, `interface`,
  `dialerProxy`, `happyEyeballs`, etc.

The parsing layer in `src/xray/parsing/transports/` already models most of these
objects; the gap is the link parsers and the link→runtime generator, not the
JSON model.

### Changes required

Capture, don't allowlist:

- Make link parsers collect **all** non-structural query parameters (or VMess
  JSON keys) into `extensions`, instead of a fixed allowlist. Reserve typed
  columns for the structural fields (address, port, uuid, network, tls, sni,
  host, path), and route everything else through `extensions`.
- Apply this uniformly to vless, vmess, trojan, ss, http, socks5 so every
  protocol that can use REALITY/flow/custom transports preserves its parameters.

Make extension recovery a contract, not a one-off:

- Treat `raw_config` as the source of truth for parameters that have no typed
  column. `node_from_record` already re-parses it; document that invariant and
  cover it with tests so it is not regressed. Alternatively, decide to persist
  extensions in the DB (see open decisions).

Generate faithfully or fail loud:

- Extend `src/xray/config/stream.rs` to cover mKCP and httpupgrade, plus the
  missing per-transport fields (ws headers/early-data, grpc multiMode/idle,
  xhttp extra, tls allowInsecure).
- Fix gRPC to read `serviceName` from the correct parameter.
- When the generator encounters a `network` or `security` it does not support,
  return a descriptive error naming the unsupported value (e.g.
  `unsupported network "kcp" for runtime generation`) instead of emitting a
  partial `streamSettings`.
- Surface ignored parameters: when a known-but-unhandled extension is present,
  record a diagnostic (warn-level log and/or a field on the test result) so the
  user learns the config was downgraded rather than silently mis-built.

### Verification

- Round-trip tests per protocol: parse a representative link (REALITY xhttp,
  REALITY+vision tcp, ws+tls, grpc, mKCP, httpupgrade, trojan+reality), generate
  the runtime config, and assert the generated `streamSettings`/outbound matches
  the link's parameters.
- A test that an unsupported network/security produces a clear error, not a
  silently incomplete config.
- A test that `node_from_record` recovers extensions for trojan and vmess, not
  just vless.
- Manual: `xrat test` against a mixed subscription (REALITY, vision, ws, grpc)
  and confirm each connects or fails with an actionable message.

### Open decisions

- Persist `extensions` as a DB column (e.g. JSON) versus always re-deriving from
  `raw_config`. Re-deriving keeps the schema small but couples runtime behavior
  to link re-parsing; a column makes the data explicit and queryable but needs a
  migration and keeps two sources in sync.
- Whether ignored-parameter diagnostics belong in logs only, or also as a field
  on the test/probe result surfaced in `xrat test` output and the TUI.
- How strict to be: reject any config with an unhandled parameter, or generate a
  best-effort config plus a warning. Leaning toward fail-loud for parameters
  that change the wire protocol (security, flow, network) and warn-only for
  cosmetic ones.

### Related

- REALITY support and extension recovery landed in v0.9.0
  (`src/xray/config/stream.rs`, `src/db/record/configs.rs`).
- Trojan password is read without percent-decoding
  (`src/config/protocols/trojan.rs`); encoded passwords would auth-fail. Fix
  alongside the parser rework.

## Parser reference

Legacy path: `docs/backlog/bugfix/xray-knife-parser-reference.md`

# Reference: xray-knife link parser field coverage

### Status

Reference (no work item; supports `config-roundtrip-fidelity.md`)

### What this is

[`xray-knife`](https://github.com/lilendian0x00/xray-knife) (Go, dual xray-core

- sing-box) solves the same link → runtime-config problem xrat does. This
  records exactly which parameters its xray parsers read, as a target for xrat's
  parser rework. Source: `pkg/core/xray/{vless,trojan,vmess}.go` on `master`,
  inspected 2026-06-20.

### How it differs from xrat

xray-knife reads every transport/security parameter explicitly into typed struct
fields, for **each** protocol — including Trojan, which xrat parses with
`extensions: None`. It also validates `sni`/`host`, applies sensible defaults,
and treats a missing `type` as `tcp` and a missing Trojan `security` as `tls`.

### VLESS (`pkg/core/xray/vless.go`)

Reads from the query string:

- `security` (`tls` / `reality` / none), `type` (network), `alpn`, `fp`
  (fingerprint), `sni`, `host`, `path`
- `flow`, `pbk` (reality public key), `sid` (reality short id), `spx` (spiderX)
- `headerType` (e.g. `http` for TCP obfuscation), `serviceName` (gRPC), `mode`
  (gRPC gun/multi or xhttp mode), `quicSecurity`

Defaults: `path` → `/` for `headerType=http`/`ws`/`h2`/`xhttp`; `type` → `tcp`
when empty.

### Trojan (`pkg/core/xray/trojan.go`)

Reads the same broad set: `flow`, `security`, `alpn`, `fp`, `type`, `sni`,
`host`, `path`, `headerType`, `serviceName`, `mode`, `pbk`, `sid`, `spx`,
`quicSecurity`.

Defaults: `type` → `tcp`; `security` → `tls` when empty; `fp` → `chrome` when
`security` is `tls`/`reality`; `flow` → `none` for gRPC.

### VMess (`pkg/core/xray/vmess.go`)

Base64 JSON or legacy query form. Reads `security`/`scy` (cipher) from userinfo,
`path`, `host`, `sni`, `alterId`, and maps legacy `obfs=websocket` → ws and
`obfsParam` → host. Normalizes `type`/network and defaults `path` → `/` for
http-family transports.

### Gap table vs xrat

| Parameter         | xray-knife (vless/trojan) | xrat today                         |
| ----------------- | ------------------------- | ---------------------------------- |
| `flow`            | both                      | vless only (added v0.9.0)          |
| `pbk`/`sid`/`spx` | both                      | vless only                         |
| `fp`/`alpn`       | both                      | vless only                         |
| `headerType`      | both                      | dropped                            |
| `serviceName`     | both                      | dropped (grpc uses `path` instead) |
| `mode`            | both                      | vless only                         |
| `quicSecurity`    | both                      | dropped                            |
| Trojan params     | full set                  | `extensions: None` (all dropped)   |
| VMess `scy`/`aid` | read                      | dropped                            |

### Takeaways for the rework

- Read parameters per protocol, not via a vless-only allowlist; Trojan and VMess
  need the same coverage (Trojan+REALITY is otherwise broken like the v0.9.0
  bug).
- Read `serviceName` for gRPC instead of overloading `path`.
- Adopt the same defaulting rules (`type`→`tcp`, Trojan `security`→`tls`,
  `fp`→`chrome` for tls/reality) so generated configs match common clients.
- Validate `sni`/`host` on parse, as xray-knife does, to fail early on garbage
  entries (the junk `info` / `2087` addresses we saw would be caught here).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 All non-structural link and VMess JSON parameters are preserved with native values and repeated URL keys
- [x] #2 Persisted extensions survive database round trips and legacy rows recover them from raw links without changing identity
- [x] #3 Generated Xray configs faithfully cover raw, xhttp, mKCP, gRPC, WebSocket, and HTTPUpgrade or reject unsupported wire-affecting settings clearly
- [x] #4 Tests cover parser, persistence, deduplication, generation, percent-decoding, and fail-loud diagnostics
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Replace string-only extension storage with deterministic JSON values and capture all non-structural parameters. 2. Persist extensions with a backward-compatible migration and include them in canonical deduplication while retaining raw-link fallback for legacy rows. 3. Generate supported current Xray transports and security fields, normalize aliases, and reject unsupported or incomplete settings. 4. Add regression tests and update config documentation.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented JSON-valued extension capture for all link parsers, v2 deduplication, SQLite/PostgreSQL extension persistence with legacy raw-link backfill, current Xray transport/security generation, and fail-loud validation for unsupported wire settings. Updated protocol, generation, import, and database docs. Validation: just fmt ci passed (737 tests; clippy with -D warnings; Rust/Markdown/SQL formatting).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Preserved non-structural config parameters end to end and made Xray runtime generation faithful for supported current transports, with explicit errors for unsupported configurations. Added migration 0022 for both backends, checksum coverage, legacy backfill, and regression tests.
<!-- SECTION:FINAL_SUMMARY:END -->
