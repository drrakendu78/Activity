# Privacy Policy

**Effective date:** March 24, 2026
**Application:** Activity — Discord Rich Presence Manager
**Developer:** Drrakendu78

---

## 1. Overview

Activity is a local-only desktop application. It does not collect, store, process, or transmit any personally identifiable information (PII). No user data leaves your device except through explicit, user-initiated interactions with third-party public APIs as described in Section 3.

## 2. Data Collection

**Activity collects no data whatsoever.**

- No telemetry, analytics, or crash reporting
- No user accounts, registration, or authentication
- No cookies, tracking pixels, or fingerprinting
- No usage statistics or behavioral profiling
- No IP address logging or geolocation

## 3. Third-Party API Communications

Activity communicates with the following external services strictly to provide core functionality. All requests are initiated locally and contain no personally identifiable information.

| Service | Purpose | Data Sent | Data Received |
|---------|---------|-----------|---------------|
| **Discord RPC** | Display rich presence status | Application name, state, timestamps, icon keys | Connection acknowledgment |
| **SteamGridDB API** | Fetch application/game icons | Application name, API key (user-provided) | Icon image URLs |
| **Iconify API** | Fetch fallback application icons | Icon identifier string | SVG icon data |
| **Deezer Public API** | Retrieve album tracklists and metadata | Album/artist search query | Album metadata, track listings |
| **Spotify Web API** | Retrieve album artwork | Album/artist search query, client credentials (hardcoded, non-user) | Album art URLs |

All API calls are made over HTTPS (TLS 1.2+). No authentication tokens, session identifiers, or user credentials are transmitted except the optional SteamGridDB API key, which is stored locally and never shared with any party other than SteamGridDB.

## 4. Local Data Storage

All application data is stored exclusively on the user's device in the operating system's standard application data directory (`%APPDATA%` on Windows).

Stored data includes:
- **Configuration file** (`config.json`): user preferences, theme, language, toggle states
- **SteamGridDB API key** (if provided): stored in plaintext in the local config file
- **Cached icon mappings**: application-to-icon associations for offline use

No data is encrypted at rest as it contains no sensitive information. No data is synchronized, uploaded, or backed up to any remote server.

## 5. Inter-Process Communication

Activity uses the following local IPC mechanisms:

- **Discord RPC**: Local socket connection (`\\?\pipe\discord-ipc-0`) to the Discord desktop client
- **Windows GSMTC**: Read-only access to the Global System Media Transport Controls for detecting currently playing media
- **WASAPI**: Local audio session control for per-application volume management

All IPC is strictly local and does not traverse any network boundary.

## 6. Auto-Updater

Activity includes a silent auto-updater that checks for new releases via the GitHub Releases API (`api.github.com`). This request contains:

- The current application version
- Standard HTTPS headers (User-Agent)

Downloaded update binaries are verified using SHA-256 hash comparison before execution. No user data is included in update requests.

## 7. Permissions

Activity requires no elevated privileges. It runs in user-space and accesses only:

- Local filesystem (AppData directory, read/write)
- Local named pipes (Discord IPC, read/write)
- Network (HTTPS, outbound only, to the APIs listed in Section 3)
- Windows media session APIs (read-only)
- Windows audio session APIs (read/write for volume control)

## 8. Children's Privacy

Activity does not knowingly collect any information from any user, regardless of age, as it collects no data at all.

## 9. Changes to This Policy

Any changes to this privacy policy will be reflected in this document and included in the corresponding release notes.

## 10. Contact

For privacy-related inquiries, open an issue on the [GitHub repository](https://github.com/drrakendu78/Activity/issues).

---

**TL;DR — Activity collects nothing. Everything stays on your machine.**
