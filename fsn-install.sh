#!/usr/bin/env bash
# FreeSynergy.Node – Bootstrap Installer
#
# Downloads / builds the `fsn` binary, then hands off to `fsn init`.
# All setup questions (project, hosts, secrets) are handled by the Rust CLI.
#
# Quick install:
#   bash <(curl -fsSL https://install.freesynergy.net/fsn-install.sh)
#
# Verified install (recommended for production):
#   curl -fsSL https://install.freesynergy.net/fsn-install.sh         -o fsn-install.sh
#   curl -fsSL https://install.freesynergy.net/fsn-install.sh.sha256  -o fsn-install.sh.sha256
#   sha256sum -c fsn-install.sh.sha256 && bash fsn-install.sh
#
# Flags:
#   --repo URL      FSN repository to clone (default: official)
#   --target DIR    Installation directory            (default: ~/FreeSynergy.Node)
#   --skip-build    Use pre-built binary from GitHub releases instead of compiling
#   --skip-init     Clone + build only, do not run `fsn init`

set -euo pipefail

# ── Config defaults ────────────────────────────────────────────────────────────
FSN_REPO="${FSN_REPO:-https://github.com/FreeSynergyNet/FreeSynergy.Node}"
FSN_TARGET="${FSN_TARGET:-$HOME/FreeSynergy.Node}"
FSN_BIN="${FSN_BIN:-$HOME/.local/bin/fsn}"
FSN_SKIP_BUILD="${FSN_SKIP_BUILD:-0}"
FSN_SKIP_INIT="${FSN_SKIP_INIT:-0}"

# ── i18n (minimal – .po lookup for shell messages) ────────────────────────────
_() {
    local key="$1"; shift
    # Future: look up key in locale .po file under locales/
    printf '%s\n' "$key"
}

# ── Helpers ───────────────────────────────────────────────────────────────────
info()  { printf '\033[1;34m==> \033[0m%s\n' "$1"; }
ok()    { printf '\033[1;32m✓   \033[0m%s\n' "$1"; }
warn()  { printf '\033[1;33m!   \033[0m%s\n' "$1" >&2; }
die()   { printf '\033[1;31mERROR: \033[0m%s\n' "$1" >&2; exit 1; }
need()  { command -v "$1" &>/dev/null || die "$(_ "Required command not found: $1")"; }

# ── Argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)        FSN_REPO="$2";   shift 2 ;;
        --target)      FSN_TARGET="$2"; shift 2 ;;
        --skip-build)  FSN_SKIP_BUILD=1; shift ;;
        --skip-init)   FSN_SKIP_INIT=1;  shift ;;
        *) die "Unknown flag: $1" ;;
    esac
done

# ── OS detection ──────────────────────────────────────────────────────────────
detect_os() {
    if [[ -f /etc/os-release ]]; then
        # shellcheck source=/dev/null
        source /etc/os-release
        echo "${ID:-unknown}"
    else
        echo "unknown"
    fi
}

OS="$(detect_os)"
info "$(_ "Detected OS: $OS")"

# ── System dependencies ───────────────────────────────────────────────────────
install_deps() {
    local pkgs=("git" "curl" "podman" "systemd")
    local missing=()

    for pkg in "${pkgs[@]}"; do
        command -v "$pkg" &>/dev/null || missing+=("$pkg")
    done

    # systemd is not a command – check via pid 1
    if ! [[ -d /run/systemd/system ]]; then
        die "$(_ "systemd is required but not running. FSN uses Podman Quadlets (systemd user units).")"
    fi

    if [[ ${#missing[@]} -eq 0 ]]; then
        ok "$(_ "All system dependencies present.")"; return
    fi

    info "$(_ "Installing missing packages: ${missing[*]}")"
    case "$OS" in
        fedora|rhel|centos|rocky|almalinux)
            sudo dnf install -y "${missing[@]}" ;;
        debian|ubuntu|linuxmint|pop)
            sudo apt-get install -y "${missing[@]}" ;;
        arch|manjaro)
            sudo pacman -Sy --noconfirm "${missing[@]}" ;;
        opensuse*|sles)
            sudo zypper install -y "${missing[@]}" ;;
        *)
            warn "$(_ "Unknown OS – please install manually: ${missing[*]}")"
            ;;
    esac
}

install_deps

# ── Enable Podman user socket (needed for Quadlets) ───────────────────────────
if ! systemctl --user is-active podman.socket &>/dev/null; then
    info "$(_ "Enabling Podman user socket…")"
    systemctl --user enable --now podman.socket || true
fi

# Enable lingering so user units survive logout
if command -v loginctl &>/dev/null; then
    loginctl enable-linger "$(id -un)" 2>/dev/null || true
fi

# ── Clone / update repo ───────────────────────────────────────────────────────
if [[ -d "$FSN_TARGET/.git" ]]; then
    info "$(_ "Updating existing repo at $FSN_TARGET")"
    git -C "$FSN_TARGET" pull --ff-only
elif [[ "$(git -C "$(dirname "$0" 2>/dev/null || echo .)" rev-parse --show-toplevel 2>/dev/null)" == "$(pwd)" ]]; then
    info "$(_ "Running from inside repo – skipping clone.")"
    FSN_TARGET="$(pwd)"
else
    info "$(_ "Cloning FreeSynergy.Node to $FSN_TARGET")"
    git clone --depth 1 "$FSN_REPO" "$FSN_TARGET"
fi

cd "$FSN_TARGET"

# ── Build or download `fsn` binary ────────────────────────────────────────────
mkdir -p "$(dirname "$FSN_BIN")"

if [[ "$FSN_SKIP_BUILD" -eq 1 ]]; then
    # Download pre-built release binary
    info "$(_ "Downloading pre-built fsn binary…")"
    ARCH="$(uname -m)"
    RELEASE_URL="$FSN_REPO/releases/latest/download/fsn-$ARCH-unknown-linux-musl"
    curl -fsSL "$RELEASE_URL" -o "$FSN_BIN"
    chmod +x "$FSN_BIN"
    ok "$(_ "Downloaded fsn to $FSN_BIN")"
else
    # Build from source
    if ! command -v cargo &>/dev/null; then
        info "$(_ "Installing Rust toolchain via rustup…")"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi

    info "$(_ "Building fsn binary (this takes a few minutes on first run)…")"
    cargo build --release -p fsn-cli

    install -m 755 cli/target/release/fsn "$FSN_BIN"
    ok "$(_ "Installed fsn to $FSN_BIN")"
fi

# Ensure ~/.local/bin is on PATH for this session
export PATH="$HOME/.local/bin:$PATH"

# ── Verify binary ─────────────────────────────────────────────────────────────
need fsn
ok "$(_ "fsn $(fsn --version 2>/dev/null || echo '(version unknown)') ready.")"

# ── Hand off to fsn init ──────────────────────────────────────────────────────
if [[ "$FSN_SKIP_INIT" -eq 0 ]]; then
    info "$(_ "Starting setup wizard…")"
    exec fsn init --root "$FSN_TARGET"
fi
