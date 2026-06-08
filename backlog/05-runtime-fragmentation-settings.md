# 05. Hard, P2: Runtime Xray fragmentation settings

### Status

Planned

### Goal

Expose Xray TCP fragmentation settings in `config.toml` and apply them to
generated Xray runtime configs in a safe, explicit way. This should support the
fragmentation behavior many Xray clients expose while preserving xrat's current
separation between normalized proxy nodes and generated runtime JSON.

### Background

Xray implements fragmentation on the `freedom` outbound via the `fragment`
object. It can split early outgoing TCP data, commonly TLS ClientHello-related
traffic, using:

- `packets`: either a write range such as `1-3`, or `tlshello`.
- `length`: an `Int32Range` string such as `100-200`.
- `interval`: an `Int32Range` string in milliseconds such as `10-20`.

Xray docs describe this as a way to control outgoing TCP fragmentation and, in
some cases, deceive censorship systems such as SNI blacklist filters. This is a
network-circumvention tuning feature, not a general performance feature, so it
must be disabled by default and documented as environment-dependent.

Reference:

- Xray `freedom` outbound `fragment`:
  https://xtls.github.io/en/config/outbounds/freedom.html

### Current behavior

- Parser support already exists for full Xray JSON:
  - `src/xray/parsing/protocols/common.rs` has `FragmentObject`.
  - `src/xray/parsing/protocols/outbound_settings/basic.rs` has
    `OutboundSettingsFreedom.fragment`.
- Generated runtime configs do not create a `freedom` fragmentation hop:
  - `src/xray/config/generator/mod.rs` emits a single proxy outbound for runtime
    configs.
  - `src/xray/config/outbound.rs` converts normalized `Node` records directly to
    proxy outbounds.
  - `src/xray/config/types.rs` currently models only the generated outbound
    shape needed by direct proxy outbounds.
- `config.toml` has no `[runtime.fragment]` or equivalent section.

### Proposed user-facing config

Add a disabled-by-default section under the existing runtime configuration
family. It should sit near `[runtime.rotation]`, `[runtime.log]`,
`[runtime.socks]`, `[runtime.http]`, and `[runtime.shadowsocks]` because
fragmentation changes generated Xray runtime outbounds. It is not a parser,
database, subscription, or testing-stage setting.

Recommended placement in `config.toml`:

```toml
[runtime]
engine = "xray"
replace_active_session = true

[runtime.fragment]
enabled = false
packets = "tlshello"
length = [100, 200]
interval = [10, 20]

[runtime.rotation]
enabled = true
```

Minimal section:

```toml
[runtime.fragment]
enabled = false
packets = "tlshello"
length = [100, 200]
interval = [10, 20]
```

Field behavior:

- `enabled`: when `false`, generated runtime config remains unchanged.
- `packets`: allow `tlshello` or a positive write range such as `1-3`.
- `length`: Xray `Int32Range` string; allow a single positive integer or
  `min-max`.
- `interval`: Xray `Int32Range` string in milliseconds; allow a single
  non-negative integer or `min-max`.

Section ownership:

- Belongs under `[runtime.fragment]`.
- Applies only while generating Xray runtime configs from normalized nodes.
- Does not belong under `[testing]`; probe/test inheritance is an explicit
  implementation decision, not the primary purpose of the setting.
- Does not belong under `[parser]` or import protocol sections; imported links
  should not silently set global runtime behavior.

### Implementation model

Fragmentation is not a property of the proxy outbound itself. It belongs to a
`freedom` outbound. The generated config needs an explicit outbound chain rather
than simply adding fields to the existing proxy outbound.

Preferred model:

1. Keep the original proxy outbound tagged `proxy`.
2. Add a `freedom` outbound tagged `fragment`.
3. Configure the proxy outbound to dial through the `fragment` outbound using a
   forwarding mechanism that preserves the proxy outbound's transport settings.
4. Route normal inbound traffic to `proxy` as before.

Important Xray nuance:

- `proxySettings` can forward to another outbound, but by default it may ignore
  the current outbound's own `streamSettings`.
- If using `proxySettings`, set `transportLayer = true` when required so the
  proxy outbound's transport stack still works.
- If `sockopt.dialerProxy` is a better fit for this chain, design that as part
  of the implementation instead of forcing `proxySettings`.

### Changes required

- Add config types/defaults under `src/app/config/`:
  - Add `FragmentSettings` to runtime settings.
  - Add default constants in `src/app/config/defaults.rs`.
  - Deserialize `[runtime.fragment]` with defaults.
- Add validation in `src/app/commands/validate.rs`:
  - `packets` must be `tlshello` or a valid positive range.
  - `length` must be a valid positive range.
  - `interval` must be a valid non-negative range.
  - Ranges must have `min <= max`.
- Extend generated Xray config support:
  - Add generated `Fragment` and `FreedomOutboundSettings` structs or build the
    freedom settings JSON in one focused helper.
  - Add support for `proxySettings.transportLayer` or `sockopt.dialerProxy`,
    whichever is selected.
  - Ensure outbound tags stay unique.
- Update runtime generation:
  - Add a generation-options struct so runtime config creation can receive
    app-level settings.
  - When enabled, emit both `proxy` and `fragment` outbounds and chain them
    correctly.
  - Decide whether probe/test configs can use fragmentation. Prefer disabled for
    probes by default because it may change measured latency and success rates.
- Update generated/default config files:
  - `src/app/commands/init_default_config.toml`
  - `testdata/config.example.toml`
- Update docs:
  - `docs/src/05-reference/config-file.md`
  - `docs/src/01-getting-started/configuration.md`
  - `docs/src/06-architecture/config-generation.md`
  - Feature docs if a new runtime tuning page is added.

### Compatibility and safety

- Default must remain behavior-preserving: no `freedom` fragmentation outbound
  emitted when `[runtime.fragment].enabled = false`.
- Do not persist generated root-level Xray JSON in the database; keep generation
  on demand from normalized node data plus runtime settings.
- Do not apply fragmentation to sing-box until a separate sing-box-specific
  feature is designed.
- Avoid silently enabling fragmentation from imported links unless the parser
  and domain model explicitly support preserving source-provided fragmentation
  settings.
- Be clear in docs that fragmentation can help or hurt depending on network,
  transport, destination, and censorship behavior.

### Verification

- Config parsing test: omitted `[runtime.fragment]` uses disabled defaults.
- Config parsing test: explicit section preserves all field values.
- Validation tests:
  - Accept `tlshello`, `1`, `1-3`, `100-200`, and `0-20` for interval.
  - Reject empty values, reversed ranges, negative ranges where not allowed, and
    non-numeric ranges.
- Xray generation tests:
  - Disabled fragmentation emits exactly the existing outbound shape.
  - Enabled fragmentation emits a `freedom` outbound with `settings.fragment`.
  - Enabled fragmentation chains the proxy outbound through the fragmentation
    outbound without dropping transport settings.
  - JSON uses Xray camelCase keys.
- Manual check:
  - Generate a runtime config with fragmentation enabled and run `xray -test`
    against it if the local Xray binary is available.
  - Connect through a known config and confirm normal HTTP and HTTPS traffic
    still works.

### Open decisions

- Should fragmentation apply only to managed runtime configs, or also to
  `xrat test` probe configs?
- Should the chaining mechanism use `proxySettings.transportLayer = true` or
  `sockopt.dialerProxy`?
- Should xrat expose only `fragment`, or also Xray `noises` from the same
  `freedom` outbound feature family?
- Should per-node fragmentation be added later, or is a global runtime setting
  sufficient?
