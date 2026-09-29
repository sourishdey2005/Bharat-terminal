# Bharat Terminal — Windows Installer Builder
# Author: Sourish Dey
#
# Builds the release binaries (with embedded icons) and packages them into a
# per-user MSI with branded dialogs using the WiX Toolset v5.
#
# Prerequisites (one time):
#   dotnet tool install --global wix --version 5.0.2
#   wix extension add --global WixToolset.UI.wixext/5.0.2
#
# Usage: run from the repository root.
#   powershell -ExecutionPolicy Bypass -File scripts/build-installer.ps1

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$project = Join-Path $root "bharat-terminal"
$installerDir = Join-Path $project "installer"
$msiOut = Join-Path $project "releases\BharatTerminal-v3.0.0.msi"

Write-Host "Building Bharat Terminal v3 (release)..." -ForegroundColor Cyan
cargo build --release --workspace --manifest-path (Join-Path $project "Cargo.toml")

Write-Host "Packaging per-user MSI..." -ForegroundColor Yellow
Push-Location $installerDir
try {
    wix build BharatTerminal.wxs `
        -o $msiOut `
        -ext WixToolset.UI.wixext `
        -arch x64 `
        -d AppVersion=3.0.0
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "BUILD COMPLETE" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host " Installer: $msiOut"
Write-Host " Portable:  $(Join-Path $project 'releases\BharatTerminal-v3.0.0.exe')"
Write-Host " Made by Sourish Dey" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Green
