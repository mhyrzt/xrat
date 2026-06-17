# Add Runtime Outbound Pool Mode

**Type:** Feature **Priority:** High **Area:** Runtime / Config Generation /
Proxy Selection **Project:** XRAT

### Context

XRAT already focuses on importing, storing, testing, ranking, rotating, and
running Xray-compatible proxy configurations from the terminal. The README also
lists runtime management, proxy testing, ranking, automatic rotation on
schedule/health failure, and managed Xray/V2Ray local sessions as current
capabilities. ([GitHub][1])

Currently, the runtime flow is mainly centered around running a selected stored
config or rotating active configs. This task adds a higher-level mode where XRAT
can build a **single managed runtime config** containing multiple stored proxy
outbounds and route all traffic through an outbound selection group.

---

## Goal

Allow users to run multiple available proxy configs as one logical outbound
pool, without manually writing Xray/sing-box routing rules.

The user-facing mental model should be:

```text
xrat selects N usable configs
xrat generates one runtime config
local browser/app connects to one local inbound
runtime chooses among proxy outbounds automatically
```

---

## User Story

As an XRAT user, I want to select a set of tested proxy configs and run them as
one outbound pool, so that traffic can automatically use the best/available
proxy without me manually defining routing rules.

---

## Proposed CLI

```bash
xrat run-pool
```

Possible options:

```bash
xrat run-pool \
  --engine xray \
  --limit 10 \
  --strategy least-ping \
  --inbound socks://127.0.0.1:10808
```

Alternative integrated form:

```bash
xrat run --pool --strategy least-ping --limit 10
```

Suggested strategies:

```text
least-ping       choose best healthy proxy by latency
round-robin      distribute connections across selected proxies
random           randomly pick among healthy proxies
priority         prefer configs in rank order, fallback on failure
```

---

## Expected Behavior

XRAT should:

1. Select candidate configs from the database.
2. Filter out disabled, deleted, or recently failed configs.
3. Prefer configs with successful recent test history.
4. Generate a temporary runtime config containing:
   - one local inbound
   - multiple proxy outbounds
   - one outbound group / balancer
   - one catch-all default route into that group

5. Start and supervise the runtime like existing managed sessions.
6. Show active pool status in `xrat status`.
7. Log selected candidates, rejected candidates, strategy, and runtime health
   events.

---

## Implementation Notes

### For Xray backend

Generate multiple outbounds and one `routing.balancers` entry.

Xray routing docs say that when no routing rule matches, traffic goes through
the first outbound by default; when a rule points to a load balancer, Xray
selects an outbound through that load balancer. ([XTLS][2])

So XRAT should not rely on “multiple outbounds exist” alone. It should generate
a catch-all routing rule like:

```json
{
  "type": "field",
  "network": "tcp,udp",
  "balancerTag": "xrat-pool"
}
```

Conceptual generated Xray structure:

```json
{
  "inbounds": [
    {
      "tag": "xrat-in",
      "listen": "127.0.0.1",
      "port": 10808,
      "protocol": "socks"
    }
  ],
  "outbounds": [
    {
      "tag": "proxy-1",
      "protocol": "vless",
      "settings": {}
    },
    {
      "tag": "proxy-2",
      "protocol": "trojan",
      "settings": {}
    },
    {
      "tag": "proxy-3",
      "protocol": "vmess",
      "settings": {}
    },
    {
      "tag": "direct",
      "protocol": "freedom"
    }
  ],
  "routing": {
    "rules": [
      {
        "type": "field",
        "network": "tcp,udp",
        "balancerTag": "xrat-pool"
      }
    ],
    "balancers": [
      {
        "tag": "xrat-pool",
        "selector": ["proxy-"],
        "strategy": {
          "type": "leastPing"
        },
        "fallbackTag": "direct"
      }
    ]
  }
}
```

### For sing-box backend

Generate a `urltest` outbound and set it as the route final outbound.

sing-box `urltest` accepts a list of outbound tags and tests them against a URL;
if the URL is empty, it uses `https://www.gstatic.com/generate_204` by default.
([Sing Box][3])

Conceptual generated sing-box structure:

```json
{
  "inbounds": [
    {
      "type": "mixed",
      "tag": "xrat-in",
      "listen": "127.0.0.1",
      "listen_port": 10808
    }
  ],
  "outbounds": [
    {
      "type": "urltest",
      "tag": "xrat-pool",
      "outbounds": ["proxy-1", "proxy-2", "proxy-3"],
      "url": "https://www.gstatic.com/generate_204",
      "interval": "3m",
      "tolerance": 50
    },
    {
      "type": "vless",
      "tag": "proxy-1"
    },
    {
      "type": "trojan",
      "tag": "proxy-2"
    },
    {
      "type": "vmess",
      "tag": "proxy-3"
    }
  ],
  "route": {
    "final": "xrat-pool"
  }
}
```

---

## Important Design Decision

Do **not** expose this as “routing configuration” to the user.

From UX perspective, this should feel like:

```bash
xrat run-pool --strategy least-ping
```

not:

```bash
xrat generate-routing-balancer-selector-observatory-config ...
```

Internally it uses routing/balancer/urltest, but externally it is just **pool
mode**.

---

## Acceptance Criteria

### Functional

- User can start a runtime pool from multiple stored configs.
- User can choose at least one strategy: `least-ping` or `round-robin`.
- XRAT refuses to start pool mode if fewer than 2 valid configs are available,
  unless `--allow-single` is passed.
- Generated runtime config contains all selected proxy configs as separate
  tagged outbounds.
- Runtime traffic goes through the pool, not simply the first outbound.
- `xrat status` shows:
  - pool mode enabled
  - engine: xray/sing-box
  - selected strategy
  - number of candidate outbounds
  - active local inbound address

- `xrat logs` includes pool selection and runtime health events.

### Safety / Reliability

- Disabled or soft-deleted configs are excluded.
- Recently failed configs are excluded unless `--include-failed` is passed.
- Duplicate outbound tags are impossible; XRAT generates stable unique tags
  like:

```text
proxy-<config_id>
```

- Generated config passes engine validation before runtime starts.
- If runtime startup fails, XRAT prints the generated config path for debugging.

### Tests

Add tests for:

- pool candidate selection
- tag generation
- Xray pool config generation
- sing-box pool config generation
- empty candidate set
- single candidate set
- duplicate protocol/server cases
- strategy mapping

---

## Non-Goals for First Version

Do not implement full per-domain routing in this task.

Out of scope:

```text
YouTube → proxy A
Telegram → proxy B
Iranian sites → direct
ads → block
```

This task is only:

```text
all traffic → outbound pool
```

Also, do not promise exact per-request ordered retry unless XRAT implements that
at supervisor level. Xray/sing-box selection groups can choose
healthy/best/round-robin outbounds, but “try proxy-1 for the same request, then
retry proxy-2 for that exact same request” is a stricter behavior and should be
tracked separately.

---

## Suggested Follow-up Task

### Add Strict Priority Fallback Mode

Implement a strategy where XRAT prefers the highest-ranked config and only
switches to the next config after runtime health failure.

This can reuse XRAT’s existing rotation/health-supervision model instead of
relying fully on engine-native balancers. XRAT already advertises automatic
rotation on schedule or health failure, so strict priority fallback may fit
naturally as a runtime-supervisor feature. ([GitHub][1])

[1]:
  https://github.com/mhyrzt/xrat/blob/master/README.md
  "xrat/README.md at master · mhyrzt/xrat · GitHub"
[2]:
  https://xtls.github.io/en/config/routing.html?utm_source=chatgpt.com
  "Routing"
[3]:
  https://sing-box.sagernet.org/configuration/outbound/urltest/?utm_source=chatgpt.com
  "URLTest - sing-box"
