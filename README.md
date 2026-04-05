<div align="center">

<img src="./images/pactune-logo.svg" width="128" alt="Vinyl Music Player" />

# Vinyl

**A simple and beautiful music player for Linux**

Built with Rust and GTK4 for a fast and smooth experience.

[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?style=flat-square&logo=rust)](https://rust-lang.org)
[![GTK4](https://img.shields.io/badge/GTK4-Libadwaita-blue?style=flat-square)](https://gtk.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue?style=flat-square)](LICENSE)

</div>

---

## What is Vinyl?

Vinyl is a music player that makes listening to your music collection easy and enjoyable. It has a clean design that fits perfectly with modern Linux desktops, especially GNOME.

**Why Vinyl?**
- Fast and lightweight
- Easy to use
- Beautiful album artwork display
- Shows song lyrics automatically
- Works with your desktop's media controls

---

## Features

### 🎵 Music Library
- **Album View** — See all your albums with cover art in a grid
- **Track List** — Click any album to see all its songs
- **Drag & Drop** — Just drag music files or folders into the window

### 🎼 Playback
- **Play Controls** — Play, pause, skip, and seek through songs
- **Shuffle & Repeat** — Shuffle your music or repeat songs/albums
- **Volume Control** — Easy volume slider
- **Media Keys** — Control playback with your keyboard media keys

### 📝 Lyrics
- **Auto-Sync Lyrics** — Shows lyrics that move with the music
- **Auto-Download** — Fetches lyrics online if you don't have them
- **LRC Support** — Works with `.lrc` lyric files

### 🎨 Interface
- **Dark Theme** — Beautiful dark interface that's easy on the eyes
- **GNOME Integration** — Fits perfectly with GNOME desktop
- **Responsive** — Works great on any screen size
- **Smooth Animations** — Everything moves smoothly

### 🔧 Smart Features
- **Remembers Your Music** — Opens with your last playlist
- **Desktop Integration** — Shows up in your system's media controls
- **Keyboard Shortcuts** — Press spacebar to play/pause
- **Trackpad Gestures** — Swipe to go back

---

## Screenshots

<div align="center">
  <img src="./images/Screenshort-1.png" alt="Vinyl Music Player" width="800" />
  <p><i>Album view with cover art grid</i></p>
</div>

---

## Installation

### Easy Install

1. **Download the code:**
```bash
git clone https://codeberg.org/M23Snezhok/Vinyl.git
cd Vinyl
```

2. **Run the installer:**
```bash
chmod +x install.sh
./install.sh
```

That's it! The installer will:
- Check if you have everything needed
- Build the app
- Install it to your system
- Add it to your application menu

**No root/sudo needed** — Everything installs to your home folder.

### What You Need

The installer checks for these automatically:
- Rust (for building)
- GTK4 (for the interface)
- Libadwaita (for the modern look)
- GStreamer (for playing music)

If something is missing, the installer will tell you what to install.

---

## How to Use

### First Time Setup

1. **Open Vinyl** from your application menu
2. **Add Music** — Click "Open Folder" and choose your music folder
3. **Browse Albums** — Switch to the "Albums" tab to see your collection

### Playing Music

- **Play a Song** — Click any album, then click a song
- **Pause** — Click the pause button or press spacebar
- **Skip Songs** — Use the next/previous buttons
- **Adjust Volume** — Use the volume slider
- **Shuffle** — Click the shuffle button to play in random order
- **Repeat** — Click repeat to loop songs or albums

### Viewing Lyrics

- Click the lyrics button to show/hide lyrics
- Lyrics sync automatically with the music
- If lyrics aren't found, Vinyl tries to download them

### Keyboard Shortcuts

- **Spacebar** — Play/Pause
- **Media Keys** — Next, Previous, Play/Pause (if your keyboard has them)

---

## For Developers

### Building from Source

```bash
# Clone the repository
git clone https://codeberg.org/M23Snezhok/Vinyl.git
cd Vinyl

# Build release version
cargo build --release

# The app will be at: target/release/PacTune
```

### Quick Development

Use the reinstall script for fast testing:

```bash
chmod +x reinstall.sh
./reinstall.sh
```

This rebuilds and reinstalls instantly — perfect for testing changes.

### What's Inside

- **Language:** Rust
- **UI Framework:** GTK4 + Libadwaita
- **Reactive UI:** Relm4
- **Audio:** GStreamer
- **Metadata:** Lofty
- **Lyrics:** lyrx parser + custom fetcher

---

## Supported Audio Formats

Vinyl plays most common audio files:
- MP3
- FLAC
- OGG Vorbis
- Opus
- M4A/AAC
- WAV
- And more!

*(Depends on your GStreamer plugins)*

---

## Future Plans

Things we want to add:
- [ ] Create and save playlists
- [ ] Search for songs and albums
- [ ] Audio equalizer
- [ ] Last.fm scrobbling
- [ ] Custom color themes
- [ ] Podcast support
- [ ] Queue management

Have an idea? Open an issue on Codeberg!

---

## Contributing

We welcome contributions! Here's how you can help:

1. **Report Bugs** — Found something broken? Let us know!
2. **Suggest Features** — Have an idea? Share it!
3. **Submit Code** — Fork, code, and send a pull request
4. **Improve Docs** — Help make this README even better

---

## Credits

**Created by:** Mikhail Kostin ([@M23Snezhok](https://codeberg.org/M23Snezhok))

**Built with love for:**
- Music lovers
- Linux users
- Open source community

---

## License

**GNU General Public License v3.0**

This is free software. You can use it, modify it, and share it freely.

See the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Enjoy your music! 🎵**

If you like Vinyl, give it a star on [Codeberg](https://codeberg.org/M23Snezhok/Vinyl)!

</div>

