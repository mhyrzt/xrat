# IP Scanning

xrat includes a TCP reachability scanner for testing candidate IP addresses,
useful for discovering fast CDN endpoints or Cloudflare edge IPs.

## Overview

The scan command:

1. Reads candidate IPs from CLI flags or a file
2. Attempts TCP connection to each IP on a specified port
3. Measures connection latency
4. Persists results to the database
5. Prints a summary with reachability and latency

## Usage

```bash
xrat scan [flags]
```

### Scan Specific IPs

```bash
xrat scan --ips 1.1.1.1,8.8.8.8,9.9.9.9
```

### Scan from a File

```bash
xrat scan --file ./candidate-ips.txt
```

File format (one IP per line):

```
1.1.1.1
8.8.8.8
9.9.9.9
104.16.0.1
```

### Custom Port and Timeout

```bash
xrat scan --ips 1.1.1.1,8.8.8.8 --port 8443 --timeout 2000
```

### View History

```bash
xrat scan --history 20
```

## Configuration

| Flag             | Description                     | Default |
| ---------------- | ------------------------------- | ------- |
| `--ips <list>`   | Comma-separated IPs to scan     | -       |
| `--file <path>`  | File with newline-separated IPs | -       |
| `--port <port>`  | Target TCP port                 | `443`   |
| `--timeout <ms>` | TCP connect timeout             | `4000`  |
| `--history <n>`  | Print latest N results and exit | -       |

## Scan Process

### Step 1: Load IPs

Read IPs from `--ips` or `--file`:

```rust
let ips = if !args.ips.is_empty() {
    args.ips.clone()
} else if let Some(file) = &args.file {
    std::fs::read_to_string(file)?
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
} else {
    return Err("no IPs provided");
};
```

### Step 2: TCP Connect

For each IP, attempt a TCP connection:

```rust
async fn tcp_connect(ip: &str, port: u16, timeout: Duration) -> Result<Duration> {
    let start = Instant::now();
    let addr = format!("{}:{}", ip, port);

    match timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => Ok(start.elapsed()),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err(Error::Timeout),
    }
}
```

### Step 3: Measure Latency

Record the connection time in milliseconds:

```rust
let latency_ms = start.elapsed().as_millis() as u64;
```

### Step 4: Persist Results

Insert or update the result in `cf_scan_results`:

```sql
INSERT INTO cf_scan_results (ip, latency_ms, error, last_scanned_at)
VALUES (?, ?, ?, ?)
ON CONFLICT(ip) DO UPDATE SET
  latency_ms = excluded.latency_ms,
  error = excluded.error,
  last_scanned_at = excluded.last_scanned_at
```

### Step 5: Print Summary

```
IP              Port    Latency    Status
1.1.1.1         443     12ms       reachable
8.8.8.8         443     15ms       reachable
9.9.9.9         443     -          timeout
104.16.0.1      443     8ms        reachable
```

## Persistence

Scan results are persisted to the `cf_scan_results` table:

| Column            | Type      | Description                         |
| ----------------- | --------- | ----------------------------------- |
| `id`              | INTEGER   | Primary key                         |
| `ip`              | TEXT      | IP address (unique)                 |
| `latency_ms`      | INTEGER   | Connection latency (NULL if failed) |
| `error`           | TEXT      | Error message (NULL if successful)  |
| `last_scanned_at` | TIMESTAMP | Last scan timestamp                 |

### Upsert Behavior

Results are upserted (insert or update on conflict):

- New IP → insert new row
- Existing IP → update latency, error, timestamp

This allows tracking IP reachability over time.

## History

View persisted scan results:

```bash
xrat scan --history 20
```

Output:

```
Scan History (latest 20)
─────────────────────────────────────────────────────
IP              Latency    Last Scanned
1.1.1.1         12ms       2026-05-28 10:30:00
8.8.8.8         15ms       2026-05-28 10:30:00
104.16.0.1      8ms        2026-05-28 10:30:00
9.9.9.9         -          2026-05-28 10:30:00 (timeout)
```

Results are sorted by `last_scanned_at` descending.

## Use Cases

### Cloudflare IP Scanning

Discover fast Cloudflare edge IPs:

```bash
# Generate candidate IPs from Cloudflare ranges
cat > cf-ips.txt <<EOF
1.1.1.1
1.0.0.1
104.16.0.1
104.17.0.1
EOF

# Scan on port 443
xrat scan --file cf-ips.txt --port 443 --timeout 3000

# View results
xrat scan --history 10
```

### CDN Endpoint Testing

Test CDN nodes in your region:

```bash
xrat scan --ips 151.101.1.69,151.101.65.69,151.101.129.69 --port 443
```

### Network Reconnaissance

Discover reachable IPs in a range:

```bash
# Generate IP range
for i in {1..254}; do echo "192.168.1.$i"; done > ips.txt

# Scan on custom port
xrat scan --file ips.txt --port 8443 --timeout 1000
```

## Output Format

### Success

```
IP              Port    Latency    Status
1.1.1.1         443     12ms       reachable
```

### Failure

```
IP              Port    Latency    Status
9.9.9.9         443     -          timeout
```

### Error Types

| Error         | Description                           |
| ------------- | ------------------------------------- |
| `timeout`     | Connection timed out                  |
| `refused`     | Connection refused (port closed)      |
| `unreachable` | Network unreachable                   |
| `dns`         | DNS resolution failed (for hostnames) |
| `io`          | I/O error                             |

## Performance

### Concurrency

Scans are performed sequentially to avoid overwhelming the network. For large IP
lists, consider splitting into multiple runs.

### Timeout

Adjust timeout based on network conditions:

- **Fast network**: `--timeout 2000` (2 seconds)
- **Slow network**: `--timeout 5000` (5 seconds)
- **High latency**: `--timeout 10000` (10 seconds)

## Limitations

- **TCP only**: Does not support UDP or ICMP
- **Single port**: Scans one port at a time
- **No authentication**: Does not test proxy authentication
- **Sequential**: No parallel scanning (yet)

## Related

- [`scan` CLI](../02-cli/scan.md) — command reference
- [Testing](testing.md) — test stored proxy configs (not raw IPs)
- [Database Schema](../05-reference/database-schema.md) — `cf_scan_results`
  table
