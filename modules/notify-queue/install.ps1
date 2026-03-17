# Sovereign Shell — Notification Queue installer
$ErrorActionPreference = "Stop"
$configDir = "$env:APPDATA\SovereignShell\notify-queue"

Write-Host "Installing Notification Queue..." -ForegroundColor Cyan

if (-not (Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
}

$defaultConfig = Join-Path $PSScriptRoot "config.default.toml"
$targetConfig = Join-Path $configDir "config.toml"

if (-not (Test-Path $targetConfig)) {
    Copy-Item $defaultConfig $targetConfig
    Write-Host "  Default configuration installed." -ForegroundColor Green
} else {
    Write-Host "  Configuration already exists, skipping." -ForegroundColor Yellow
}

Write-Host "Notification Queue installed successfully." -ForegroundColor Green
