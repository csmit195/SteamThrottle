# Changelog

All notable changes to Steam Throttle are documented here.

## 0.1.2 - 2026-09-11

- Throttle instead of pausing while a match is loading, telemetry is uncertain, or Arena download windows are disabled.
- Make pausing an explicit non-Arena option; existing settings migrate once to bandwidth limiting so pausing must be re-enabled deliberately.
- Serialize Steam transitions and reject stale actions so outdated pause commands cannot race newer game state or settings.
- Redesign Behavior and Settings with native-style panels, switches, and silent automatic saving.
- Add a compact bandwidth textbox with direct entry, a progress overlay, responsive 0.5 MB/s drag adjustment, dynamic content sizing, and saving only when editing finishes.
- Route the custom close button through close-to-tray behavior instead of exiting the application.
- Add a root `pnpm dev` command that starts the complete Tauri development application.

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
