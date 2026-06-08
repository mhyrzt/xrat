#!/usr/bin/env bash
set -euo pipefail

REPO="mhyrzt/xrat"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BUILD_FROM_SOURCE="${BUILD_FROM_SOURCE:-}"
INSTALL_DESKTOP="${INSTALL_DESKTOP:-1}"
ASSUME_YES=0
WORK_DIR="$(mktemp -d)"

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
    echo -e "${NC}  installing xrat - the rat that tunnels for you"
    echo -e "  Proxy config manager for Xray and sing-box"
    echo
}

info()  { echo -e "${GRN}[+]${NC} $*"; }
warn()  { echo -e "${YLW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*" >&2; }
step()  { echo -e "${BLU}[→]${NC} $*"; }
ask()   { echo -e "${YLW}[?]${NC} $*"; }

usage() {
    cat <<'EOF'
Usage: install.sh [OPTIONS]

Options:
  --from-source    Build xrat from this checkout instead of downloading a release
  -y, --yes        Skip prompts and answer yes to setup, daemon, and linger
  -h, --help       Show this help

Environment:
  INSTALL_DIR      Binary install directory (default: $HOME/.local/bin)
  INSTALL_DESKTOP  Install desktop launcher and icons (default: 1, set 0 to skip)
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --from-source)
                BUILD_FROM_SOURCE=1
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

detect_os() {
    local os_name
    os_name="$(uname -s)"

    if [[ -r /etc/os-release ]]; then
        local pretty_name=""
        . /etc/os-release
        pretty_name="${PRETTY_NAME:-${NAME:-}}"

        if [[ -n "$pretty_name" ]]; then
            echo "${pretty_name} (${os_name}, kernel $(uname -r))"
            return 0
        fi
    fi

    echo "${os_name} (kernel $(uname -r))"
}

extract_semver() {
    awk '
        match($0, /[0-9]+(\.[0-9]+)+/) {
            print "v" substr($0, RSTART, RLENGTH)
            exit
        }
    '
}

binary_version() {
    local cmd="$1"
    local output version

    output=$("$cmd" version 2>&1 || true)
    version=$(printf '%s\n' "$output" | extract_semver)

    if [[ -n "$version" ]]; then
        echo "$version"
    else
        echo "version unavailable"
    fi
}

detect_shell() {
    local shell_path="${SHELL:-}"
    local shell_name version_output version

    shell_name="$(basename "${shell_path:-unknown}")"
    case "$shell_name" in
        bash|zsh|fish)
            version_output=$("$shell_path" --version 2>&1 || true)
            version=$(printf '%s\n' "$version_output" | extract_semver)
            ;;
        *)
            version=""
            ;;
    esac

    if [[ -n "$version" ]]; then
        echo "${shell_name} ${version}"
    else
        echo "$shell_name"
    fi
}

print_detection() {
    local os_info machine arch shell_info
    os_info="$(detect_os)"
    machine="$(uname -m)"
    arch="$(detect_arch)"
    shell_info="$(detect_shell)"

    info "OS: ${os_info}"
    info "CPU architecture: ${machine} (${arch})"
    info "Shell: ${shell_info}"
    echo
}

check_cmd() {
    command -v "$1" &>/dev/null
}

desktop_session_type() {
    printf '%s' "${XDG_SESSION_TYPE:-unknown}" | tr '[:upper:]' '[:lower:]'
}

select_desktop_terminal() {
    local session_type
    session_type="$(desktop_session_type)"

    if [[ "$session_type" == "wayland" ]]; then
        for terminal in kitty alacritty wezterm footclient foot konsole; do
            if check_cmd "$terminal"; then
                echo "$terminal"
                return 0
            fi
        done
    else
        for terminal in kitty alacritty wezterm konsole gnome-terminal xterm; do
            if check_cmd "$terminal"; then
                echo "$terminal"
                return 0
            fi
        done
    fi

    echo ""
}

write_desktop_launcher() {
    local terminal="$1"
    local launcher_path="${INSTALL_DIR}/xrat-desktop"
    local xrat_path="${INSTALL_DIR}/xrat"

    case "$terminal" in
        kitty)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec kitty --class=xrat --title=XRAT "$xrat_path" tui "\$@"
EOF
            ;;
        alacritty)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec alacritty --class xrat,xrat --title XRAT -e "$xrat_path" tui "\$@"
EOF
            ;;
        wezterm)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec wezterm start --always-new-process --class xrat "$xrat_path" tui "\$@"
EOF
            ;;
        footclient)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec footclient --app-id=xrat --title=XRAT "$xrat_path" tui "\$@"
EOF
            ;;
        foot)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec foot --app-id=xrat --title=XRAT "$xrat_path" tui "\$@"
EOF
            ;;
        konsole)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec konsole --desktopfile xrat -e "$xrat_path" tui "\$@"
EOF
            ;;
        gnome-terminal)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec gnome-terminal --class=xrat --title=XRAT -- "$xrat_path" tui "\$@"
EOF
            ;;
        xterm)
            cat > "$launcher_path" <<EOF
#!/usr/bin/env sh
exec xterm -class xrat -title XRAT -e "$xrat_path" tui "\$@"
EOF
            ;;
        *)
            return 1
            ;;
    esac

    chmod +x "$launcher_path"
}

require_cmd() {
    if ! check_cmd "$1"; then
        error "Required tool not found: $1"
        exit 1
    fi
}

check_xray() {
    if check_cmd xray; then
        local xray_path
        xray_path="$(command -v xray)"
        info "xray: ${xray_path} ($(binary_version "$xray_path"))"
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
        local singbox_path
        singbox_path="$(command -v sing-box)"
        info "sing-box: ${singbox_path} ($(binary_version "$singbox_path"))"
        return 0
    fi

    warn "sing-box is not installed (optional — needed for sing-box config support)."
    echo
    echo "  Install sing-box:"
    echo -e "  ${DIM}curl -fsSL https://sing-box.app/install.sh | sh${NC}"
    echo
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

    step "Downloading ${filename}..."
    curl -fsSL --progress-bar -o "${WORK_DIR}/${filename}" "${base_url}/${filename}"

    step "Verifying checksum..."
    curl -fsSL -o "${WORK_DIR}/SHASUMS256.txt" "${base_url}/SHASUMS256.txt"
    if ! grep -F "${filename}" "${WORK_DIR}/SHASUMS256.txt" > "${WORK_DIR}/checksum.txt"; then
        error "No checksum entry found for ${filename}."
        exit 1
    fi
    (cd "${WORK_DIR}" && sha256sum -c checksum.txt)

    step "Extracting binary and extras..."
    tar -xzf "${WORK_DIR}/${filename}" -C "${WORK_DIR}"
}

install_binary() {
    mkdir -p "$INSTALL_DIR"
    mv "${WORK_DIR}/xrat" "$INSTALL_DIR/xrat"
    chmod +x "$INSTALL_DIR/xrat"
    cat > "$INSTALL_DIR/xratui" <<EOF
#!/usr/bin/env sh
exec "$INSTALL_DIR/xrat" tui "\$@"
EOF
    chmod +x "$INSTALL_DIR/xratui"
    info "Installed to ${INSTALL_DIR}/xrat"
    info "Installed TUI shortcut to ${INSTALL_DIR}/xratui"
}

build_from_source() {
    require_cmd cargo

    local repo_dir
    repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

    if [[ ! -f "$repo_dir/Cargo.toml" ]]; then
        error "--from-source requires running install.sh directly from the repo (no Cargo.toml found at ${repo_dir})"
        exit 1
    fi

    step "Building xrat from ${repo_dir}..."
    (cd "$repo_dir" && cargo build --release)

    step "Generating man pages and completions..."
    local bin="$repo_dir/target/release/xrat"
    mkdir -p "${WORK_DIR}/man/man1" "${WORK_DIR}/completions"
    "$bin" manpage --output "${WORK_DIR}/man/man1" >/dev/null
    "$bin" completions bash > "${WORK_DIR}/completions/xrat.bash"
    "$bin" completions zsh  > "${WORK_DIR}/completions/_xrat"
    "$bin" completions fish > "${WORK_DIR}/completions/xrat.fish"

    if [[ -f "$repo_dir/packaging/desktop/xrat.desktop" ]]; then
        mkdir -p "${WORK_DIR}/desktop"
        cp "$repo_dir/packaging/desktop/xrat.desktop" "${WORK_DIR}/desktop/xrat.desktop"
    fi

    if [[ -d "$repo_dir/docs/src/media/icons" ]]; then
        mkdir -p "${WORK_DIR}/desktop/icons"
        cp "$repo_dir/docs/src/media/icons/xrat-icon-48x48.png" "${WORK_DIR}/desktop/icons/xrat-48x48.png"
        cp "$repo_dir/docs/src/media/icons/xrat-icon-256x256.png" "${WORK_DIR}/desktop/icons/xrat-256x256.png"
    fi

    cp "$bin" "${WORK_DIR}/xrat"
}

install_extras() {
    local data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"

    if [[ -d "${WORK_DIR}/man" ]]; then
        local man_dir="${data_dir}/man/man1"
        mkdir -p "$man_dir"
        rm -f "$man_dir/xrat.1" "$man_dir"/xrat-*.1
        cp "${WORK_DIR}"/man/man1/*.1 "$man_dir/" 2>/dev/null || true
        info "Man pages installed to ${man_dir}"
    fi

    if [[ -d "${WORK_DIR}/completions" ]]; then
        local bash_dir="${data_dir}/bash-completion/completions"
        local zsh_dir="${data_dir}/zsh/site-functions"
        local fish_dir="${HOME}/.config/fish/completions"

        [[ -f "${WORK_DIR}/completions/xrat.bash" ]] && { mkdir -p "$bash_dir"; cp "${WORK_DIR}/completions/xrat.bash" "$bash_dir/xrat"; }
        [[ -f "${WORK_DIR}/completions/_xrat" ]]      && { mkdir -p "$zsh_dir";  cp "${WORK_DIR}/completions/_xrat" "$zsh_dir/_xrat"; }
        [[ -f "${WORK_DIR}/completions/xrat.fish" ]] && { mkdir -p "$fish_dir"; cp "${WORK_DIR}/completions/xrat.fish" "$fish_dir/xrat.fish"; }
        info "Shell completions installed"
    fi

    if [[ "$INSTALL_DESKTOP" != "0" && -d "${WORK_DIR}/desktop" ]]; then
        local apps_dir="${data_dir}/applications"
        local icon_root="${data_dir}/icons/hicolor"
        local icon_48_dir="${icon_root}/48x48/apps"
        local icon_256_dir="${icon_root}/256x256/apps"

        if [[ -f "${WORK_DIR}/desktop/xrat.desktop" ]]; then
            mkdir -p "$apps_dir"
            local desktop_terminal=""
            local exec_path="${INSTALL_DIR}/xrat tui"
            local terminal_value="true"
            local startup_wm_class=""

            desktop_terminal="$(select_desktop_terminal)"
            if [[ -n "$desktop_terminal" ]] && write_desktop_launcher "$desktop_terminal"; then
                exec_path="${INSTALL_DIR}/xrat-desktop"
                terminal_value="false"
                startup_wm_class="xrat"
                info "Desktop launcher will use ${desktop_terminal} with xrat window identity"
            else
                warn "No supported terminal found for desktop window identity; taskbar may show the terminal icon."
            fi

            awk -v exec_path="$exec_path" -v terminal_value="$terminal_value" -v startup_wm_class="$startup_wm_class" '
                BEGIN { saw_startup_wm_class = 0 }
                /^Exec=/ { print "Exec=" exec_path; next }
                /^Terminal=/ { print "Terminal=" terminal_value; next }
                /^StartupWMClass=/ {
                    if (startup_wm_class != "") {
                        print "StartupWMClass=" startup_wm_class
                        saw_startup_wm_class = 1
                    }
                    next
                }
                /^StartupNotify=/ {
                    if (startup_wm_class != "" && saw_startup_wm_class == 0) {
                        print "StartupWMClass=" startup_wm_class
                        saw_startup_wm_class = 1
                    }
                    print
                    next
                }
                { print }
                END {
                    if (startup_wm_class != "" && saw_startup_wm_class == 0) {
                        print "StartupWMClass=" startup_wm_class
                    }
                }
            ' "${WORK_DIR}/desktop/xrat.desktop" > "$apps_dir/xrat.desktop"
        fi
        [[ -f "${WORK_DIR}/desktop/icons/xrat-48x48.png" ]] && { mkdir -p "$icon_48_dir"; cp "${WORK_DIR}/desktop/icons/xrat-48x48.png" "$icon_48_dir/xrat.png"; }
        [[ -f "${WORK_DIR}/desktop/icons/xrat-256x256.png" ]] && { mkdir -p "$icon_256_dir"; cp "${WORK_DIR}/desktop/icons/xrat-256x256.png" "$icon_256_dir/xrat.png"; }
        check_cmd update-desktop-database && update-desktop-database "$apps_dir" >/dev/null 2>&1 || true
        check_cmd gtk-update-icon-cache && gtk-update-icon-cache -f -t "$icon_root" >/dev/null 2>&1 || true
        info "Desktop launcher installed"
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

prompt_yes_no() {
    local prompt="$1" default="$2" answer

    if [[ "$ASSUME_YES" == "1" ]]; then
        echo "y"
        return 0
    fi

    if ! { : </dev/tty; } 2>/dev/null; then
        echo "n"
        return 0
    fi

    ask "$prompt" >/dev/tty
    read -r answer </dev/tty || answer=""
    answer="${answer,,}"

    if [[ -z "$answer" ]]; then
        echo "$default"
    else
        echo "$answer"
    fi
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
    echo -e "    ${DIM}xrat connect <ref>${NC}"
    echo
    echo "  Rotate proxy:"
    echo -e "    ${DIM}xrat rotate now${NC}"
    echo -e "    ${DIM}xrat rotate status${NC}"
    echo -e "    ${DIM}# requires xrat-daemon; install.sh can install it as a systemd user service${NC}"
    echo
    echo "  Proxy helpers:"
    echo -e "    ${DIM}xrat proxy endpoints${NC}"
    echo -e "    ${DIM}eval \"\$(xrat proxy shell toggle)\"${NC}"
    echo -e "    ${DIM}xrat proxy desktop toggle${NC}"
    echo
    echo "  Launch TUI:"
    echo -e "    ${DIM}xrat tui${NC}"
    echo -e "    ${DIM}xratui${NC}"
    echo
    echo -e "  Docs: ${BLU}https://mhyrzt.github.io/xrat/${NC}"
    echo
}

main() {
    trap 'rm -rf "${WORK_DIR}"' EXIT
    parse_args "$@"

    print_ascii

    if [[ "$(uname -s)" != "Linux" ]]; then
        error "This installer supports Linux only."
        exit 1
    fi

    print_detection
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

    do_setup=$(prompt_yes_no "Automatically set up xrat? (init config + optional systemd daemon) [Y/n]" "y")
    if [[ "${do_setup,,}" != "n" ]]; then
        echo
        run_init
        echo
        do_daemon=$(prompt_yes_no "Install and start xrat-daemon as a systemd user service? [Y/n]" "y")
        if [[ "${do_daemon,,}" != "n" ]]; then
            run_daemon_install
            echo
            do_linger=$(prompt_yes_no "Start xrat-daemon at boot before login / keep it after logout? [y/N]" "n")
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

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
