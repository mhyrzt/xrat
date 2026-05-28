# parse

Parse and validate config links without persisting to the database.

```bash
xrat parse [input] [flags]
```

## Arguments

| Argument | Description                                                          |
| -------- | -------------------------------------------------------------------- |
| `input`  | Single config URI to parse (optional if using `--file` or `--stdin`) |

## Flags

| Flag                | Description                                                                              |
| ------------------- | ---------------------------------------------------------------------------------------- |
| `--file <path>`     | Read config links (one per line) from a local file                                       |
| `--stdin`           | Read config links (one per line) from stdin                                              |
| `--json`            | Print the generated runtime JSON config for the parsed node                              |
| `--engine <engine>` | Proxy engine for runtime config generation: `auto`, `xray`, `sing-box` (default: `auto`) |

## Engine Modes

| Mode       | Behavior                                              |
| ---------- | ----------------------------------------------------- |
| `auto`     | Uses sing-box for hysteria2, xray for everything else |
| `xray`     | Always use Xray-core (rejects hysteria2)              |
| `sing-box` | Always use sing-box                                   |

## Examples

Parse a single VLESS link:

```bash
xrat parse "vless://uuid@example.com:443?type=ws&security=tls&sni=cdn.example.com#Node"
```

Parse from a file:

```bash
xrat parse --file ./links.txt
```

Parse from stdin:

```bash
cat links.txt | xrat parse --stdin
```

Generate runtime JSON:

```bash
xrat parse --json "vless://uuid@example.com:443?type=tcp#Node"
```

Force sing-box engine:

```bash
xrat parse --engine sing-box "hy2://secret@example.com:443?sni=edge.example.com#HY2"
```

## Output

Without `--json`, prints decoded node fields (protocol, address, port, network,
TLS, SNI, etc.).

With `--json`, prints the full Xray or sing-box runtime configuration JSON that
would be generated for the parsed node.
