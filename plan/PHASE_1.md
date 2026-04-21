# Phase 1 Status

## Scope Reference

`PLAN.md` is not present in the repository root at the moment, so this status is based on `plan/README.md` and the current `src/` implementation.

Phase 1 in `plan/README.md` includes:

- finalize parsing behavior and verify parity with the original Python script
- normalize parsed nodes consistently before persistence
- support importing from subscription URLs and raw subscription text
- deduplicate configs before saving them

## Current State

### Implemented

1. **Parsing for supported protocols exists**
   - `src/parser.rs` parses `vless`, `vmess`, `ss`, `trojan`, `http`, and `socks5`
   - `vless`, `vmess`, and `ss` match the current Python reference in `v2p.py`
   - `trojan`, `http`, and `socks5` have now been added in Rust

2. **Normalization exists**
   - `src/parser.rs` normalizes empty network values to `tcp`
   - websocket nodes inherit `host` from `sni` and default `path` to `/`
   - gRPC nodes default `path` to `/`
   - empty TLS values are normalized to `None`

3. **Deduplication exists**
   - `src/parser.rs` deduplicates nodes using `Node::dedup_key()`
   - `src/model.rs` defines the dedup key from protocol, address, port, username, uuid, and password

4. **Import from subscription URL exists**
   - `src/main.rs` and `src/io.rs` support reading from either a URL or a local file
   - file content can now be raw JSON, base64-encoded subscription text, plain config lines, or newline-separated subscription URLs
   - `src/main.rs` expands URL lists before parsing

5. **CLI flow works for URL input**
   - `src/cli.rs` accepts a generic input source and output file
   - `src/main.rs` writes parsed nodes or raw JSON to disk

6. **Protocol modeling is now typed**
   - `src/model.rs` now uses a `Protocol` enum instead of raw protocol strings
   - this reduces invalid protocol states and makes persistence/API work cleaner later

7. **Python parity is largely preserved**
   - Rust behavior in `src/parser.rs` and `src/decode.rs` closely follows `v2p.py`
   - module split is cleaner, but functional behavior is substantially the same for the shared protocols

## Remaining Work For Phase 1

1. **Parity is not yet verified by tests**
   - implementation looks aligned with `v2p.py`, but there are no parser/decode tests proving parity
   - Phase 1 says to finalize parsing behavior and verify parity, which is only partially complete without validation coverage

2. **Normalization rules are implemented but not documented/locked by tests**
   - behavior exists in code, but regression protection is missing

3. **Mixed input ingestion needs tests**
   - file ingestion now supports JSON, base64, plain link lists, and newline-separated URLs
   - this behavior should be locked down with focused tests before Phase 1 is considered complete

## Phase 1 Assessment

Phase 1 is **mostly complete**, but not fully finished.

### Done

- parser support for `vless`, `vmess`, `ss`, `trojan`, `http`, and `socks5`
- normalization pass
- deduplication before save
- subscription URL ingestion
- local file ingestion for JSON, base64, plain links, or newline-separated URLs
- typed `Protocol` enum in the domain model

### Left to finish

- add tests to verify parser/decoder parity and normalization behavior
- add tests for file-based mixed input handling

## Suggested Completion Criteria

Phase 1 can be considered complete after:

1. the CLI or import layer accepts either:
   - a subscription URL, or
   - raw subscription text / file input
2. parser and decode behavior is covered by focused tests
3. parity cases from `v2p.py` are validated for the supported protocols
