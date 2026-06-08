# Move PAC Domain Logic Out Of The Axum Route Module

## Finding

### [Priority: Low] Move PAC domain logic out of the Axum route module

**Files involved:**

- `src/server/routes/pac.rs`
- `src/app/config/routing.rs`
- `src/app/commands/proxy/pac.rs`

**Problem:** `src/server/routes/pac.rs` contains HTTP request handling,
host-header authorization, active runtime endpoint lookup, PAC rule models, PAC
rendering, CIDR helpers, and JavaScript escaping. Only the route-specific pieces
belong in the server adapter.

**Why this change is needed:** PAC rendering is domain/application behavior that
can be reused by CLI proxy commands, tests, and potentially TUI sharing
features. Keeping it in a route module makes it feel HTTP-specific and increases
coupling between Axum and proxy configuration behavior.

**How to implement it:** Move `PacEndpoints`, `PacRules`, `render_pac`, and PAC
helper functions into `src/app/proxy/pac.rs` or `src/app/services/proxy_pac.rs`.
Keep `proxy_pac`, `require_allowed_pac_host`, and response header construction
in `src/server/routes/pac.rs`. Add pure tests for PAC rendering in the new
module and route tests only for HTTP behavior.

**Positive effect on the codebase:** PAC behavior becomes reusable and easier to
test without Axum. The route file shrinks to a thin adapter.

**Suggested target architecture:** Proxy/PAC application module owns rendering
and rule conversion; HTTP and CLI adapters expose it.

**Risk / migration notes:** Low risk because the rendering functions are mostly
pure. Move tests with the module and keep existing route tests as compatibility
checks.
