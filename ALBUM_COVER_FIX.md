# Album Cover Size Fix

## Problem
The album cover on the left side (player panel) was changing size when switching songs. Some covers would appear smaller than others.

## Root Cause
When `gtk::Image` uses `set_paintable()` to display textures (album covers), it doesn't automatically constrain the image to the `pixel_size` property like it does for icon names. Different album covers have different dimensions, causing the widget to resize based on the texture's natural size.

## Solution Applied

### 1. Changed Widget Type (src/app.rs)
Replaced `gtk::Image` with `gtk::Picture` which has proper content fitting:
```rust
gtk::Picture {
    add_css_class: "player-cover",
    set_content_fit: gtk::ContentFit::Cover,
    set_can_shrink: false,
}
```

### 2. Added Overlay with Fallback Icon
Wrapped the Picture in an Overlay with a fallback icon that shows when no cover is available:
```rust
gtk::Overlay {
    set_width_request: 260,
    set_height_request: 260,
    // Picture for album art
    // Image overlay for fallback icon
}
```

### 3. CSS Constraints (src/main.rs)
Added min/max width and height constraints to ensure consistent sizing:

```css
.player-cover {
    min-width: 260px;
    min-height: 260px;
    max-width: 260px;
    max-height: 260px;
    background: #282828;
}

.album-cover-container {
    min-width: 160px;
    min-height: 160px;
    max-width: 160px;
    max-height: 160px;
}

.song-cover {
    min-width: 120px;
    min-height: 120px;
    max-width: 120px;
    max-height: 120px;
}
```

## Result
- Player cover (right panel): Always 260x260px with proper scaling
- Album grid covers: Always 160x160px
- Track detail covers: Always 120x120px
- `gtk::Picture` with `ContentFit::Cover` ensures images scale properly while maintaining aspect ratio

All album covers now maintain consistent size regardless of the source image dimensions.

## Testing
Run `./reinstall.sh` and switch between different songs (e.g., "Evan DI unna pethan" and "Vaanam"). The album cover should now stay the same size with proper scaling.
