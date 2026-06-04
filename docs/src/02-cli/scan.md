# scan

Scan candidate IPs for TCP reachability and persist results.

```bash
xrat scan [flags]
```

## Flags

| Flag                | Description                                                         |
| ------------------- | ------------------------------------------------------------------- |
| `--ips <ips>`       | Comma-separated IPs to scan, e.g. `1.1.1.1,8.8.8.8`                 |
| `--file <path>`     | Read newline-separated IPs from a file                              |
| `--port <port>`     | Target TCP port (default: `443`)                                    |
| `--timeout <ms>`    | TCP connect timeout in milliseconds (default: `4000`)               |
| `--history <limit>` | Print the latest N persisted scan results and exit (skips scanning) |
| `--format <format>` | Output format for `--history`: `table`, `tsv`, `json` (default: `table`) |

## Examples

Scan specific IPs:

```bash
xrat scan --ips 1.1.1.1,8.8.8.8,9.9.9.9
```

Scan from a file:

```bash
xrat scan --file ./candidate-ips.txt
```

Scan on a custom port with shorter timeout:

```bash
xrat scan --ips 1.1.1.1,8.8.8.8 --port 8443 --timeout 2000
```

View scan history:

```bash
xrat scan --history 20
xrat scan --history 20 --format json
```

## Behavior

1. Reads candidate IPs from `--ips` or `--file`
2. Attempts TCP connection to each IP on the specified port
3. Measures connection latency
4. Persists results to the `cf_scan_results` table (upsert)
5. Prints scan results with latency and success/failure status

## Use Cases

- **Cloudflare IP scanning**: Test candidate Cloudflare edge IPs for low latency
- **CDN endpoint testing**: Identify fast CDN nodes in your region
- **Network reconnaissance**: Discover reachable IPs in a range

## Output

Normal scans print a concise persistence summary. `--history` prints an aligned
table by default; use `--format tsv` or `--format json` for scripts.

## Persistence

Results are persisted to the database and can be retrieved with `--history`.
This allows tracking IP reachability over time and identifying consistently fast
endpoints.

## Related

- [`test`](test.md) — test stored proxy configs (not raw IPs)
