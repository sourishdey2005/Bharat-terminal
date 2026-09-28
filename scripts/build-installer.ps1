# Bharat Terminal — Windows Installer Builder
# Author: Sourish Dey

Write-Host "Building Bharat Terminal v3..." -ForegroundColor Cyan

cargo build --release --workspace

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Installing cargo-bundle..." -ForegroundColor Yellow
cargo install cargo-bundle --quiet

Write-Host "Creating Windows bundle..." -ForegroundColor Yellow
cargo bundle --release -p bt-app

$bundleDir = "target\release\bundle\windows"
Write-Host "Bundle created at: $bundleDir" -ForegroundColor Green

Write-Host "Creating portable ZIP..." -ForegroundColor Yellow
$zipPath = "target\release\BharatTerminal-v3.0.0-portable.zip"
Compress-Archive -Path "$bundleDir\*" -DestinationPath $zipPath -Force

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "BUILD COMPLETE" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host " Installer: $bundleDir\Bharat Terminal.msi"
Write-Host " Portable:  $zipPath"
Write-Host " Made by Sourish Dey" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Green
