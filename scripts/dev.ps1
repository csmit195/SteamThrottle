$ErrorActionPreference = "Stop"
$repository = Split-Path $PSScriptRoot -Parent
$env:CARGO_TARGET_DIR = Join-Path ([Environment]::GetFolderPath("LocalApplicationData")) "SteamThrottle\build-cache"

Push-Location $repository
try {
    & (Join-Path $repository "ui\node_modules\.bin\tauri.cmd") dev
} finally {
    Pop-Location
}
