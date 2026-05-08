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
- No dedicated `parse` CLI command exists.
- JSON outputs exist for runtime/session commands (`connect/status/disconnect`),
  but not for parse diagnostics.
- Xray runtime JSON generation exists and is reusable:
  - `src/xray/config/mod.rs`
  - `src/xray/config/outbound.rs`
- No sing-box parser/runtime path currently exists.

---

## File-by-file delta checklist

## A) CLI surface: new `parse` command

### `src/cli/parse.rs` (new)

- [ ] Add `ParseArgs` with input modes:
  - [ ] positional link arg (single),
  - [ ] `--file <path>`,
  - [ ] `--stdin`,
  - [ ] `--json`,
  - [ ] `--engine <auto|xray|sing-box>` (default `auto`).
- [ ] Validate incompatible combinations:
  - [ ] `--json` + multiple links should fail (or define explicit array format).
  - [ ] only one input mode active unless intentionally supported.
- [ ] Add unit tests for clap parsing in `src/cli/tests.rs`.

### `src/cli/command.rs`

- [ ] Add `Command::Parse(ParseArgs)` variant and help text.

### `src/cli/mod.rs`

- [ ] Export `ParseArgs`.

### `src/main.rs` / root CLI wiring files

- [ ] Ensure parse subcommand is visible in CLI help and dispatch path.

---

## B) App command handler

### `src/app/commands/parse.rs` (new)

- [ ] Implement command entrypoint `run(context, args)` (context optional if not
      needed).
- [ ] Load links from selected input source(s), trim and ignore empty lines.
- [ ] For each link:
  - [ ] resolve engine (or auto-resolve),
  - [ ] parse + normalize,
  - [ ] render details or JSON.
- [ ] Return structured `AppError` for parse failures with clear source context
      (arg/file line/stdin index).

### `src/app/commands/mod.rs`

- [ ] Register new parse handler in command dispatch.

### `src/app/error.rs`

- [ ] Add/extend errors for parse command validation (input mode conflict, JSON
      multi-link conflict, unsupported engine/scheme, parse failure details).

---

## C) Input handling reuse / extension

### `src/app/input/source.rs` and/or `src/app/import.rs`

- [ ] Extract reusable helpers for reading:
  - [ ] single raw link,
  - [ ] file path line list,
  - [ ] stdin text.
- [ ] Keep parse command non-persistent (must not write DB records).
- [ ] Reuse existing decode behavior where appropriate (base64-encoded lists).

---

## D) Parse domain service (recommended extraction)

### `src/config/parse_service.rs` (new, optional but recommended)

- [ ] Add high-level API for:
  - [ ] parse single link -> normalized node + engine decision,
  - [ ] parse batch with indexed errors.
- [ ] Keep parsing behavior aligned with existing protocol modules in
      `src/config/protocols/*.rs`.
- [ ] Ensure no silent swallowing in parse mode (import currently skips invalid
      lines after warning; parse mode should report explicitly).

### `src/config/mod.rs`

- [ ] Wire new service API exports.
- [ ] Keep existing `parse_text` behavior unchanged for import path unless
      explicitly refactored.

---

## E) Engine selection and sing-box path

### `src/model/protocol.rs` and parser mapping files

- [ ] Add protocol identifiers needed for sing-box scope (`hysteria2`/`hy2` at
      minimum).

### `src/config/line.rs` + `src/config/protocols/*`

- [ ] Add parser entry for sing-box-only schemes.
- [ ] Decide representation:
  - [ ] extend `Node` for fields required by sing-box protocol(s), or
  - [ ] add protocol-specific extension payload.

### `src/app/config/runtime.rs` (or new engine module)

- [ ] Introduce engine enum and resolver:
  - [ ] `auto` maps schemes to xray or sing-box,
  - [ ] explicit override validates compatibility.

### `src/singbox/*` (new module family)

- [ ] Add sing-box runtime config generator equivalent to xray generator:
  - [ ] outbound build from parsed node,
  - [ ] minimal inbound for local proxy endpoint(s),
  - [ ] JSON serialization type definitions.
- [ ] Add runtime process adapter if needed for future command reuse.

Note: For parse-only phase, sing-box runtime process spawn is optional if
`--json` can generate config without launching.

---

## F) `parse --json` output path

### `src/app/commands/parse.rs`

- [ ] Implement `--json` output logic:
  - [ ] for xray engine: call `src/xray/config/*` builders,
  - [ ] for sing-box engine: call new `src/singbox/config/*` builders.
- [ ] Output pretty JSON to stdout.
- [ ] Remove/omit empty optional fields for cleaner output (define deterministic
      cleanup policy).

### `src/xray/config/mod.rs` (possible extension)

- [ ] Add helper for parse-preview JSON with default local inbound(s), mirroring
      xray-knife parse JSON behavior where reasonable.

---

## G) Testing checklist

### CLI tests (`src/cli/tests.rs`)

- [ ] parse command arg parsing: single link, file, stdin.
- [ ] `--json` + multi-link rejection.
- [ ] engine flag accepted values and invalid value rejection.

### Config/parser unit tests (`src/config/*`)

- [ ] new scheme parser tests (`hysteria2`/`hy2`).
- [ ] auto engine routing tests by scheme.
- [ ] normalization defaults for new protocols.

### App command tests (`src/app/commands/parse.rs` tests)

- [ ] parse details output for valid link.
- [ ] parse failure output includes link index/source.
- [ ] parse JSON output shape sanity for xray.
- [ ] parse JSON output shape sanity for sing-box.

### Regression safety

- [ ] import/add behavior remains unchanged for existing protocols.
- [ ] existing test suite stays green (`cargo test -q`).

---

## Behavior differences to resolve explicitly

- [ ] **Error policy**: should parse batch stop on first error (xray-knife-like)
      or continue and summarize?
- [ ] **`--json` with multiple links**: reject (simpler parity) vs emit JSON
      array (xrat-friendly extension).
- [ ] **Cleanup policy** for JSON output: exact empty-field stripping parity vs
      typed serializer defaults.
- [ ] **Engine override semantics**: explicit `--engine xray` on sing-box-only
      scheme should hard-fail with compatibility message.

---

## Suggested implementation order (small safe slices)

1. [ ] Add CLI + handler skeleton for `parse` without sing-box and without JSON
      (details output only for existing protocols).
2. [ ] Add `--json` for xray path by reusing `src/xray/config/*`.
3. [ ] Add engine resolver (`auto|xray|sing-box`) and compatibility checks.
4. [ ] Add initial sing-box protocol parsing (`hysteria2`/`hy2`) + JSON builder.
5. [ ] Add full tests and docs updates.

---

## Documentation updates checklist

- [ ] Update `README.md` command list with `parse`.
- [ ] Add examples:
  - [ ] `xrat parse 'vless://...'`
  - [ ] `xrat parse --file links.txt`
  - [ ] `xrat parse --stdin`
  - [ ] `xrat parse --json --engine auto 'vless://...'`
- [ ] Add design note in `docs/plan/` describing engine abstraction decision.

---

## Exit criteria for "Area #1 complete"

- [ ] `xrat parse` exists and works for single/file/stdin inputs.
- [ ] `xrat parse --json` exists and produces runtime config preview.
- [ ] engine selection supports `auto|xray|sing-box`.
- [ ] sing-box protocol(s) in scope are parseable and validated.
- [ ] tests added and passing.
- [ ] docs updated.
