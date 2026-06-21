# validate

Validate that an XRAT `config.toml` file exists, parses, and is internally
consistent.

```bash
xrat validate <path> [--format <human|json>]
```

A path is required. The command does not modify anything; it only reports
whether the file is valid.

## Flags

| Flag       | Description                                |
| ---------- | ------------------------------------------ |
| `--format` | Output format: `human` (default) or `json` |

## What it checks

- **Runtime**: engine is one of `xray`, `v2ray`, `sing-box`; rotation
  `test_concurrency` is non-negative; rotation `test_stages` only contains known
  stage names (`icmp`/`ping`, `real_delay`, `download`, and their aliases);
  enabled inbounds have a host and a non-zero, non-duplicated port; SOCKS auth
  and Shadowsocks material are present when enabled.
- **Database**: when `backend = "postgres"`, validates user, database name,
  connection-pool bounds, and connect timeout.
- **Testing**: concurrency is non-negative; `[testing].order` has no duplicate
  stages; enabled probes have valid HTTP/HTTPS URLs and positive timeouts.
- **Server**: when enabled, host is present and any API key is structurally
  valid.

### Secret references

Secret values such as passwords and API keys can be inline literals or
environment references (`{ env = "VAR_NAME" }`). Validation is **structural**: a
literal must be non-empty and an env reference must name a variable, but the
environment variable is **not** required to be set at validation time. Actual
resolution happens at runtime.

## Diagnostics

Each validation error is reported as a diagnostic with four parts: the offending
`field`, the `problem` with its value, the `reason` the constraint matters, and
a `fix` that includes accepted values or ranges. Both `human` and `json` output
carry the same information, so structured consumers get the same repair guidance
as the terminal.

## Examples

```bash
xrat validate config.toml
```

```
OK config.toml is valid.
```

Human output for an invalid config:

```
config.toml has 1 validation error(s):

  [runtime].engine unsupported engine: bad
    why: the engine selects which proxy core generates and runs the runtime config.
    fix: use one of: xray, v2ray, sing-box.
```

Machine-readable output:

```bash
xrat validate --format json config.toml
```

```json
{
  "path": "config.toml",
  "valid": false,
  "errors": [
    {
      "field": "[runtime].engine",
      "problem": "unsupported engine: bad",
      "reason": "the engine selects which proxy core generates and runs the runtime config.",
      "fix": "use one of: xray, v2ray, sing-box."
    }
  ]
}
```

## Exit status

Returns a non-zero exit code when the config is invalid, so it can be used in
scripts and CI.

## Related

- [config management](config-management.md) — edit and inspect the active config
- [init](init.md) — create a starter `config.toml`
