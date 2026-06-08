# Add HTTP Client Port

## Finding

### [Priority: High] Add an HTTP client port for external HTTP calls

**Files involved:**

- `src/app/commands/upgrade/release.rs`
- `src/app/commands/geoip/download/executor.rs`
- `src/tui/run/tasks/version_check.rs`
- `src/app/input/source.rs`
- `src/config/import/subscription.rs`
- `src/prober/real_delay/check/request.rs`
- `src/prober/download/check/proxied.rs`
- `src/prober/upload/request.rs`

**Problem:** `reqwest` is used directly in 8 production files outside of GeoIP
lookups. Upgrade downloads, TUI version check, subscription URL imports, and all
three probers (download, upload, real-delay) each construct their own
`reqwest::Client` with their own builder configuration, timeout settings, and
proxy configuration. None of these paths can be tested without real HTTP servers
or network access.

**Why this change is needed:** Test coverage for upgrade, import, and probe
flows is weak because HTTP calls cannot be stubbed. Timeouts, bad responses,
redirects, DNS failures, and proxy errors are untested. Duplicated client
builder configuration also risks inconsistent timeout or TLS settings.

**How to implement it:** Introduce an `HttpClient` trait with methods for GET,
HEAD, and streaming downloads. Provide a `ReqwestClient` production adapter and
a `MockHttpClient` test adapter. Add a factory that wires proxy settings and
timeouts from app configuration. Replace direct `reqwest::Client` builder calls
in all 8 files with the injected port.

**Positive effect on the codebase:** Upgrade, import, version-check, and probe
tests can simulate HTTP responses without network. HTTP timeout and error
handling become consistent across the codebase.

**Suggested target architecture:** `HttpClient` port in application layer;
`ReqwestClient` adapter in infrastructure layer; injected via `AppContext` or
use-case constructors.

**Risk / migration notes:** Low risk. Start with the simplest consumers (version
check, subscription import) and migrate probers and upgrade last. Keep the
existing `GeoIpLookup` remote lookups as-is since they are already abstracted
behind that trait.
