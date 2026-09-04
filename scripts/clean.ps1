$ErrorActionPreference = "Stop"
$repository = Split-Path $PSScriptRoot -Parent
$externalTarget = Join-Path ([Environment]::GetFolderPath("LocalApplicationData")) "SteamThrottle\build-cache"

Push-Location $repository
try {
    cargo clean
    if (Test-Path -LiteralPath $externalTarget) {
        cargo clean --target-dir $externalTarget
    }
    foreach ($path in @("gen", "ui/build", "ui/.svelte-kit")) {
        if (Test-Path -LiteralPath $path) {
            [System.IO.Directory]::Delete((Resolve-Path -LiteralPath $path).Path, $true)
        }
    }
} finally {
    Pop-Location
}
