# syntax=docker/dockerfile:1

FROM rust:1-alpine AS builder

WORKDIR /app

RUN apk add --no-cache build-base musl-dev

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY packaging ./packaging
COPY src ./src

RUN cargo build --release --locked

FROM alpine:3.22 AS xray

ARG TARGETARCH
ARG XRAY_VERSION=latest

RUN apk add --no-cache ca-certificates unzip wget \
    && case "$TARGETARCH" in \
        amd64) asset="Xray-linux-64.zip" ;; \
        arm64) asset="Xray-linux-arm64-v8a.zip" ;; \
        *) echo "unsupported target architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && if [ "$XRAY_VERSION" = "latest" ]; then \
        url="https://github.com/XTLS/Xray-core/releases/latest/download/${asset}"; \
    else \
        url="https://github.com/XTLS/Xray-core/releases/download/${XRAY_VERSION}/${asset}"; \
    fi \
    && wget -O /tmp/xray.zip "$url" \
    && mkdir -p /tmp/xray \
    && unzip /tmp/xray.zip -d /tmp/xray \
    && install -Dm755 /tmp/xray/xray /usr/local/bin/xray \
    && install -Dm644 /tmp/xray/geoip.dat /usr/local/share/xray/geoip.dat \
    && install -Dm644 /tmp/xray/geosite.dat /usr/local/share/xray/geosite.dat

FROM alpine:3.22

RUN apk add --no-cache ca-certificates iputils \
    && addgroup -S xrat \
    && adduser -S -G xrat -h /home/xrat xrat \
    && mkdir -p /data/xrat \
    && chown -R xrat:xrat /data/xrat /home/xrat

COPY --from=builder /app/target/release/xrat /usr/local/bin/xrat
COPY --from=xray /usr/local/bin/xray /usr/local/bin/xray
COPY --from=xray /usr/local/share/xray /usr/local/share/xray

ENV HOME=/home/xrat \
    XRAT_PATH=/data/xrat \
    XRAY_LOCATION_ASSET=/usr/local/share/xray

USER xrat
WORKDIR /data/xrat
VOLUME ["/data/xrat"]
EXPOSE 8080 1080

ENTRYPOINT ["xrat"]
CMD ["--help"]
