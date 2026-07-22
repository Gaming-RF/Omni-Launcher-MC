#!/usr/bin/env bash
# OmniLauncherMC Universal Installer
# Works on any Linux distro - builds from source, always latest
# Usage: curl -fsSL https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.sh | bash

set -euo pipefail

REPO="https://github.com/Gaming-RF/Omni-Launcher-MC.git"
INSTALL_DIR="$HOME/.local/share/OmniLauncherMC"
BIN_LINK="$HOME/.local/bin/omni-launcher-mc"
DESKTOP_FILE="$HOME/.local/share/applications/omni-launcher-mc.desktop"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
fail()  { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }

# ── Detect package manager ──────────────────────────────────────────
detect_pkg_mgr() {
  if command -v apt-get &>/dev/null; then echo "apt"
  elif command -v dnf &>/dev/null; then echo "dnf"
  elif command -v pacman &>/dev/null; then echo "pacman"
  elif command -v zypper &>/dev/null; then echo "zypper"
  elif command -v apk &>/dev/null; then echo "apk"
  else echo "unknown"
  fi
}

install_pkgs() {
  local mgr=$(detect_pkg_mgr)
  info "Installing dependencies via $mgr..."
  case "$mgr" in
    apt)
      sudo apt-get update -qq
      sudo apt-get install -y -qq \
        build-essential curl wget git pkg-config libssl-dev \
        libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
        librsvg2-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev \
        file patchelf libglib2.0-dev libgdk-pixbuf-2.0-dev \
        libpango1.0-dev libcairo2-dev libatk1.0-dev
      ;;
    dnf)
      sudo dnf install -y \
        gcc gcc-c++ make curl wget git openssl-devel pkg-config \
        gtk3-devel webkit2gtk4.1-devel libayatana-appindicator-gtk3-devel \
        librsvg2-devel javascriptcoregtk4.1-devel libsoup3-devel \
        patchelf glib2-devel gdk-pixbuf2-devel pango-devel cairo-devel atk-devel
      ;;
    pacman)
      sudo pacman -Syu --noconfirm --needed \
        base-devel curl wget git openssl pkg-config \
        gtk3 webkit2gtk-4.1 libayatana-appindicator librsvg \
        patchelf
      ;;
    zypper)
      sudo zypper install -y \
        gcc gcc-c++ make curl wget git openssl-devel pkg-config \
        gtk3-devel webkit2gtk4.1-devel libayatana-appindicator3-devel \
        librsvg-devel patchelf
      ;;
    apk)
      sudo apk add \
        build-base curl wget git openssl-dev pkgconf \
        gtk+3.0-dev webkit2gtk-dev librsvg-dev patchelf
      ;;
    *)
      warn "Unknown package manager. You need: build-essential, curl, git, libgtk-3-dev, libwebkit2gtk-4.1-dev, libssl-dev, patchelf"
      ;;
  esac
  ok "System dependencies installed"
}

# ── Install Rust ────────────────────────────────────────────────────
install_rust() {
  if command -v cargo &>/dev/null; then
    ok "Rust already installed ($(rustc --version))"
  else
    info "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    ok "Rust installed ($(rustc --version))"
  fi
}

# ── Install Node.js + pnpm ──────────────────────────────────────────
install_node() {
  if command -v node &>/dev/null; then
    ok "Node.js already installed ($(node --version))"
  else
    info "Installing Node.js 22 LTS..."
    if command -v curl &>/dev/null; then
      curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
      if command -v apt-get &>/dev/null; then
        sudo apt-get install -y -qq nodejs
      elif command -v dnf &>/dev/null; then
        sudo dnf install -y nodejs
      elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm nodejs npm
      else
        fail "Install Node.js 22+ manually: https://nodejs.org"
      fi
    fi
    ok "Node.js installed ($(node --version))"
  fi

  if command -v pnpm &>/dev/null; then
    ok "pnpm already installed ($(pnpm --version))"
  else
    info "Installing pnpm..."
    npm install -g pnpm
    ok "pnpm installed ($(pnpm --version))"
  fi
}

# ── Clone or update repo ────────────────────────────────────────────
clone_or_update() {
  mkdir -p "$(dirname "$INSTALL_DIR")"
  if [ -d "$INSTALL_DIR/.git" ]; then
    info "Updating existing clone..."
    cd "$INSTALL_DIR"
    git fetch --all --tags
    git reset --hard origin/main
  else
    info "Cloning repository..."
    git clone --depth 1 "$REPO" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
  fi
  ok "Source ready ($(git rev-parse --short HEAD))"
}

# ── Build ───────────────────────────────────────────────────────────
build_app() {
  cd "$INSTALL_DIR"
  info "Installing frontend dependencies..."
  pnpm install --frozen-lockfile 2>/dev/null || pnpm install
  ok "Frontend deps installed"

  info "Building frontend..."
  pnpm build
  ok "Frontend built"

  info "Building Tauri app (release)... this takes a few minutes"
  cd src-tauri
  cargo build --release
  ok "Rust backend built"

  local binary="$INSTALL_DIR/src-tauri/target/release/omni-launcher-mc"
  if [ ! -f "$binary" ]; then
    fail "Binary not found at $binary"
  fi
  ok "Binary ready: $binary ($(du -h "$binary" | cut -f1))"
}

# ── Install ─────────────────────────────────────────────────────────
install_app() {
  mkdir -p "$HOME/.local/bin"
  mkdir -p "$HOME/.local/share/applications"

  local binary="$INSTALL_DIR/src-tauri/target/release/omni-launcher-mc"

  # Symlink binary
  ln -sf "$binary" "$BIN_LINK"
  chmod +x "$BIN_LINK"
  ok "Linked binary to $BIN_LINK"

  # Add to PATH if needed
  if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    warn "~/.local/bin not in PATH. Add to your shell rc:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
  fi

  # Desktop entry
  cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Name=OmniLauncherMC
Comment=Cross-platform Minecraft launcher
Exec=$BIN_LINK %u
Icon=$INSTALL_DIR/src-tauri/icons/128x128.png
Type=Application
Categories=Game;
Terminal=false
MimeType=x-scheme-handler/omnilaunchermc;
StartupWMClass=omni-launcher-mc
EOF
  ok "Desktop entry created"

  # Update desktop database
  if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
  fi
}

# ── Update shortcut ─────────────────────────────────────────────────
create_update_script() {
  local update_bin="$HOME/.local/bin/omni-launcher-mc-update"
  cat > "$update_bin" <<'UPDEOF'
#!/usr/bin/env bash
set -euo pipefail
INSTALL_DIR="$HOME/.local/share/OmniLauncherMC"
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}[INFO]${NC} Updating OmniLauncherMC..."
cd "$INSTALL_DIR"
git fetch --all --tags
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main)

if [ "$LOCAL" = "$REMOTE" ]; then
  echo -e "${GREEN}[OK]${NC} Already up to date ($(git rev-parse --short HEAD))"
  exit 0
fi

git reset --hard origin/main
echo -e "${CYAN}[INFO]${NC} Rebuilding..."
pnpm install --frozen-lockfile 2>/dev/null || pnpm install
pnpm build
cd src-tauri
cargo build --release
echo -e "${GREEN}[OK]${NC} Updated to $(git rev-parse --short HEAD)"
echo "Run: omni-launcher-mc"
UPDEOF
  chmod +x "$update_bin"
  ok "Update script: omni-launcher-mc-update"
}

# ── Main ────────────────────────────────────────────────────────────
main() {
  echo ""
  echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
  echo -e "${CYAN}║     OmniLauncherMC Universal Installer   ║${NC}"
  echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
  echo ""

  install_pkgs
  install_rust
  install_node
  clone_or_update
  build_app
  install_app
  create_update_script

  echo ""
  echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
  echo -e "${GREEN}║          Installation Complete!           ║${NC}"
  echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
  echo ""
  echo "  Run:           omni-launcher-mc"
  echo "  Update later:  omni-launcher-mc-update"
  echo "  Source:        $INSTALL_DIR"
  echo ""
}

main "$@"
