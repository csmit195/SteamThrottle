# Contributing

Contributions to Steam Throttle are welcome, especially focused fixes, tests, sanitized Arena telemetry fixtures, and documentation improvements.

## Before opening an issue

- Search existing issues first.
- Include the Steam Throttle version and Windows version.
- Describe the League mode and exact state transition involved.
- Remove Riot IDs, summoner names, auth tokens, Steam account information, and personal filesystem paths from logs or fixtures.
- Use GitHub's private security reporting for vulnerabilities.

## Development setup

Install Rust stable, Node.js 22 or newer, npm, and the Microsoft C++ Build Tools required by Tauri. Then run:

```powershell
./scripts/setup.ps1
./scripts/dev.ps1
```

Keep integrations within the existing League and Steam modules. Steam Throttle must remain local-only and must not add packet blocking, firewall or QoS changes, process injection, memory access, or administrator-only behavior.

Use decimal MB/s in the interface and bytes per second internally. Add a failing regression test before changing game-state classification, policy behavior, or Steam transitions.

## Before opening a pull request

Run the complete verification suite:

```powershell
./scripts/check.ps1
```

Keep pull requests focused. Explain the state observed, the Steam action expected, and how the change was verified. Pull requests require passing checks and maintainer approval before merge.

By contributing, you agree that your work is licensed under the repository's MIT License and that you will follow the [Code of Conduct](CODE_OF_CONDUCT.md).
