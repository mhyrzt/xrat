# Phase 1 Status

## Scope Reference

`PLAN.md` is not present in the repository root at the moment, so this status is
based on `plan/README.md` and the current `src/` implementation.

Phase 1 in `plan/README.md` includes:

- finalize parsing behavior for the supported link formats
- normalize parsed nodes consistently before persistence
- support importing from subscription URLs and raw subscription text
- deduplicate configs before saving them

## Current State

### Implemented

1. **Parsing for supported protocols exists**
   - `src/parser.rs` parses `vless`, `vmess`, `ss`, `trojan`, `http`, and
     `socks5`
   - `vless`, `vmess`, and `ss` have regression coverage based on the currently
     expected behavior
   - `trojan`, `http`, and `socks5` are implemented in Rust and still need the
     same level of test coverage

2. **Normalization exists**
   - `src/parser.rs` normalizes empty network values to `tcp`
   - websocket nodes inherit `host` from `sni` and default `path` to `/`
   - gRPC nodes default `path` to `/`
   - empty TLS values are normalized to `None`

3. **Deduplication exists**
   - `src/parser.rs` deduplicates nodes using `Node::dedup_key()`
   - `src/model.rs` defines the dedup key from protocol, address, port,
     username, uuid, and password

4. **Import from subscription URL exists**
   - `src/main.rs` and `src/io.rs` support reading from either a URL or a local
     file
   - file content can now be raw JSON, base64-encoded subscription text, plain
     config lines, or newline-separated subscription URLs
   - `src/main.rs` expands URL lists before parsing

5. **CLI flow works for URL input**
   - `src/cli.rs` accepts a generic input source and output file
   - `src/main.rs` writes parsed nodes or raw JSON to disk

6. **Protocol modeling is now typed**
   - `src/model.rs` now uses a `Protocol` enum instead of raw protocol strings
   - this reduces invalid protocol states and makes persistence/API work cleaner
     later

7. **Supported parser behavior is increasingly locked down**
   - `src/parser.rs` now includes focused regression tests for `vless`, `vmess`,
     and `ss`
   - normalization and dedup behavior are also covered by parser tests

## Remaining Work For Phase 1

1. **CLI import does not use the JSON-aware import parser**
   - `xrat import` currently flows through `src/app/import.rs::load_nodes`,
     decodes the input with `decode_or_raw_text`, then calls
     `src/config/mod.rs::parse_text`.
   - `parse_text` only handles line-based share links and does not use
     `src/config/import/mod.rs::parse_import`, so SIP008/Xray JSON auto-detect
     is not active in the persisted import path.

2. **Raw JSON/file JSON import is rejected**
   - `src/app/import.rs::reject_raw_json_config` rejects any JSON before
     parsing.
   - This contradicts the older status text claiming file content can be raw
     JSON and the import docs claiming JSON format detection for URLs/files.

3. **Xray JSON parsing is validation-only**
   - `src/config/import/parsers/xray.rs::parse_xray_json` currently deserializes
     Xray JSON and returns an empty node list.
   - It does not yet extract outbounds and convert them into persisted `Node`
     values.

4. **Subscription metadata parsing is not wired into `xrat import`**
   - `src/config/import/subscription.rs::fetch_subscription` parses
     `subscription-userinfo` metadata, but `xrat import` uses
     `src/app/input/source.rs::fetch_url` and only receives raw bytes.
   - URL imports therefore do not persist or expose subscription metadata from
     response headers.

5. **Mixed input ingestion still needs focused tests**
   - Decode tests now cover base64, raw JSON, and raw text fallback in
     `src/support/decode.rs`.
   - Import/config tests cover base64 and plain link lists, but the persisted
     CLI import flow still needs tests for files, URL-list expansion, JSON
     rejection or support, and subscription metadata behavior.

## Phase 1 Assessment

Phase 1 is **mostly complete**, but not fully finished.

Most parser, normalization, deduplication, and decode behavior is now covered by
tests, but the persisted import path has important behavior gaps around JSON
format handling and subscription metadata.

### Done

- parser support for `vless`, `vmess`, `ss`, `trojan`, `http`, and `socks5`
- normalization pass
- deduplication before save
- subscription URL ingestion
- local file ingestion for base64, plain links, or newline-separated URLs
- typed `Protocol` enum in the domain model
- regression tests for core supported parser behavior
- decode tests for base64, raw JSON, raw text fallback, and empty input

### Left to finish

- decide whether raw JSON import is in scope for Phase 1; either support it in
  `xrat import` or update docs/status text to say it is intentionally
  unsupported
- wire `xrat import` to the JSON-aware import parser if SIP008/Xray JSON import
  remains in scope
- implement Xray outbound extraction if Xray JSON import remains in scope
- wire subscription URL imports through the metadata-aware subscription fetcher
  or document metadata as out of scope
- add focused tests for file-based mixed input handling and the persisted import
  command path

## Suggested Completion Criteria

Phase 1 can be considered complete after:

1. the CLI or import layer accepts either:
   - a subscription URL, or
   - raw subscription text / file input
2. parser and decode behavior is covered by focused tests
3. supported link formats are validated by regression tests
