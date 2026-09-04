# Contributing

Thanks for helping improve SteamThrottle. Bug reports, sanitized telemetry fixtures, documentation, tests, and focused pull requests are welcome.

## Ground rules

- Do not include summoner names, Riot IDs, auth tokens, Steam account data, or absolute personal paths in issues or fixtures.
- Do not add packet blocking, firewall/QoS manipulation, process injection, memory access, or administrator-only behavior.
- Keep League and Steam integrations behind testable adapters.
- Add a failing regression test before changing state classification, rules, or Steam transitions.
- Use decimal MB/s in the UI and bytes per second internally.

## Pull requests

Create a focused branch, run `npm test`, `npm run check`, `npm run build`, `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`, then open a pull request explaining the observed state and expected Steam action. A passing CI run and approval from `csmit195` are required before merge.
