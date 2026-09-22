# syntax=docker/dockerfile:1

ARG XRAY_TAG=latest
ARG SINGBOX_TAG=latest

FROM rust:1-alpine AS builder

ARG TARGETARCH
WORKDIR /app

RUN apk add --no-cache build-base musl-dev

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY packaging ./packaging
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,id=xrat-target-${TARGETARCH},sharing=locked \
    cargo build --release --locked \
    && cp target/release/xrat /tmp/xrat


FROM teddysun/xray:${XRAY_TAG} AS xray
FROM ghcr.io/sagernet/sing-box:${SINGBOX_TAG} AS singbox


FROM alpine:3.22 AS runtime

RUN apk add --no-cache ca-certificates iputils \
    && addgroup -S xrat \
    && adduser -S -G xrat -h /home/xrat xrat \
    && mkdir -p /data/xrat \
    && chown -R xrat:xrat /data/xrat /home/xrat

COPY --from=xray /usr/bin/xray /usr/local/bin/xray
COPY --from=xray /usr/share/xray /usr/local/share/xray
COPY --from=singbox /usr/local/bin/sing-box /usr/local/bin/sing-box

ENV HOME=/home/xrat
ENV XRAT_PATH=/data/xrat 
ENV XRAY_LOCATION_ASSET=/usr/local/share/xray

USER xrat
WORKDIR /data/xrat
VOLUME ["/data/xrat"]
EXPOSE 18200 18201 18202 18203

ENTRYPOINT ["xrat"]
CMD ["--help"]


FROM runtime AS release
ARG TARGETARCH
COPY --chmod=755 image-bin/${TARGETARCH}/xrat /usr/local/bin/xrat


FROM runtime AS local
COPY --from=builder /tmp/xrat /usr/local/bin/xrat
