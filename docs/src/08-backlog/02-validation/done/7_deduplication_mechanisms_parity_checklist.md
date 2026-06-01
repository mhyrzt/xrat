# Deduplication Mechanisms Parity Checklist (xray-knife -> xrat)

This checklist maps gap area **#7 Deduplication Mechanisms** from:

- `docs/validation/0_xray-knife_vs_xrat_gap_checklist.md`
- `../xray-knife/QA/7_deduplication_mechanisms.md`

---

## Scope and target behavior

Parity target for this area:

1. In-memory dedup during config ingestion and processing.
2. DB-level uniqueness constraints for durable dedup.
3. Clear dedup semantics (raw-string vs canonical-structure).
4. Scanner-result dedup expectations when scanner subsystem exists.

Out of scope for this checklist:

- introducing scanner subsystem itself (area #6),
- runtime rotation subsystem details (area #5).

---

## xray-knife reference map

Primary source files in `../xray-knife`:

- `cmd/subs/subscription.go` (`RemoveDuplicate`)
- `pkg/http/httptester.go` (`DeduplicateLinks`)
- `cmd/http/http.go` (pre-test dedup)
- `pkg/proxy/service.go` (rotation-pool dedup)
- `database/queries.go` (`ON CONFLICT(config_link)`)
- `cmd/cfscanner/cfscanner.go` (`finalResultsMap[ip]`)

Behavioral source narrative:

- `../xray-knife/QA/7_deduplication_mechanisms.md`

---

## Current state snapshot (xrat)

- In-memory ingest dedup exists:
  - `src/config/mod.rs` (set-based dedup path)
- Canonical semantic key exists:
  - `src/model/node_dedup_key.rs`
  - migration: `migrations/*/0003_canonical_config_dedup_key.sql`
- DB upsert uniqueness exists:
  - `src/db/repository/configs.rs` (`ON CONFLICT(dedup_key)`)
- Policy notes already exist in validation docs:
  - `docs/validation/2_storage_persistence_parity_checklist.md` (documents
    canonical key as intentional design difference)

Known differences vs xray-knife:

- xray-knife primarily dedups by trimmed raw link string.
- xrat dedups by canonicalized semantic key (`dedup_key`), which intentionally
  merges equivalent variants.

Scanner-specific dedup:

- scanner IP-level dedup is not implemented yet because scanner orchestration is
  not yet implemented in xrat.

---

## Checklist

### `../xray-knife/QA/7_deduplication_mechanisms.md` alignment

- [x] In-memory config dedup path exists before durable write.
- [x] DB uniqueness constraint/upsert path prevents duplicate durable rows.
- [x] Dedup behavior is deterministic and represented by explicit key model.
- [x] Canonical dedup-key migration/versioning exists.
- [x] Confirm/document canonical dedup key as long-term source of truth.
- [x] Add explicit cross-tool comparability note (raw-link vs canonical key).
- [x] If scanner is added, add IP-level final-result dedup strategy + DB key.

Gap status summary:

- **MATCHED / STRONGER IN XRAT** for canonical config dedup.
- **MOSTLY COMPLETE** for config/subscription dedup semantics and enforcement.
- **MISSING (dependency)** for scanner-result dedup policy tied to full area #6
  scanner parity.

---

## Suggested implementation order

1. [x] Finalize dedup policy decision in docs: canonical semantic key is
       normative.
2. [x] Add/expand tests for canonicalization edge cases (ordering, defaults,
       equivalent forms).
3. [x] Add scanner IP-level dedup when scanner subsystem is introduced.

---

## Exit criteria

- [x] Dedup contract is documented as canonical-key based and stable.
- [x] Canonical-key dedup behavior is covered by focused tests.
- [x] Scanner dedup policy is implemented (or explicitly deferred) with clear
      rationale.

---

## Summary

- xrat already has robust canonical dedup foundations and DB enforcement.
- Main remaining work is test depth for canonicalization edge cases and
  scanner-linked dedup once area #6 reaches fuller parity.

## Completion blockers

**Reviewed: 2026-06-01**
**Resolved: 2026-06-01**

All blockers have been resolved:

1. **Canonical-key dedup edge-case tests** - Verified: 10 focused edge-case tests exist in `src/model/node_dedup_key.rs` covering Unicode, special characters, port boundaries, all 7 protocols, equivalent URI variants, and network type distinctions.

2. **Scanner IP-level dedup** - Verified: Scanner uses `BTreeSet` for IP dedup before probing (`src/app/commands/scan.rs`) and `UNIQUE(ip)` upsert in the DB (`migrations/*/0011_add_cf_scan_results.sql`).

3. **Exit criteria met** - All exit criteria checkboxes updated to reflect actual implementation state.
