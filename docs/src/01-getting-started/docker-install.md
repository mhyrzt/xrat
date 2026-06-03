# Docker Install

Use the Docker image when you want `xrat` and Xray-core in one container. The
image is published to GitHub Container Registry on each tagged release.

For host-level systemd daemon management, shell completions, or man pages, use
the [Installation Script](installation.md) or
[Manual Binary Install](manual-binary-install.md) instead.

## Pull

```bash
docker pull ghcr.io/mhyrzt/xrat:latest
```

For a specific release, use the version tag:

```bash
docker pull ghcr.io/mhyrzt/xrat:0.1.1
```

## State

The container stores all xrat state under `/data/xrat`.

```bash
docker volume create xrat-data
```

Run commands with the volume mounted:

```bash
docker run --rm -it \
  -v xrat-data:/data/xrat \
  ghcr.io/mhyrzt/xrat:latest init
```

## Import and List

```bash
docker run --rm -it \
  -v xrat-data:/data/xrat \
  ghcr.io/mhyrzt/xrat:latest import "https://example.com/sub.txt"

docker run --rm -it \
  -v xrat-data:/data/xrat \
  ghcr.io/mhyrzt/xrat:latest list
```

## Serve the HTTP API

Bind the API to all container interfaces and publish the port on the host:

```bash
docker run --rm -it \
  -v xrat-data:/data/xrat \
  -p 8080:8080 \
  ghcr.io/mhyrzt/xrat:latest serve --host 0.0.0.0 --port 8080
```

## Run a Local Proxy

The image includes Xray-core. Publish the proxy ports you enable in
`config.toml`.

```bash
docker run --rm -it \
  -v xrat-data:/data/xrat \
  -p 1080:1080 \
  ghcr.io/mhyrzt/xrat:latest connect <config-id>
```

The default generated config binds the SOCKS proxy to `127.0.0.1` inside the
container. To reach it from the host through `-p`, set the runtime host to
`0.0.0.0` in `/data/xrat/config.toml`.

```toml
[runtime.socks]
enabled = true
host = "0.0.0.0"
port = 1080
```

## Build Locally

```bash
docker build -t xrat .
docker run --rm -it -v xrat-data:/data/xrat xrat --help
```
