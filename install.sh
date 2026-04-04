#!/usr/bin/env bash
set -e

# ─────────────────────────────────────────────
#  PacTune Music Player — Universal Installer
# ─────────────────────────────────────────────

BOLD="\033[1m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

info()    { echo -e "${GREEN}[✓]${RESET} $1"; }
warn()    { echo -e "${YELLOW}[!]${RESET} $1"; }
error()   { echo -e "${RED}[✗]${RESET} $1"; exit 1; }
section() { echo -e "\n${BOLD}$1${RESET}"; }

echo -e "${BOLD}"
echo "  ╔══════════════════════════════════════╗"
echo "  ║    PacTune Music Player Installer    ║"
echo "  ╚══════════════════════════════════════╝"
echo -e "${RESET}"

# ── 1. Check we're in the repo root ──────────────────────────────────────────
if [[ ! -f "Cargo.toml" ]]; then
    error "Run this script from the root of the PacTune repository."
fi

# ── 2. Check required tools ───────────────────────────────────────────────────
section "Checking dependencies..."

check_cmd() {
    if ! command -v "$1" &>/dev/null; then
        error "Required tool '$1' not found. Please install it and try again."
    fi
    info "$1 found"
}

check_cmd cargo
check_cmd pkg-config

# Check GTK4 / Libadwaita / GStreamer via pkg-config
check_pkg() {
    if ! pkg-config --exists "$1" 2>/dev/null; then
        warn "pkg-config could not find '$1'. The build may fail."
        warn "Install the development package for '$1' (e.g. lib${1}-dev or ${1}-devel)."
    else
        info "$1 found"
    fi
}

check_pkg gtk4
check_pkg libadwaita-1
check_pkg gstreamer-1.0
check_pkg gstreamer-play-1.0

# ── 3. Build ──────────────────────────────────────────────────────────────────
section "Building PacTune (release)..."
cargo build --release
info "Build complete"

# ── 4. Install binary ─────────────────────────────────────────────────────────
section "Installing..."

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"
install -Dm755 target/release/PacTune "$BIN_DIR/pactune"
info "Binary installed to $BIN_DIR/pactune"

# Warn if ~/.local/bin is not in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    warn "$BIN_DIR is not in your PATH."
    warn "Add this to your shell config (~/.bashrc, ~/.zshrc, ~/.config/fish/config.fish):"
    warn "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

# ── 5. Install icons ──────────────────────────────────────────────────────────
section "Installing icons..."

# Check if PNG icons exist, if not create them
if [[ ! -f "data/hicolor/256x256/apps/page.codeberg.M23Snezhok.PacTune.png" ]]; then
    warn "PNG icons not found. Generating them now..."
    if [[ -x "./create_icons.sh" ]]; then
        ./create_icons.sh
    else
        error "create_icons.sh not found or not executable. Run: chmod +x create_icons.sh && ./create_icons.sh"
    fi
fi

# Install PNG icons at multiple sizes for best quality
for size in 48 64 128 256 512; do
    ICON_DIR="$HOME/.local/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$ICON_DIR"
    if [[ -f "data/hicolor/${size}x${size}/apps/page.codeberg.M23Snezhok.PacTune.png" ]]; then
        cp "data/hicolor/${size}x${size}/apps/page.codeberg.M23Snezhok.PacTune.png" "$ICON_DIR/"
    fi
done

# Also install the SVG as fallback
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
mkdir -p "$ICON_DIR"
cp data/hicolor/scalable/apps/ICON.svg "$ICON_DIR/page.codeberg.M23Snezhok.PacTune.svg"

info "Icons installed (PNG + SVG)"

# ── 6. Install desktop entry ──────────────────────────────────────────────────
APP_DIR="$HOME/.local/share/applications"
mkdir -p "$APP_DIR"

# Write a clean desktop file with the correct absolute path and no DBusActivatable
cat > "$APP_DIR/pactune.desktop" << DESKTOP
[Desktop Entry]
Name=PacTune
GenericName=Music Player
TryExec=$BIN_DIR/pactune
Exec=$BIN_DIR/pactune %U
Icon=page.codeberg.M23Snezhok.PacTune
Terminal=false
Type=Application
Categories=GNOME;GTK;Music;Audio;
Keywords=music;player;media;audio;playlist;
StartupNotify=true
X-SingleMainWindow=true
MimeType=audio/mpeg;audio/wav;audio/x-aac;audio/x-aiff;audio/x-ape;audio/x-flac;audio/x-m4a;audio/x-m4b;audio/x-mp1;audio/x-mp2;audio/x-mp3;audio/x-mpg;audio/x-mpeg;audio/x-mpegurl;audio/x-opus+ogg;audio/x-pn-aiff;audio/x-pn-au;audio/x-pn-wav;audio/x-speex;audio/x-vorbis;audio/x-vorbis+ogg;audio/x-wavpack;inode/directory;
DESKTOP

info "Desktop entry installed to $APP_DIR/pactune.desktop"

# ── 7. Install D-Bus service file (for GNOME launcher activation) ─────────────
DBUS_DIR="$HOME/.local/share/dbus-1/services"
mkdir -p "$DBUS_DIR"

cat > "$DBUS_DIR/page.codeberg.M23Snezhok.PacTune.service" << DBUS
[D-BUS Service]
Name=page.codeberg.M23Snezhok.PacTune
Exec=$BIN_DIR/pactune --gapplication-service
DBUS

info "D-Bus service file installed"

# ── 8. Refresh caches ─────────────────────────────────────────────────────────
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}Installation complete!${RESET}"
echo ""
echo "  You can now:"
echo "  • Launch PacTune from your app launcher (search for 'PacTune')"
echo "  • Or run it from the terminal: pactune"
echo ""
