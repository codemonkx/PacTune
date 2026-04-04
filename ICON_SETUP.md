# PacTune Icon Setup Guide

## Problem
The original ICON.svg (17MB) is too complex and renders poorly at small sizes in app launchers.

## Solution
Convert the SVG to optimized PNG icons at multiple sizes (48x48, 64x64, 128x128, 256x256, 512x512).

## Steps

### 1. Generate PNG Icons
Run this command from the project root:
```bash
./create_icons.sh
```

This will create PNG icons in:
- `data/hicolor/48x48/apps/`
- `data/hicolor/64x64/apps/`
- `data/hicolor/128x128/apps/`
- `data/hicolor/256x256/apps/`
- `data/hicolor/512x512/apps/`

### 2. Install with New Icons
```bash
./reinstall.sh
```

This will:
- Rebuild the app
- Install all PNG icons at different sizes
- Update desktop files
- Refresh icon cache

### 3. Verify
Launch PacTune from your app launcher - the icon should now look crisp and clear!

## Note
For best quality, install Inkscape:
```bash
sudo pacman -S inkscape  # Arch
sudo apt install inkscape  # Ubuntu/Debian
```

The script will automatically use Inkscape if available, otherwise it falls back to ImageMagick (which you already have).
