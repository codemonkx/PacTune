#!/usr/bin/env bash
set -e

# ─────────────────────────────────────────────
#  PacTune Music Player — Quick Reinstall Script
#  Use this after making code changes
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
echo "  ║  PacTune Quick Reinstall (Local Dev) ║"
echo "  ╚══════════════════════════════════════╝"
echo -e "${RESET}"

# ── 1. Check we're in the repo root ──────────────────────────────────────────
if [[ ! -f "Cargo.toml" ]]; then
    error "Run this script from the root of the PacTune repository."
fi

# ── 2. Stop running instances ─────────────────────────────────────────────────
section "Stopping any running PacTune instances..."
pkill -f "pactune" 2>/dev/null || true
info "Stopped running instances"

# ── 3. Clean previous build (optional, faster without) ────────────────────────
# Uncomment the next 2 lines if you want a clean build every time
# section "Cleaning previous build..."
# cargo clean

# ── 4. Rebuild ────────────────────────────────────────────────────────────────
section "Rebuilding PacTune with your changes..."
cargo build --release
info "Build complete"

# ── 5. Reinstall binary ───────────────────────────────────────────────────────
section "Reinstalling binary..."

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"
install -Dm755 target/release/PacTune "$BIN_DIR/pactune"
info "Binary updated at $BIN_DIR/pactune"

# ── 6. Update icons and desktop files ─────────────────────────────────────────
section "Updating icons and desktop files..."

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

APP_DIR="$HOME/.local/share/applications"
mkdir -p "$APP_DIR"

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

DBUS_DIR="$HOME/.local/share/dbus-1/services"
mkdir -p "$DBUS_DIR"

cat > "$DBUS_DIR/page.codeberg.M23Snezhok.PacTune.service" << DBUS
[D-BUS Service]
Name=page.codeberg.M23Snezhok.PacTune
Exec=$BIN_DIR/pactune --gapplication-service
DBUS

info "Desktop files updated"

# ── 7. Refresh caches ─────────────────────────────────────────────────────────
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}Reinstall complete!${RESET}"
echo ""
echo "  Your changes are now installed. You can:"
echo "  • Launch the updated app: pactune"
echo "  • Or run from terminal to see debug output: pactune"
echo ""
