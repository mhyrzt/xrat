#!/usr/bin/env bash
set -euo pipefail

REPO="mhyrzt/xrat"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

RED='\033[0;31m'
YLW='\033[1;33m'
GRN='\033[0;32m'
BLU='\033[0;34m'
DIM='\033[2m'
NC='\033[0m'

print_ascii() {
    echo -e "${BLU}"
    cat <<'EOF'
  __  ______  ___  ______
  \ \/ / __ \/ _ \/_  __/
   >  < / /_/ / __ |/ /
  /_/\_\____/_/ |_/_/
EOF
    echo -e "${NC}  Proxy config manager for Xray and sing-box"
    echo
}

info()  { echo -e "${GRN}[+]${NC} $*"; }
warn()  { echo -e "${YLW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; }
step()  { echo -e "${BLU}[→]${NC} $*"; }
ask()   { echo -e "${YLW}[?]${NC} $*"; }

detect_arch() {
    case "$(uname -m)" in
        x86_64)  echo "x86_64-unknown-linux-musl" ;;
        aarch64) echo "aarch64-unknown-linux-musl" ;;
        *)
            error "Unsupported architecture: $(uname -m)"
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

check_xray() {
    if check_cmd xray; then
        info "xray: $(command -v xray)"
        return 0
    fi

    error "xray is required but not installed."
    echo
    echo "  Install xray:"
    echo -e "  ${DIM}bash -c \"\$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)\" @ install${NC}"
    echo
    echo "  Or visit: https://github.com/XTLS/Xray-install"
    echo
    exit 1
}

check_singbox() {
    if check_cmd sing-box; then
        info "sing-box: $(command -v sing-box)"
        return 0
    fi

    warn "sing-box is not installed (optional — needed for sing-box config support)."
    echo
    echo "  Install sing-box:"
    echo -e "  ${DIM}curl -fsSL https://sing-box.app/install.sh | sh${NC}"
    echo
}

get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
}

download_and_verify() {
    local version="$1" arch="$2"
    local filename="xrat-${version}-${arch}.tar.gz"
    local base_url="https://github.com/${REPO}/releases/download/${version}"

    step "Downloading ${filename}..."
    curl -fsSL --progress-bar -o "/tmp/${filename}" "${base_url}/${filename}"

    step "Verifying checksum..."
    curl -fsSL -o "/tmp/SHASUMS256.txt" "${base_url}/SHASUMS256.txt"
    (cd /tmp && grep "${filename}" SHASUMS256.txt | sha256sum -c -)

    step "Extracting binary..."
    tar -xzf "/tmp/${filename}" -C /tmp ./xrat
    rm "/tmp/${filename}" "/tmp/SHASUMS256.txt"
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    mv /tmp/xrat "$INSTALL_DIR/xrat"
    chmod +x "$INSTALL_DIR/xrat"
    info "Installed to ${INSTALL_DIR}/xrat"
}

check_path() {
    if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
        warn "${INSTALL_DIR} is not in PATH."
        echo
        echo "  Add to ~/.bashrc or ~/.zshrc:"
        echo -e "  ${DIM}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
        echo
    fi
}

run_init() {
    step "Initializing xrat config directory..."
    "$INSTALL_DIR/xrat" init
}

run_daemon_install() {
    step "Installing xrat-daemon systemd user service..."
    "$INSTALL_DIR/xrat" daemon install --start
    info "Daemon installed and started."
}

show_guide() {
    echo
    echo -e "${GRN}Quick start:${NC}"
    echo
    echo "  Import a subscription:"
    echo -e "    ${DIM}xrat import <url>${NC}"
    echo
    echo "  List configs:"
    echo -e "    ${DIM}xrat list configs${NC}"
    echo
    echo "  Test connectivity:"
    echo -e "    ${DIM}xrat test${NC}"
    echo
    echo "  Connect:"
    echo -e "    ${DIM}xrat connect <id>${NC}"
    echo
    echo "  Launch TUI:"
    echo -e "    ${DIM}xrat tui${NC}"
    echo
    echo -e "  Docs: ${BLU}https://mhyrzt.github.io/xrat/${NC}"
    echo
}

main() {
    print_ascii

    if [[ "$(uname -s)" != "Linux" ]]; then
        error "This installer supports Linux only."
        exit 1
    fi

    require_cmd curl
    require_cmd tar
    require_cmd sha256sum

    local arch
    arch=$(detect_arch)
    info "Architecture: ${arch}"
    echo

    check_xray
    check_singbox

    step "Fetching latest release..."
    local version
    version=$(get_latest_version)
    info "Latest version: ${version}"
    echo

    download_and_verify "$version" "$arch"
    install_binary
    check_path

    echo
    info "xrat ${version} installed successfully."
    echo

    ask "Automatically set up xrat? (init config + optional systemd daemon) [Y/n]"
    read -r do_setup
    if [[ "${do_setup,,}" != "n" ]]; then
        echo
        run_init
        echo
        ask "Install xrat-daemon as a systemd user service? [y/N]"
        read -r do_daemon
        if [[ "${do_daemon,,}" == "y" ]]; then
            run_daemon_install
        fi
    fi

    show_guide
}

main "$@"
