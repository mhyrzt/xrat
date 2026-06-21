# Medium, P2: Distinguish GeoIP fronting endpoint from proxy origin

### Status

Planned

### Goal

Avoid presenting Cloudflare, DNS-provider proxy, CDN, or relay GeoIP metadata as
the actual Xray/sing-box server location when the imported config intentionally
dials a fronting endpoint.

The feature should make the distinction visible to users:

- **Dial/fronting endpoint**: the address xrat and the runtime connect to first.
- **Proxy origin**: the actual server behind that front door, when it can be
  verified.
- **Unknown origin**: the correct state when xrat only sees a CDN/relay address
  and has no reliable way to prove the backend server IP or country.

### Root cause

GeoIP enrichment currently looks up the configured dial address, not a verified
origin server:

- `src/app/commands/test/execution/stages.rs` passes `node.address` into
  `resolve_endpoint_meta`.
- `src/app/commands/test/stages/endpoint.rs` calls
  `geoip::enrich_address(endpoint_address, geoip_lookup)`.
- `src/support/geoip/enrich.rs` extracts the host, resolves one DNS result with
  `tokio::net::lookup_host`, then runs MMDB/remote lookup for that IP.
- TUI background enrichment follows the same address-based model and caches by
  host in `src/tui/run/tasks/data.rs`.

That behavior is technically correct for the dial endpoint, but misleading for
configs where the address is a front door.

### Detailed example

Consider a VLESS config shaped like this:

```text
protocol: vless
address: vpn.safe.com
port: 443
tls: tls
sni: origin.safe.internal
host: origin.safe.internal
path: /edge
```

Operationally, the connection can look like this:

```text
xrat/xray client
  -> vpn.safe.com:443
  -> DNS returns 104.21.10.20
  -> 104.21.10.20 belongs to Cloudflare
  -> Cloudflare/reverse proxy forwards traffic to the real Xray backend
  -> backend may be in a different country/ASN
```

Today xrat does this:

```text
node.address = "vpn.safe.com"
address_host("vpn.safe.com") = "vpn.safe.com"
lookup_host("vpn.safe.com") = 104.21.10.20
MMDB(104.21.10.20) = US / AS13335 CLOUDFLARENET
stored result = endpoint_country=US, endpoint_asn=AS13335 CLOUDFLARENET
```

The lookup result is not wrong for `104.21.10.20`, but the label is ambiguous.
Users read `endpoint_country` as "the proxy server is in the US", while the more
accurate meaning is "the configured dial endpoint currently resolves to a
Cloudflare IP that geolocates to the US." The actual proxy origin may be in
Germany, the Netherlands, Iran, or may be intentionally hidden.

A similar problem happens when the config uses a literal relay IP:

```text
address: aaa.bbb.ccc.ddd
port: 443
sni: real-node.example.net
```

MMDB can only describe `aaa.bbb.ccc.ddd`. If that IP is a relay, load balancer,
or tunnel entrance, it still does not prove the GeoIP location of
`real-node.example.net` or the final Xray service.

### Ambiguous inputs

Different fields can identify different layers of the same connection:

- `vpn.safe.com` resolves to Cloudflare because the real proxy is behind CDN
  fronting.
- `aaa.bbb.ccc.ddd` is only a relay or reverse proxy in front of the actual Xray
  server.
- VLESS/VMess/Trojan configs may have `address`, `sni`, and HTTP `host` values
  that describe different roles.

For these configs, `endpoint_country`, `endpoint_location`, and `endpoint_asn`
describe the fronting/relay provider, not necessarily the origin that ultimately
terminates or serves the proxy session. SNI and Host are also not enough to
derive the origin by themselves; resolving them can return another proxy, a
private backend name, split-horizon DNS, or nothing useful from the user's
network.

### Current impact

- Latest-run summaries and TUI rows can show `US / AS13335 CLOUDFLARENET` for
  many unrelated configs because the dial host is Cloudflare-fronted.
- Filters such as `xrat test --latest-run-summary --country US --asn cloudflare`
  match the fronting provider, not necessarily the actual proxy location.
- The persisted `geoip_cache` key is the host extracted from the dial address,
  so cached values reinforce this interpretation across runs.
- Users can make bad routing or selection decisions because a fronted config may
  look geographically close even when the real backend is elsewhere.
- A config can appear to be duplicated by country/ASN with many other unrelated
  configs simply because they share the same CDN provider.

### Changes required

- Rename or split the stored/displayed metadata so the current lookup is clearly
  `dial_endpoint_*` or `fronting_endpoint_*`, not implicitly "server origin".
- Add a separate origin-resolution model only if xrat can obtain origin evidence
  from the runtime/protocol flow. Do not infer origin country from SNI, Host, or
  DNS alone.
- Add an explicit confidence/source field for GeoIP metadata, for example:
  `dial_dns_mmdb`, `literal_ip_mmdb`, `remote_lookup`, `fronting_detected`, or
  `origin_verified`.
- Preserve current address-based lookup as a useful signal for reachability,
  CDN/relay detection, and operational filtering.
- Consider adding a fronting classification when ASN/organization matches known
  CDN or proxy providers, while keeping the provider list configurable or
  conservative.
- Update CLI/TUI/API labels and docs so users can tell whether a country/ASN is
  for the dial endpoint, a detected fronting provider, or a verified origin.
- Ensure filters document which field they target. A filter named `--country`
  should either become explicitly dial-endpoint based or gain a separate
  `--origin-country` only when origin data exists.

### Possible implementation direction

Start with a semantic split rather than trying to discover hidden origins:

1. Keep the current address lookup, but store or expose it as dial-endpoint
   metadata.
2. Add a `geoip_source` or `geoip_kind` value to make the lookup provenance
   visible in CLI/TUI/API output.
3. Add optional CDN/relay classification based on ASN/org names and known
   provider ranges. Treat this as a warning/hint, not as proof.
4. Leave origin metadata empty unless xrat later gains a reliable runtime signal
   for the final remote peer.

This avoids false precision. Saying "dial endpoint: Cloudflare, origin: unknown"
is more accurate than pretending the origin is Cloudflare or guessing from SNI.

### Verification

- Unit-test configs where `address`, `sni`, and `host` differ to ensure labels
  do not claim origin metadata from `address` lookup alone.
- Add tests for latest-run summary filtering once metadata names are split.
- Add TUI data tests for cached fronting/dial metadata.
- Add output tests that verify Cloudflare/relay examples render as dial/fronting
  metadata with unknown origin.
- Manual:
  - Import a Cloudflare-fronted config and confirm output labels the result as
    fronting/dial endpoint metadata.
  - Import a direct IP config and confirm the same lookup is still displayed as
    the dial endpoint location.

### Open decisions

- Whether to migrate existing `endpoint_country`, `endpoint_location`, and
  `endpoint_asn` columns or keep them as backward-compatible dial-endpoint
  fields.
- Whether "origin" should remain unavailable unless an engine can expose a
  verified remote peer beyond the relay/CDN hop.
- Whether CDN/relay detection belongs in GeoIP enrichment, test result
  formatting, or a separate diagnostics layer.
