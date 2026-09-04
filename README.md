# Steam Throttle

A native Steam download throttler for League of Legends players.

Steam Throttle watches League locally and changes Steam's own runtime download controls. In Arena, downloads are limited while you are alive in combat and restored during preparation, shops, votes, intermissions, and after death. Other League modes use a conservative match-wide policy.

No Overwolf, packet filtering, firewall rules, process injection, memory writing, administrator service, or cloud account is involved.

> [!IMPORTANT]
> Steam Throttle is an independent, community-made project. It is not endorsed by or affiliated with Valve Corporation or Riot Games. Steam and League of Legends are trademarks of their respective owners.

## Default policy

| League state | Steam behavior |
| --- | --- |
| Lobby, queue, champion select, or no match | Unlimited |
| Game process loading; telemetry unavailable | Safe near-pause at 0.128 MB/s |
| Non-Arena match | Safe near-pause at 0.128 MB/s |
| Arena preparation, shop, vote, or resolution | Unlimited |
| Arena combat while alive or uncertain | 10.0 MB/s |
| Arena combat while dead | Unlimited |
| Revived during combat | 10.0 MB/s immediately |

The limit is decimal megabytes per second: **10.0 MB/s = 80,000 Kbps** in Steam's console command.

## How detection works

Steam Throttle checks for `League of Legends.exe` and reads Riot's local Live Client Data API at `https://127.0.0.1:2999`. Arena is identified by `CHERRY`/map 30. Combat is active when living players exist on both `ORDER` and `CHAOS`; preparation/resolution has living players on only one side. The local player's `isDead` state handles death and revival.

This is global Arena combat detection. A team sitting out may still be shown as combat while another pairing fights; that deliberate limitation avoids invasive techniques.

## Steam control and safety

Steam Throttle invokes the installed `steam.exe` directly with Valve's runtime console controls:

- `+set_download_throttle <Kbps> false`
- `+get_download_throttle`
- `+app_download_enable <0|1>`

It never starts Steam. The previous throttle is captured before automation begins and restored when automation stops or you explicitly quit. Because Steam has no reliable getter for its global pause gate, an unknown gate is conservatively limited to 0.128 MB/s rather than risk resuming a download the user paused.

Users can explicitly enable **Steam hard pause** in Settings. In that mode Steam Throttle owns pauses it applies and sets the next throttle before reopening the download gate, preventing an unlimited burst.

## Build from source

Requirements: Windows 10/11 x64, Rust stable, Node.js 22+, npm, and the Microsoft C++ build tools required by Tauri.

```powershell
npm install
npm run check
npm test
Set-Location src-tauri
cargo test
Set-Location ..
npm run tauri build
```

Development: `npm run tauri dev`. Configuration and redacted diagnostics are stored under `%LOCALAPPDATA%\SteamThrottle`.

## Status

Version 0.1.0 is an early Windows-only implementation. Validate behavior in a custom or low-stakes match before relying on it. Packaged builds check signed GitHub Releases after startup and daily; updates are never installed or restarted automatically during a match.

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md). Changes merge through reviewed pull requests; maintainer approval is required.

Licensed under the [MIT License](LICENSE).
