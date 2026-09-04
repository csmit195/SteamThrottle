$ErrorActionPreference = "Stop"
$repository = Split-Path $PSScriptRoot -Parent

Push-Location (Join-Path $repository "ui")
try {
    npm install
} finally {
    Pop-Location
}
