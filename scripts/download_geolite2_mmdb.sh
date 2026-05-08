#!/usr/bin/env bash
set -euo pipefail

# Download a GeoLite2 MMDB file into XRAT's geoip directory.
# Optional env:
#   XRAT_PATH (default: $HOME/.config/xrat)
#   GEOIP_EDITION (default: GeoLite2-Country)
#     valid examples: GeoLite2-Country, GeoLite2-City, GeoLite2-ASN

ROOT_DIR="${XRAT_PATH:-${HOME}/.config/xrat}"
GEOIP_DIR="${ROOT_DIR}/geoip"
EDITION="${GEOIP_EDITION:-GeoLite2-Country}"

case "${EDITION}" in
  GeoLite2-Country|GeoLite2-City|GeoLite2-ASN) ;;
  *)
    echo "error: unsupported GEOIP_EDITION '${EDITION}'" >&2
    echo "supported: GeoLite2-Country, GeoLite2-City, GeoLite2-ASN" >&2
    exit 1
    ;;
esac

mkdir -p "${GEOIP_DIR}"

URL="https://github.com/P3TERX/GeoLite.mmdb/releases/latest/download/${EDITION}.mmdb"
TMP_DIR="$(mktemp -d)"
MMDB_PATH="${TMP_DIR}/${EDITION}.mmdb"

cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

echo "Downloading ${EDITION}..."
curl -fsSL "${URL}" -o "${MMDB_PATH}"

if [[ ! -s "${MMDB_PATH}" ]]; then
  echo "error: download failed or returned empty file" >&2
  exit 1
fi

DEST_PATH="${GEOIP_DIR}/${EDITION}.mmdb"
install -m 0644 "${MMDB_PATH}" "${DEST_PATH}"

echo "Installed: ${DEST_PATH}"
