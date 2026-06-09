# Docker Install

Use the Docker image when you want `xrat`, Xray-core, and sing-box in one
container. The image is published to GitHub Container Registry on each tagged
release.

For host-level systemd daemon management, shell completions, or man pages, use
the [Installation Script](installation.md) or
[Manual Binary Install](manual-binary-install.md) instead.

## Pull

```bash
docker pull ghcr.io/mhyrzt/xrat:latest
```

For a specific release, use the version tag:

```bash
docker pull ghcr.io/mhyrzt/xrat:0.1.2
```

## State

The container stores all xrat state under `/data/xrat`.

```bash
docker volume create xrat-data
```

Define a reusable alias so setup and read-only xrat commands run with the volume
mounted:

```bash
alias xrat-docker='docker run --rm -it -v xrat-data:/data/xrat ${XRAT_DOCKER_OPTS:-} ghcr.io/mhyrzt/xrat:latest'
```

Add it to your shell profile if you want it available in new terminals. Then
initialize the data directory:

```bash
xrat-docker init
```

## Import and List

```bash
xrat-docker import "https://example.com/sub.txt"

xrat-docker list
```

## Serve the HTTP API

Bind the API to all container interfaces and publish the generated default API
port on the host:

```bash
XRAT_DOCKER_OPTS="-p 18203:18203"
xrat-docker serve --host 0.0.0.0
```

Then verify it from the host:

```bash
curl http://localhost:18203/health
```

## Run a Local Proxy

`xrat connect` talks to the local daemon IPC socket, so the proxy container must
keep the daemon running while you connect from another command. Publish the
proxy ports you enable in `config.toml`; the generated defaults use SOCKS on
`18200`, HTTP on `18201`, Shadowsocks on `18202`, and the API server on
`18203`.

```bash
docker run -d --name xrat \
  -v xrat-data:/data/xrat \
  -p 127.0.0.1:18200:18200 \
  ghcr.io/mhyrzt/xrat:latest daemon run-server

docker exec -it xrat xrat connect <config-id>
```

The generated config already binds the SOCKS proxy to `0.0.0.0` inside the
container. Keep the Docker publish address restricted to `127.0.0.1` unless you
intentionally want LAN access.

```toml
[runtime.socks]
enabled = true
host = "0.0.0.0"
port = 18200
```

If you also enable the HTTP proxy, Shadowsocks inbound, or API server in
`config.toml`, publish their matching container ports:

```bash
-p 127.0.0.1:18201:18201  # HTTP proxy
-p 127.0.0.1:18202:18202  # Shadowsocks inbound
-p 127.0.0.1:18203:18203  # HTTP API, requires [server].host = "0.0.0.0"
```

Stop the long-running container when finished:

```bash
docker stop xrat
docker rm xrat
```

## Build Locally

```bash
docker build -t xrat .
docker run --rm -it -v xrat-data:/data/xrat xrat --help
```
