<p align="center">
  <img src="src-tauri/icons/activity logo avec texte sous.png" alt="Activity" width="280" />
</p>

<p align="center">
  <strong>Discord Rich Presence Manager</strong>
</p>

<p align="center">
  Automatically display what you're doing on Discord.<br/>
  No config files. No scripts. Just launch and go.
</p>

<p align="center">
  <a href="https://github.com/drrakendu78/Activity/releases/latest"><img src="https://img.shields.io/github/v/release/drrakendu78/Activity?style=flat-square&color=blue&sort=semver" alt="Latest Release" /></a>
  <a href="https://github.com/drrakendu78/Activity/blob/master/LICENSE"><img src="https://img.shields.io/github/license/drrakendu78/Activity?style=flat-square" alt="License" /></a>
  <a href="https://github.com/drrakendu78/Activity/releases"><img src="https://img.shields.io/github/downloads/drrakendu78/Activity/total?style=flat-square&color=green" alt="Downloads" /></a>
</p>

---

## Features

- **Auto Detection** — Detects running apps and games in real time and updates your Discord presence automatically.
- **Custom App Configs** — Create personalized presence configs for any application with custom details, state, and icons.
- **Auto Icon Fetching** — Automatically fetches app icons from SteamGridDB. No manual setup needed.
- **Icon Picker** — Browse and choose from multiple icon variants for each app.
- **30 Languages** — Full i18n support with 30 languages, from English to Vietnamese.
- **First-Launch Wizard** — Setup wizard on first run to pick your language, theme, and preferences.
- **Light & Dark Theme** — Apple-style UI with smooth animations and spring physics.
- **System Tray** — Runs quietly in the background with a tray icon and quick controls.
- **Auto Start** — Optionally launch Activity on system startup.
- **Default Presence Presets** — Choose from visual presets for your default status — no code, just click.

## Installation

### Manual Download

Download the latest installer from the [Releases](https://github.com/drrakendu78/Activity/releases/latest) page.

## Quick Start

1. **Launch Activity** — The setup wizard guides you through language, theme, and auto-start.
2. **Open any app or game** — Activity detects it automatically.
3. **Your Discord status updates** — Your friends see what you're using in real time.
4. **Customize** — Add custom configs for specific apps with your own details and icons.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Framework | [Tauri 2](https://v2.tauri.app/) |
| Frontend | React 18 + TypeScript |
| Styling | Custom CSS + Apple Design |
| Backend | Rust |
| i18n | i18next (30 languages) |
| Icons | SteamGridDB API |

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

## License

This project is licensed under the [MIT License](LICENSE).

---

<p align="center">
  Made with Rust and React by <a href="https://github.com/drrakendu78">Drrakendu78</a>
</p>
