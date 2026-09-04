# Steam Throttle

Steam Throttle adjusts Steam download bandwidth around League of Legends matches. It is a small, Windows-only desktop app built for players who want downloads running between Arena fights without competing with the game while they are alive.

It does not use Overwolf, packet filtering, firewall rules, process injection, memory access, an administrator service, or a cloud account.

## Download

Download the installer or portable build from [GitHub Releases](https://github.com/csmit195/SteamThrottle/releases/latest).

- The installer is per-user and does not require administrator access.
- The portable ZIP contains a standalone executable.
- Windows may show a SmartScreen warning because the builds are not Authenticode-signed.

## What it does

The default Arena behavior is:

| League state                                   | Steam download state |
| ---------------------------------------------- | -------------------- |
| Outside a match                                | Unlimited            |
| Arena preparation, shop, vote, or intermission | Unlimited            |
| Arena combat while alive                       | Limited to 10 MB/s   |
| Arena combat while dead                        | Unlimited            |
| Revived during combat                          | Limited immediately  |
| Other League modes                             | Paused for the match |
| Loading or uncertain telemetry                 | Paused               |

The limit and each major behavior can be changed in the app. Steam Throttle captures the existing Steam limit before automation begins and restores it when automation is disabled or the app exits normally.

## How it works

Steam Throttle reads Riot's local Live Client Data API at `https://127.0.0.1:2999` and checks the local League game process. Arena is identified by the `CHERRY` game mode or map 30. Combat is inferred from living players on both teams, while the local player's `isDead` state detects death and revival.

Steam is controlled through its own runtime console commands:

- `+set_download_throttle <Kbps> false`
- `+get_download_throttle`
- `+app_download_enable <0|1>`

The currently running `steam.exe` is preferred, so non-default and portable installations work without configuration. Steam Throttle does not start Steam or modify network adapters.

One Arena limitation remains: if your team is sitting out, another active pairing can still make the global round appear to be combat. Local death and revival are detected independently.

## Privacy

All detection and control happen locally. Steam Throttle does not send Riot credentials, Steam account information, player identity, telemetry, or diagnostics anywhere.

Settings are stored in `%LOCALAPPDATA%\SteamThrottle`.

## Build from source

Requirements:

- Windows 10 or 11, x64
- Rust stable
- Node.js 22 or newer
- npm
- Microsoft C++ Build Tools required by Tauri

```powershell
npm install
npm run format:check
npm test
npm run check

Set-Location src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
Set-Location ..

npm run tauri build
```

Run the development build with `npm run tauri dev`.

## Contributing

Issues and focused pull requests are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before sharing logs or League fixtures, and report security issues according to [SECURITY.md](SECURITY.md).

Steam Throttle is released under the [MIT License](LICENSE).

This is an independent community project and is not endorsed by or affiliated with Valve Corporation or Riot Games. Steam and League of Legends are trademarks of their respective owners.
