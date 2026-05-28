# Parse + Validate Config Parity Checklist (xray-knife -> xrat)

This checklist is a detailed implementation plan for gap area **#1 Parse +
Validate Config**, based on:

- `docs/validation/xray-knife_vs_xrat_gap_checklist.md`
- `../xray-knife/QA/1_parse_and_validate_config.md`
- `../xray-knife/cmd/parse/parse.go`
- `../xray-knife/pkg/core/factory.go`

Decisions confirmed for this track:

- Include **sing-box** support.
- Add dedicated **`parse` command**.
- Add **`parse --json`** output mode.

---

## Scope and target behavior

Parity target for this phase:

1. Parse one or many links from arg/file/stdin without importing to DB.
2. Validate and normalize through protocol parser pipeline.
3. Output either:
   - human-readable parsed details (default),
   - generated runtime JSON (`--json`).
4. Support core selection mode:
   - explicit `--engine xray|sing-box`,
   - `--engine auto` (scheme-based routing).

Out of scope for this phase:

- Proxy rotation and scanner features.
- Full HTTP test parity fields (covered in area #3).

---

## Current state snapshot (xrat)

- Parsing exists via import/add flows:
  - `src/config/line.rs`
  - `src/config/protocols/*.rs`
  - `src/config/normalize.rs`
  - `src/app/import.rs`
- Dedicated `parse` CLI command exists:
  - `src/cli/parse.rs`
  - `src/app/commands/parse.rs`
- Parse diagnostics support both human-readable output and `--json`.
- Xray runtime JSON generation exists and is reusable:
  - `src/xray/config/mod.rs`
  - `src/xray/config/outbound.rs`
- Sing-box parse-preview JSON path exists for hy2/hysteria2:
  - `src/config/protocols/hy2.rs`
  - `src/singbox/config.rs`

---

## File-by-file delta checklist

## A) CLI surface: new `parse` command

### `src/cli/parse.rs` (new)

- [x] Add `ParseArgs` with input modes:
  - [x] positional link arg (single),
  - [x] `--file <path>`,
  - [x] `--stdin`,
  - [x] `--json`,
  - [x] `--engine <auto|xray|sing-box>` (default `auto`).
- [x] Validate incompatible combinations:
  - [x] `--json` + multiple links should fail (or define explicit array format).
  - [x] only one input mode active unless intentionally supported.
- [x] Add unit tests for clap parsing in `src/cli/tests.rs`.

### `src/cli/command.rs`

- [x] Add `Command::Parse(ParseArgs)` variant and help text.

### `src/cli/mod.rs`

- [x] Export `ParseArgs`.

### `src/main.rs` / root CLI wiring files

- [x] Ensure parse subcommand is visible in CLI help and dispatch path.

---

## B) App command handler

### `src/app/commands/parse.rs` (new)

- [x] Implement command entrypoint `run(context, args)` (context optional if not
      needed).
- [x] Load links from selected input source(s), trim and ignore empty lines.
- [x] For each link:
  - [x] resolve engine (or auto-resolve),
  - [x] parse + normalize,
  - [x] render details or JSON.
- [x] Return structured `AppError` for parse failures with clear source context
      (arg/file line/stdin index).

### `src/app/commands/mod.rs`

- [x] Register new parse handler in command dispatch.

### `src/app/error.rs`

- [x] Add/extend errors for parse command validation (input mode conflict, JSON
      multi-link conflict, unsupported engine/scheme, parse failure details).

---

## C) Input handling reuse / extension

### `src/app/input/source.rs` and/or `src/app/import.rs`

- [x] Extract reusable helpers for reading:
  - [x] single raw link,
  - [x] file path line list,
  - [x] stdin text.
- [x] Keep parse command non-persistent (must not write DB records).
- [x] Reuse existing decode behavior where appropriate (base64-encoded lists).

---

## D) Parse domain service (recommended extraction)

### `src/config/parse_service.rs` (new, optional but recommended)

- [x] Add high-level API for:
  - [x] parse single link -> normalized node + engine decision,
  - [x] parse batch with indexed errors.
- [x] Keep parsing behavior aligned with existing protocol modules in
      `src/config/protocols/*.rs`.
- [x] Ensure no silent swallowing in parse mode (import currently skips invalid
      lines after warning; parse mode should report explicitly).

### `src/config/mod.rs`

- [x] Wire new service API exports.
- [x] Keep existing `parse_text` behavior unchanged for import path unless
      explicitly refactored.

---

## E) Engine selection and sing-box path

### `src/model/protocol.rs` and parser mapping files

- [x] Add protocol identifiers needed for sing-box scope (`hysteria2`/`hy2` at
      minimum).

### `src/config/line.rs` + `src/config/protocols/*`

- [x] Add parser entry for sing-box-only schemes.
- [x] Decide representation:
  - [x] extend `Node` for fields required by sing-box protocol(s), or
  - [x] add protocol-specific extension payload.

### `src/app/config/runtime.rs` (or new engine module)

- [x] Introduce engine enum and resolver:
  - [x] `auto` maps schemes to xray or sing-box,
  - [x] explicit override validates compatibility.

### `src/singbox/*` (new module family)

- [x] Add sing-box runtime config generator equivalent to xray generator:
  - [x] outbound build from parsed node,
  - [x] minimal inbound for local proxy endpoint(s),
  - [x] JSON serialization type definitions.
- [x] Add runtime process adapter if needed for future command reuse.

Note: For parse-only phase, sing-box runtime process spawn is optional if
`--json` can generate config without launching.

---

## F) `parse --json` output path

### `src/app/commands/parse.rs`

- [x] Implement `--json` output logic:
  - [x] for xray engine: call `src/xray/config/*` builders,
  - [x] for sing-box engine: call new `src/singbox/config/*` builders.
- [x] Output pretty JSON to stdout.
- [x] Remove/omit empty optional fields for cleaner output (define deterministic
      cleanup policy).

### `src/xray/config/mod.rs` (possible extension)

- [x] Add helper for parse-preview JSON with default local inbound(s), mirroring
      xray-knife parse JSON behavior where reasonable.

---

## G) Testing checklist

### CLI tests (`src/cli/tests.rs` + `src/app/commands/parse.rs` tests)

- [x] parse command arg parsing: single link, file, stdin.
- [x] `--json` + multi-link rejection.
- [x] engine flag accepted values and invalid value rejection.

### Config/parser unit tests (`src/config/*`)

- [x] new scheme parser tests (`hysteria2`/`hy2`).
- [x] auto engine routing tests by scheme.
- [x] normalization defaults for new protocols.

### App command tests (`src/app/commands/parse.rs` tests)

- [x] parse details output for valid link.
- [x] parse failure output includes link index/source.
- [x] parse JSON output shape sanity for xray.
- [x] parse JSON output shape sanity for sing-box.

### Regression safety

- [x] import/add behavior remains unchanged for existing protocols.
- [x] existing test suite stays green (`cargo test -q`).

---

## Behavior differences to resolve explicitly

- [x] **Error policy**: parse batch stops on first error (xray-knife-like).
- [x] **`--json` with multiple links**: reject (simpler parity) vs emit JSON
      array (xrat-friendly extension).
- [x] **Cleanup policy** for JSON output: exact empty-field stripping parity vs
      typed serializer defaults.
- [x] **Engine override semantics**: explicit `--engine xray` on sing-box-only
      scheme should hard-fail with compatibility message.

---

## Suggested implementation order (small safe slices)

1. [x] Add CLI + handler skeleton for `parse` without sing-box and without JSON
       (details output only for existing protocols).
2. [x] Add `--json` for xray path by reusing `src/xray/config/*`.
3. [x] Add engine resolver (`auto|xray|sing-box`) and compatibility checks.
4. [x] Add initial sing-box protocol parsing (`hysteria2`/`hy2`) + JSON builder.
5. [ ] Add full tests and docs updates.

---

## Documentation updates checklist

- [x] Update `README.md` command list with `parse`.
- [ ] Add examples:
  - [ ] `xrat parse 'vless://...'`
  - [ ] `xrat parse --file links.txt`
  - [ ] `xrat parse --stdin`
  - [x] `xrat parse --json --engine auto 'vless://...'`
- [ ] Add design note in `docs/plan/` describing engine abstraction decision.

---

## Exit criteria for "Area #1 complete"

- [x] `xrat parse` exists and works for single/file/stdin inputs.
- [x] `xrat parse --json` exists and produces runtime config preview.
- [x] engine selection supports `auto|xray|sing-box`.
- [x] sing-box protocol(s) in scope are parseable and validated.
- [x] tests added and passing.
- [ ] docs updated.
