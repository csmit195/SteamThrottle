# Changelog

All notable changes to Steam Throttle are documented here.

## 0.1.1 - 2026-09-05

- Fix the portable Windows build opening a white window followed by "localhost refused to connect". Production builds now use the Tauri CLI to embed the interface.
- Verify packaged executable startup in CI and release builds, including interface rendering and receipt of the backend snapshot without a development server.
- Resolve the Tauri CLI from the frontend directory when building release installers.

## 0.1.0 - 2026-09-05

- Detect League and Arena state through local Riot APIs and process state.
- Limit Steam downloads during Arena combat while the local player is alive.
- Restore full download speed between rounds and after death.
- Pause downloads during other League modes.
- Detect the running Steam installation automatically.
- Preserve and restore the user's previous Steam throttle.
- Provide a compact native Windows interface, tray support, autostart, and update checks.
