ARG XRAY_TAG=latest
ARG SINGBOX_TAG=latest

FROM rust:1-alpine AS builder

WORKDIR /app

RUN apk add --no-cache build-base musl-dev

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY packaging ./packaging
COPY src ./src

RUN cargo build --release --locked


FROM teddysun/xray:${XRAY_TAG} AS xray
FROM ghcr.io/sagernet/sing-box:${SINGBOX_TAG} AS singbox


FROM alpine:3.22

RUN apk add --no-cache ca-certificates iputils \
    && addgroup -S xrat \
    && adduser -S -G xrat -h /home/xrat xrat \
    && mkdir -p /data/xrat \
    && chown -R xrat:xrat /data/xrat /home/xrat

COPY --from=xray /usr/bin/xray /usr/local/bin/xray
COPY --from=xray /usr/share/xray /usr/local/share/xray
COPY --from=singbox /usr/local/bin/sing-box /usr/local/bin/sing-box
COPY --from=builder /app/target/release/xrat /usr/local/bin/xrat

ENV HOME=/home/xrat
ENV XRAT_PATH=/data/xrat 
ENV XRAY_LOCATION_ASSET=/usr/local/share/xray

USER xrat
WORKDIR /data/xrat
VOLUME ["/data/xrat"]
EXPOSE 18200 18201 18202 18203

ENTRYPOINT ["xrat"]
CMD ["--help"]
