#!/usr/bin/env bash
# OmniLauncherMC Universal Installer
# Works on Linux and macOS — builds from source, always latest
# Usage: curl -fsSL https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.sh | bash
# Update: omni-launcher-mc-update

set -euo pipefail

REPO="https://github.com/Gaming-RF/Omni-Launcher-MC.git"
INSTALL_DIR="$HOME/.local/share/OmniLauncherMC"
BIN_LINK="$HOME/.local/bin/omni-launcher-mc"
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
fail()  { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }

OS="$(uname -s)"

# ── Linux deps ──────────────────────────────────────────────────────
install_linux_deps() {
  if command -v apt-get &>/dev/null; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq build-essential curl wget git pkg-config libssl-dev \
      libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
      librsvg2-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev \
      file patchelf libglib2.0-dev libgdk-pixbuf-2.0-dev libpango1.0-dev libcairo2-dev libatk1.0-dev
  elif command -v dnf &>/dev/null; then
    sudo dnf install -y gcc gcc-c++ make curl wget git openssl-devel pkg-config \
      gtk3-devel webkit2gtk4.1-devel libayatana-appindicator-gtk3-devel \
      librsvg2-devel javascriptcoregtk4.1-devel libsoup3-devel \
      patchelf glib2-devel gdk-pixbuf2-devel pango-devel cairo-devel atk-devel
  elif command -v pacman &>/dev/null; then
    sudo pacman -Syu --noconfirm --needed base-devel curl wget git openssl pkg-config \
      gtk3 webkit2gtk-4.1 libayatana-appindicator librsvg patchelf
  elif command -v zypper &>/dev/null; then
    sudo zypper install -y gcc gcc-c++ make curl wget git openssl-devel pkg-config \
      gtk3-devel webkit2gtk4.1-devel libayatana-appindicator3-devel librsvg-devel patchelf
  elif command -v apk &>/dev/null; then
    sudo apk add build-base curl wget git openssl-dev pkgconf gtk+3.0-dev webkit2gtk-dev librsvg-dev patchelf
  else
    warn "Unknown Linux distro. You need: build-essential, curl, git, libgtk-3-dev, libwebkit2gtk-4.1-dev, libssl-dev, patchelf"
  fi
}

# ── macOS deps ──────────────────────────────────────────────────────
install_macos_deps() {
  # Xcode CLI tools
  if ! xcode-select -p &>/dev/null; then
    info "Installing Xcode Command Line Tools..."
    xcode-select --install 2>/dev/null || true
    echo "A dialog will appear — click Install. Re-run this script after it finishes."
    fail "Xcode CLI tools required. Install them and re-run."
  fi
  ok "Xcode CLI tools present"

  # Homebrew
  if ! command -v brew &>/dev/null; then
    info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    eval "$(/opt/homebrew/bin/brew shellenv 2>/dev/null || /usr/local/bin/brew shellenv 2>/dev/null)"
  fi
  ok "Homebrew ready"

  brew install curl wget
}

# ── Install Rust ────────────────────────────────────────────────────
install_rust() {
  if command -v cargo &>/dev/null; then
    ok "Rust installed ($(rustc --version))"
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
    ok "Node.js installed ($(node --version))"
  else
    info "Installing Node.js 22..."
    if [ "$OS" = "Darwin" ]; then
      brew install node@22
      brew link --overwrite node@22
    elif command -v apt-get &>/dev/null; then
      curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
      sudo apt-get install -y -qq nodejs
    elif command -v dnf &>/dev/null; then
      curl -fsSL https://rpm.nodesource.com/setup_22.x | sudo bash -
      sudo dnf install -y nodejs
    elif command -v pacman &>/dev/null; then
      sudo pacman -S --noconfirm nodejs npm
    else
      fail "Install Node.js 22+ manually: https://nodejs.org"
    fi
    ok "Node.js installed ($(node --version))"
  fi

  if command -v pnpm &>/dev/null; then
    ok "pnpm installed ($(pnpm --version))"
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

  info "Building Tauri backend (release)... this may take a few minutes"
  cd src-tauri
  cargo build --release
  ok "Backend built"

  local binary="$INSTALL_DIR/src-tauri/target/release/omni-launcher-mc"
  if [ ! -f "$binary" ]; then
    fail "Binary not found at $binary"
  fi
  ok "Binary: $binary ($(du -h "$binary" | cut -f1))"
}

# ── Install ─────────────────────────────────────────────────────────
install_app() {
  mkdir -p "$HOME/.local/bin"

  local binary="$INSTALL_DIR/src-tauri/target/release/omni-launcher-mc"
  ln -sf "$binary" "$BIN_LINK"
  chmod +x "$BIN_LINK"
  ok "Linked to $BIN_LINK"

  # PATH check
  if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    warn "~/.local/bin not in PATH. Add to your shell rc:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    # Auto-detect shell rc
    local rc=""
    case "$SHELL" in
      */zsh)  rc="$HOME/.zshrc" ;;
      */bash) rc="$HOME/.bashrc" ;;
      */fish) rc="$HOME/.config/fish/config.fish" ;;
    esac
    if [ -n "$rc" ] && [ -f "$rc" ]; then
      if ! grep -q '.local/bin' "$rc" 2>/dev/null; then
        echo '' >> "$rc"
        echo '# OmniLauncherMC' >> "$rc"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$rc"
        info "Added PATH to $rc — restart your shell after install"
      fi
    fi
  fi

  # Desktop entry (Linux only)
  if [ "$OS" = "Linux" ]; then
    mkdir -p "$HOME/.local/share/applications"
    local icon_path="$INSTALL_DIR/src-tauri/icons/128x128.png"
    [ ! -f "$icon_path" ] && icon_path="$INSTALL_DIR/src-tauri/icons/icon.png"
    cat > "$HOME/.local/share/applications/omni-launcher-mc.desktop" <<EOF
[Desktop Entry]
Name=OmniLauncherMC
Comment=Cross-platform Minecraft launcher
Exec=$BIN_LINK %u
Icon=$icon_path
Type=Application
Categories=Game;
Terminal=false
MimeType=x-scheme-handler/omnilaunchermc;
StartupWMClass=omni-launcher-mc
EOF
    update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
    ok "Desktop entry created"
  fi

  # macOS .app symlink
  if [ "$OS" = "Darwin" ]; then
    local app_bundle="$INSTALL_DIR/src-tauri/target/release/bundle/macos/OmniLauncherMC.app"
    if [ -d "$app_bundle" ]; then
      ln -sf "$app_bundle" "$HOME/Applications/OmniLauncherMC.app" 2>/dev/null || true
      ok "Linked .app to ~/Applications/"
    fi
  fi
}

# ── Update script ───────────────────────────────────────────────────
create_update_script() {
  local update_bin="$HOME/.local/bin/omni-launcher-mc-update"
  cat > "$update_bin" <<'UPDEOF'
#!/usr/bin/env bash
set -euo pipefail
INSTALL_DIR="$HOME/.local/share/OmniLauncherMC"
GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
[ ! -d "$INSTALL_DIR/.git" ] && echo "Not installed. Run install.sh first." && exit 1
cd "$INSTALL_DIR"
git fetch --all --tags
LOCAL=$(git rev-parse HEAD); REMOTE=$(git rev-parse origin/main)
if [ "$LOCAL" = "$REMOTE" ]; then
  echo -e "${GREEN}[OK]${NC} Already latest ($(git rev-parse --short HEAD))"
  exit 0
fi
git reset --hard origin/main
echo -e "${CYAN}[INFO]${NC} Rebuilding from $(git rev-parse --short HEAD)..."
pnpm install --frozen-lockfile 2>/dev/null || pnpm install
pnpm build
cd src-tauri && cargo build --release
echo -e "${GREEN}[OK]${NC} Updated! Run: omni-launcher-mc"
UPDEOF
  chmod +x "$update_bin"
  ok "Update command: omni-launcher-mc-update"
}

# ── Main ────────────────────────────────────────────────────────────
main() {
  echo ""
  echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
  echo -e "${CYAN}║   OmniLauncherMC — Universal Installer   ║${NC}"
  echo -e "${CYAN}║   Linux + macOS | builds from source     ║${NC}"
  echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
  echo ""
  info "Detected OS: $OS"

  case "$OS" in
    Linux)  install_linux_deps ;;
    Darwin) install_macos_deps ;;
    *)      fail "Unsupported OS: $OS. Use install.ps1 on Windows." ;;
  esac

  install_rust
  install_node
  clone_or_update
  build_app
  install_app
  create_update_script

  echo ""
  echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
  echo -e "${GREEN}║         Installation Complete!            ║${NC}"
  echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
  echo ""
  echo "  Run:           omni-launcher-mc"
  echo "  Update later:  omni-launcher-mc-update"
  echo "  Source:        $INSTALL_DIR"
  echo ""
}

main "$@"
