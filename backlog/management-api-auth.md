# Hard, P1: Proper authentication for a management HTTP API

### Status

Planned

### Goal

Add real authentication suitable for state-changing endpoints before exposing a
management API (add/delete/edit configs, connect/disconnect, rotation control,
etc.). The current `?key=KEY` scheme (`src/server/auth.rs`,
`src/server/routes/*`) is acceptable for read-only, non-critical endpoints but
must not gate mutations.

- also change `?key=` to `?token=`

### Current behavior

- API key passed as a `?key=` query parameter on each route
  (`ConfigsQuery`/`JsonQuery` `key` field), validated by `require_api_key` in
  `src/server/auth.rs`.
- All routes are read-only `GET` (`src/server/routes/mod.rs`): health, json,
  b64, configs list/get, proxy.pac.
- Single shared key compared with `provided != expected`.

### Weaknesses to address for write/management endpoints

- Key in the query string leaks into server access logs, browser history,
  bookmarks, and `Referer` headers — unsafe for privileged operations.
- Comparison is not constant-time (`!=`), a timing side channel.
- No header-based credential (`Authorization: Bearer`/`X-API-Key`).
- No separation of capability: one key grants everything; no read-only vs manage
  distinction.
- No rate limiting, lockout, or audit trail for failed/privileged calls.

### Changes required

- Introduce header-based auth (`Authorization: Bearer <token>` and/or
  `X-API-Key`) for any non-`GET`/management route; keep `?key=` allowed only for
  the existing read-only endpoints (or behind a compat flag) and document it as
  low-sensitivity.
- Use a constant-time comparison for secrets.
- Model capabilities/scopes: at minimum read-only vs manage, so a read token
  cannot mutate state. Consider per-scope tokens.
- Apply auth via middleware on the management router so new routes are protected
  by default (ties into the API request-logging middleware from item 02 — record
  auth failures as best-effort events).
- Bind management endpoints to localhost by default and require explicit opt-in
  to expose them; document the exposure model.
- Add rate limiting / failed-attempt backoff for auth on management routes.
- Record privileged actions as audit events (`src/app/events.rs`).
- CLI/config: generate/store management tokens; surface them through config the
  same way the existing `api_key` is managed.

### Verification

- Auth unit tests: header token accepted, query key rejected on management
  routes, read scope cannot mutate, constant-time path exercised.
- Middleware tests that management routes 401/403 without a valid token and that
  failures record audit events.
- Manual: exercise a sample management endpoint with/without a valid token over
  localhost.

### Decisions

- `?key=` stays only for read-only, non-critical endpoints. Management endpoints
  require header tokens with scopes, constant-time comparison, and
  localhost-only binding by default.
