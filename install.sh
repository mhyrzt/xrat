#!/usr/bin/env bash
set -euo pipefail

REPO="mhyrzt/xrat"
INSTALL_DIR="$HOME/.local/bin"
BUILD_FROM_SOURCE=""
INSTALL_DESKTOP=1
ASSUME_YES=0
ENABLE_LINGER=0
WORK_DIR="$(mktemp -d)"

RED='\033[0;31m'
YLW='\033[1;33m'
GRN='\033[0;32m'
BLU='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${GRN}[+]${NC} $*"; }
warn()  { echo -e "${YLW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; }
step()  { echo -e "${BLU}[→]${NC} $*"; }

usage() {
    cat <<'EOF'
Usage: install.sh [OPTIONS]

This installer downloads/verifies/places the xrat binary, then hands off to
`xrat setup` for post-install setup (init, daemon, completions, man pages,
and desktop integration).

Options:
  --from-source       Build xrat from this checkout instead of downloading a release
  --install-dir DIR   Binary install directory (default: $HOME/.local/bin)
  --no-desktop        Skip installing the desktop launcher and icons
  --linger            Enable boot-before-login daemon start (Linux; implies daemon)
  -y, --yes           Skip prompts and accept setup defaults
  -h, --help          Show this help

EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --from-source)
                BUILD_FROM_SOURCE=1
                ;;
            --install-dir)
                if [[ $# -lt 2 || -z "$2" ]]; then
                    error "--install-dir requires a directory argument"
                    exit 1
                fi
                INSTALL_DIR="$2"
                shift
                ;;
            --install-dir=*)
                INSTALL_DIR="${1#*=}"
                if [[ -z "$INSTALL_DIR" ]]; then
                    error "--install-dir requires a directory argument"
                    exit 1
                fi
                ;;
            --no-desktop)
                INSTALL_DESKTOP=0
                ;;
            --linger)
                ENABLE_LINGER=1
                ;;
            -y|--yes)
                ASSUME_YES=1
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                echo
                usage
                exit 1
                ;;
        esac
        shift
    done
}

detect_target() {
    case "$(uname -s):$(uname -m)" in
        Linux:x86_64)                 echo "x86_64-unknown-linux-musl" ;;
        Linux:aarch64)                echo "aarch64-unknown-linux-musl" ;;
        Darwin:x86_64)                echo "x86_64-apple-darwin" ;;
        Darwin:arm64|Darwin:aarch64)  echo "aarch64-apple-darwin" ;;
        *)
            error "Unsupported platform: $(uname -s) $(uname -m)"
            exit 1
            ;;
    esac
}

check_cmd() {
    command -v "$1" &>/dev/null
}

require_cmd() {
    if ! check_cmd "$1"; then
        error "Required tool not found: $1"
        exit 1
    fi
}

verify_checksum() {
    local dir="$1" file="$2"
    if check_cmd sha256sum; then
        (cd "$dir" && sha256sum -c "$file")
    elif check_cmd shasum; then
        (cd "$dir" && shasum -a 256 -c "$file")
    else
        error "Need sha256sum or shasum to verify the download."
        exit 1
    fi
}

get_latest_version() {
    local version
    version=$(curl -fsSL -H "User-Agent: xrat-install" "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || true)

    if [[ -z "$version" ]]; then
        error "Could not determine latest release version."
        exit 1
    fi

    echo "$version"
}

download_and_verify() {
    local version="$1" arch="$2"
    local filename="xrat-${version}-${arch}.tar.gz"
    local base_url="https://github.com/${REPO}/releases/download/${version}"

    step "Scurrying to GitHub for ${filename} (^-^)/"
    curl -fsSL --progress-bar -o "${WORK_DIR}/${filename}" "${base_url}/${filename}"

    step "Nibbling the checksum to make sure the archive is clean (o_o)"
    curl -fsSL -o "${WORK_DIR}/SHASUMS256.txt" "${base_url}/SHASUMS256.txt"
    if ! grep -F "${filename}" "${WORK_DIR}/SHASUMS256.txt" > "${WORK_DIR}/checksum.txt"; then
        error "No checksum entry found for ${filename}."
        exit 1
    fi
    verify_checksum "${WORK_DIR}" checksum.txt

    step "Unpacking the tunnel gear (^_^)"
    tar -xzf "${WORK_DIR}/${filename}" -C "${WORK_DIR}"
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    mv "${WORK_DIR}/xrat" "$INSTALL_DIR/xrat"
    chmod +x "$INSTALL_DIR/xrat"
    info "Stashed xrat at ${INSTALL_DIR}/xrat (^_^)"
}

build_from_source() {
    require_cmd cargo

    local repo_dir
    repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    if [[ ! -f "$repo_dir/Cargo.toml" ]]; then
        error "--from-source requires running install.sh directly from the repo (no Cargo.toml found at ${repo_dir})"
        exit 1
    fi

    step "Chewing through Rust crates in ${repo_dir} (o_o)"
    (cd "$repo_dir" && cargo build --release)

    cp "$repo_dir/target/release/xrat" "${WORK_DIR}/xrat"
}

run_setup() {
    local setup_args=("setup")
    [[ "$ASSUME_YES" == "1" ]]     && setup_args+=("-y")
    [[ "$INSTALL_DESKTOP" == "0" ]] && setup_args+=("--no-desktop")
    [[ "$ENABLE_LINGER" == "1" ]]  && setup_args+=("--linger")

    step "Handing off to xrat setup (^_^)"
    echo
    "$INSTALL_DIR/xrat" "${setup_args[@]}"
}

main() {
    trap 'rm -rf "${WORK_DIR}"' EXIT
    parse_args "$@"

    info "Installing xrat - proxy config manager for Xray and sing-box"
    echo

    local os
    os="$(uname -s)"
    case "$os" in
        Linux|Darwin) ;;
        *)
            error "This installer supports Linux and macOS only."
            exit 1
            ;;
    esac

    local version
    if [[ -n "$BUILD_FROM_SOURCE" ]]; then
        build_from_source
        install_binary
        version=$("$INSTALL_DIR/xrat" --version | awk '{print $NF}')
    else
        require_cmd curl
        require_cmd tar

        local target
        target=$(detect_target)

        step "Sniffing out the latest release (^_^)"
        version=$(get_latest_version)
        info "Freshest cheese on GitHub: ${version} (^.^)"
        echo

        download_and_verify "$version" "$target"
        install_binary
    fi

    info "xrat ${version} scampered into place successfully (^_^)"
    echo

    run_setup
}

main "$@"
