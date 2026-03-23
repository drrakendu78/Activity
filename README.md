<p align="center">
  <img src="src-tauri/icons/activity logo avec texte sous.png" alt="Activity" width="280" />
</p>

<p align="center">
  <strong>Discord Rich Presence, automated.</strong>
</p>

<p align="center">
  Detect your active app, show your music, and control everything from one place.<br/>
  No config files. No scripts. Just launch and go.
</p>

<p align="center">
  <a href="https://github.com/drrakendu78/Activity/releases/latest"><img src="https://img.shields.io/github/v/release/drrakendu78/Activity?style=flat-square&color=blue&sort=semver" alt="Latest Release" /></a>
  <a href="https://github.com/drrakendu78/Activity/blob/master/LICENSE"><img src="https://img.shields.io/github/license/drrakendu78/Activity?style=flat-square" alt="License" /></a>
  <a href="https://github.com/drrakendu78/Activity/releases"><img src="https://img.shields.io/github/downloads/drrakendu78/Activity/total?style=flat-square&color=green" alt="Downloads" /></a>
</p>

---

## Features

### App Detection
- **Auto Detection** — Detects running apps and games in real time and updates your Discord presence automatically.
- **Browser Site Detection** — Shows which website you're on (GitHub, YouTube, Claude...) instead of just "Firefox".
- **Custom App Configs** — Create personalized presence configs for any application with custom details, state, and icons.
- **Auto Icon Fetching** — Fetches app icons from SteamGridDB (free API key required) and Iconify automatically.
- **Icon Picker** — Browse and choose from multiple icon variants for each app.

### Music Player
- **Full Album Page** — Beautiful album layout with cover art, artist, album name, year, and track count.
- **Album Tracklist** — View all tracks from the current album (powered by Deezer API) and click to skip to any track.
- **Per-App Volume** — Control the music app's volume independently from the system volume (WASAPI).
- **Media Controls** — Play/pause, next/prev, seek bar, and volume slider right in the app.
- **Foreground App Toast** — See which app or website you're using while music plays, displayed as a subtle overlay.
- **Discord Integration** — Album art as large image, foreground app in state text, with "On Firefox" or "On VS Code" context.

### General
- **32 Languages** — Full i18n support with 32 languages, from English to Vietnamese.
- **First-Launch Wizard** — Setup wizard on first run to pick your language, theme, and preferences.
- **Light & Dark Theme** — Apple-style UI with acrylic transparency and smooth animations.
- **Auto-Updater** — Checks GitHub for new releases on launch, detects EXE vs MSI install, and updates in one click.
- **System Tray** — Runs quietly in the background with a tray icon and quick controls.
- **Auto Start** — Optionally launch Activity on system startup.
- **Default Presence Presets** — Choose from visual presets for your default status.

## Installation

Download the latest installer from the [Releases](https://github.com/drrakendu78/Activity/releases/latest) page.

| Format | Description |
|--------|-------------|
| `.exe` | NSIS installer (recommended) |
| `.msi` | Windows Installer (for managed deployments) |

## Quick Start

1. **Launch Activity** — The setup wizard guides you through language, theme, and auto-start.
2. **Open any app or game** — Activity detects it automatically and updates your Discord status.
3. **Play some music** — Spotify, Apple Music, or any media player — the album page appears with full controls.
4. **Customize** — Add custom configs for specific apps with your own details and icons.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Framework | [Tauri 2](https://v2.tauri.app/) |
| Frontend | React 18 + TypeScript |
| Styling | Custom CSS + Apple Design |
| Backend | Rust |
| Media | Windows GSMTC + WASAPI |
| Music Data | Deezer API (tracklist) + Spotify API (album art) |
| Icons | SteamGridDB + Iconify |
| i18n | i18next (32 languages) |

## Building from Source

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Steps

```bash
# Clone the repository
git clone https://github.com/drrakendu78/Activity.git
cd Activity

# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

## Contributing

Contributions are welcome. Feel free to open an issue or submit a pull request.

## Privacy

Activity collects **no data**. Everything runs locally. See the full [Privacy Policy](PRIVACY.md).

## License

This project is licensed under the [MIT License](LICENSE).

---

<p align="center">
  Made with Rust and React by <a href="https://github.com/drrakendu78">Drrakendu78</a>
</p>
