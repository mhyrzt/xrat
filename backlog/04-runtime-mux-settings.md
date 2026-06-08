# 04. Medium, P2: Runtime Mux settings for generated Xray outbounds

### Status

Planned

### Goal

Expose Xray outbound Mux/XUDP settings in `config.toml` and apply them to
generated runtime and probe configs when the runtime engine is Xray. This should
let users opt into client-side multiplexing without manually editing generated
Xray JSON.

### Background

Many Xray clients expose Mux because Xray supports a `mux` object directly on an
`OutboundObject`. Mux distributes multiple logical connections over fewer
physical TCP connections, and XUDP can aggregate UDP traffic through Mux.

Official Xray docs warn that Mux is designed to reduce TCP handshake latency,
not to increase throughput. It can be useful for many short-lived TCP requests,
but often hurts video, downloads, and speed tests. Keep it disabled by default.

Reference:

- Xray outbound `MuxObject`: https://xtls.github.io/en/config/outbound.html

### Current behavior

- Parser support already exists for full Xray JSON:
  - `src/xray/parsing/protocols/outbounds.rs` has `MuxObject`.
  - `src/xray/parsing/protocols/outbounds.rs` has `BaseOutboundObject.mux`.
- Generated runtime configs do not expose or set Mux:
  - `src/xray/config/types.rs` defines the generated `Outbound` without a `mux`
    field.
  - `src/xray/config/outbound.rs` builds protocol outbounds from normalized
    `Node` records only.
  - `src/xray/config/generator/mod.rs` calls `node_to_outbound(node, "proxy")`
    without runtime config context.
- `config.toml` has no `[runtime.mux]` or equivalent section.

### Proposed user-facing config

Add a disabled-by-default section under the existing runtime configuration
family. It should sit near `[runtime.rotation]`, `[runtime.log]`,
`[runtime.socks]`, `[runtime.http]`, and `[runtime.shadowsocks]` because Mux is
a runtime outbound-generation option, not an import/parser option and not a
per-subscription setting.

Recommended placement in `config.toml`:

```toml
[runtime]
engine = "xray"
replace_active_session = true

[runtime.mux]
enabled = false
concurrency = 8
xudp_concurrency = 0
xudp_proxy_udp443 = "reject"

[runtime.rotation]
enabled = true
```

Minimal section:

```toml
[runtime.mux]
enabled = false
concurrency = 8
xudp_concurrency = 0
xudp_proxy_udp443 = "reject"
```

Field behavior:

- `enabled`: writes `mux.enabled`; default `false`.
- `concurrency`: TCP Mux concurrency; Xray treats omitted or `0` as `8`, allows
  `1..=128`, and treats values above `128` as `128`.
- `xudp_concurrency`: XUDP aggregation concurrency; Xray treats omitted or `0`
  as the traditional same-path behavior, allows `1..=1024`, and negative values
  opt out of Mux for UDP.
- `xudp_proxy_udp443`: one of `reject`, `allow`, or `skip`; default `reject`
  avoids carrying QUIC/UDP 443 through Mux.

Section ownership:

- Belongs under `[runtime.mux]`.
- Applies only while generating runtime/probe Xray configs from normalized
  nodes.
- Does not belong under `[testing]`; tests may choose to inherit or ignore it,
  but the setting is not a test-stage setting.
- Does not belong under `[parser]` or import protocol sections; imported links
  should not silently set global runtime behavior.

### Changes required

- Add config types/defaults under `src/app/config/`:
  - Add `MuxSettings` to runtime settings.
  - Add default constants in `src/app/config/defaults.rs`.
  - Deserialize `[runtime.mux]` with defaults.
- Add validation in `src/app/commands/validate.rs`:
  - `concurrency` should be `-1` or `0..=128` if negative opt-out is supported;
    otherwise require `0..=128`.
  - `xudp_concurrency` should be `-1` or `0..=1024`.
  - `xudp_proxy_udp443` must be `reject`, `allow`, or `skip`.
- Extend generated Xray config types:
  - Add a generated `Mux`/`MuxSettings` struct in `src/xray/config/types.rs`.
  - Add `mux: Option<Mux>` to generated `Outbound`.
  - Use Xray camelCase keys: `enabled`, `concurrency`, `xudpConcurrency`,
    `xudpProxyUDP443`.
- Thread runtime settings into generation:
  - Either add `node_to_outbound_with_options(...)` or pass a small Xray runtime
    generation options struct.
  - Apply Mux only to the proxy outbound, not direct/block/freedom helper
    outbounds unless explicitly required later.
  - Apply to `generate_runtime_config*` and decide whether probe configs should
    inherit it. Prefer making probe inheritance explicit because Mux can distort
    speed/latency tests.
- Update generated/default config files:
  - `src/app/commands/init_default_config.toml`
  - `testdata/config.example.toml`
- Update docs:
  - `docs/src/05-reference/config-file.md`
  - `docs/src/01-getting-started/configuration.md`
  - `docs/src/06-architecture/config-generation.md`

### Compatibility and safety

- Default must remain behavior-preserving: no `mux` object emitted when
  `[runtime.mux].enabled = false`.
- Do not infer Mux settings from imported share links unless the normalized
  model grows explicit support for source-provided Mux options.
- Avoid adding Mux to sing-box generation unless a separate sing-box-specific
  implementation is designed.
- If probe configs inherit Mux, document that speed tests may be misleading.

### Verification

- Config parsing test: omitted `[runtime.mux]` uses disabled defaults.
- Config parsing test: explicit section preserves all field values.
- Validation tests: invalid `xudp_proxy_udp443`, out-of-range concurrency, and
  out-of-range XUDP concurrency fail.
- Xray generation tests:
  - Disabled Mux omits `mux`.
  - Enabled Mux serializes camelCase keys.
  - Only the proxy outbound receives `mux`.
- Manual check:
  - Run `xrat parse <config-id>` or generated runtime preview and confirm Xray
    JSON contains the expected `mux` block only when enabled.

### Open decisions

- Should probe/test configs inherit runtime Mux settings, or should Mux apply
  only to managed runtime connections?
- Should negative values be accepted to expose Xray's TCP/UDP opt-out behavior,
  or should xrat keep the config simpler with non-negative validation only?
- Should imported configs eventually preserve per-link Mux settings, or should
  xrat keep Mux as a global runtime option?
