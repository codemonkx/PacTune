#!/usr/bin/env bash
set -e

echo "Creating optimized PNG icons from ICON.svg..."

# Check if ImageMagick or Inkscape is available
if command -v inkscape &>/dev/null; then
    TOOL="inkscape"
    echo "Using Inkscape (best quality)..."
elif command -v convert &>/dev/null; then
    TOOL="imagemagick"
    echo "Using ImageMagick..."
else
    echo "Error: Neither Inkscape nor ImageMagick found."
    echo "Please install one of them:"
    echo "  Ubuntu/Debian: sudo apt install inkscape"
    echo "  Fedora: sudo dnf install inkscape"
    echo "  Arch: sudo pacman -S inkscape"
    exit 1
fi

# Create icon directory structure
mkdir -p data/hicolor/{48x48,64x64,96x96,128x128,256x256,512x512}/apps

# Generate icons at different sizes with high quality
SIZES=(48 64 96 128 256 512)

for size in "${SIZES[@]}"; do
    output="data/hicolor/${size}x${size}/apps/page.codeberg.M23Snezhok.PacTune.png"
    
    if [ "$TOOL" = "inkscape" ]; then
        # Inkscape gives better quality for complex SVGs
        inkscape data/hicolor/scalable/apps/ICON.svg \
                 --export-type=png \
                 --export-filename="$output" \
                 --export-width=$size \
                 --export-height=$size \
                 2>/dev/null
    else
        # ImageMagick with high density for better quality
        convert -background none -density 600 data/hicolor/scalable/apps/ICON.svg \
                -resize ${size}x${size} \
                -quality 100 \
                "$output"
    fi
    
    echo "✓ Created ${size}x${size} icon"
done

echo ""
echo "✓ Done! High-quality PNG icons created in data/hicolor/"
echo ""
echo "Next step: Run ./reinstall.sh to install with the new icons"
