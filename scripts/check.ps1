$ErrorActionPreference = "Stop"
$repository = Split-Path $PSScriptRoot -Parent
$env:CARGO_TARGET_DIR = Join-Path ([Environment]::GetFolderPath("LocalApplicationData")) "SteamThrottle\build-cache"

Push-Location $repository
try {
    npm --prefix ui run format:check
    npm --prefix ui test
    npm --prefix ui run check
    npm --prefix ui run build
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
} finally {
    Pop-Location
}
