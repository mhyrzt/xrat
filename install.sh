#!/usr/bin/env bash
set -euo pipefail

REPO="mhyrzt/xrat"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BUILD_FROM_SOURCE="${BUILD_FROM_SOURCE:-}"

RED='\033[0;31m'
YLW='\033[1;33m'
GRN='\033[0;32m'
BLU='\033[0;34m'
DIM='\033[2m'
NC='\033[0m'

print_ascii() {
    echo -e "${BLU}"
    cat <<'EOF'
██╗  ██╗██████╗  █████╗ ████████╗
╚██╗██╔╝██╔══██╗██╔══██╗╚══██╔══╝
 ╚███╔╝ ██████╔╝███████║   ██║
 ██╔██╗ ██╔══██╗██╔══██║   ██║
██╔╝ ██╗██║  ██║██║  ██║   ██║
╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝
EOF
    echo -e "${YLW}"
    cat <<'EOF'
        /\___/\
       ( o   ^ )   ~
        > ._. <   /
       /       \_/
      (  )   (  )
EOF
    echo -e "${NC}  installing xrat - squeak squeak"
    echo -e "  Proxy config manager for Xray and sing-box"
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

    step "Extracting binary and extras..."
    tar -xzf "/tmp/${filename}" -C /tmp ./xrat ./man ./completions
    rm "/tmp/${filename}" "/tmp/SHASUMS256.txt"
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    mv /tmp/xrat "$INSTALL_DIR/xrat"
    chmod +x "$INSTALL_DIR/xrat"
    info "Installed to ${INSTALL_DIR}/xrat"
}

build_from_source() {
    require_cmd cargo

    local repo_dir
    repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    if [[ ! -f "$repo_dir/Cargo.toml" ]]; then
        error "BUILD_FROM_SOURCE requires running install.sh directly from the repo (no Cargo.toml found at ${repo_dir})"
        exit 1
    fi

    step "Building xrat from ${repo_dir}..."
    (cd "$repo_dir" && cargo build --release)

    step "Generating man pages and completions..."
    local bin="$repo_dir/target/release/xrat"
    mkdir -p /tmp/man/man1 /tmp/completions
    "$bin" manpage --output /tmp/man/man1
    "$bin" completions bash > /tmp/completions/xrat.bash
    "$bin" completions zsh  > /tmp/completions/_xrat
    "$bin" completions fish > /tmp/completions/xrat.fish

    cp "$bin" /tmp/xrat
}

install_extras() {
    local data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"

    if [[ -d /tmp/man ]]; then
        local man_dir="${data_dir}/man/man1"
        mkdir -p "$man_dir"
        cp /tmp/man/man1/*.1 "$man_dir/" 2>/dev/null || true
        rm -rf /tmp/man
        info "Man pages installed to ${man_dir}"
    fi

    if [[ -d /tmp/completions ]]; then
        local bash_dir="${data_dir}/bash-completion/completions"
        local zsh_dir="${data_dir}/zsh/site-functions"
        local fish_dir="${HOME}/.config/fish/completions"

        [[ -f /tmp/completions/xrat.bash ]] && { mkdir -p "$bash_dir"; cp /tmp/completions/xrat.bash "$bash_dir/xrat"; }
        [[ -f /tmp/completions/_xrat ]]      && { mkdir -p "$zsh_dir";  cp /tmp/completions/_xrat "$zsh_dir/_xrat"; }
        [[ -f /tmp/completions/xrat.fish ]] && { mkdir -p "$fish_dir"; cp /tmp/completions/xrat.fish "$fish_dir/xrat.fish"; }
        rm -rf /tmp/completions
        info "Shell completions installed"
    fi
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

run_linger_enable() {
    local user_name="${USER:-$(id -un)}"
    if ! check_cmd loginctl; then
        warn "loginctl not found; cannot enable boot startup before login automatically."
        echo
        echo "  The daemon will still auto-start when your user session starts."
        echo
        return 0
    fi

    step "Enabling systemd user lingering for ${user_name}..."
    loginctl enable-linger "$user_name"
    info "User lingering enabled. xrat-daemon can start at boot before login."
    echo
    echo "  To disable later:"
    echo -e "  ${DIM}loginctl disable-linger \"${user_name}\"${NC}"
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
    echo "  Rotate proxy:"
    echo -e "    ${DIM}xrat proxy rotate${NC}"
    echo -e "    ${DIM}# requires xrat-daemon; install.sh can install it as a systemd user service${NC}"
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

    check_xray
    check_singbox

    local version
    if [[ -n "$BUILD_FROM_SOURCE" ]]; then
        build_from_source
        install_binary
        install_extras
        version=$("$INSTALL_DIR/xrat" --version | awk '{print $NF}')
    else
        require_cmd curl
        require_cmd tar
        require_cmd sha256sum

        local arch
        arch=$(detect_arch)
        info "Architecture: ${arch}"
        echo

        step "Fetching latest release..."
        version=$(get_latest_version)
        info "Latest version: ${version}"
        echo

        download_and_verify "$version" "$arch"
        install_binary
        install_extras
    fi

    check_path
    echo
    info "xrat ${version} installed successfully."
    echo

    ask "Automatically set up xrat? (init config + optional systemd daemon) [Y/n]"
    read -r do_setup </dev/tty || do_setup=""
    if [[ "${do_setup,,}" != "n" ]]; then
        echo
        run_init
        echo
        ask "Install and start xrat-daemon as a systemd user service? [Y/n]"
        read -r do_daemon </dev/tty || do_daemon=""
        if [[ "${do_daemon,,}" != "n" ]]; then
            run_daemon_install
            echo
            ask "Start xrat-daemon at boot before login / keep it after logout? [y/N]"
            read -r do_linger </dev/tty || do_linger=""
            if [[ "${do_linger,,}" == "y" ]]; then
                run_linger_enable
            else
                user_name="${USER:-$(id -un)}"
                echo
                echo "  The daemon will auto-start when your user session starts."
                echo "  For boot startup before login, run:"
                echo -e "  ${DIM}loginctl enable-linger \"${user_name}\"${NC}"
            fi
        fi
    fi

    show_guide
}

main "$@"
