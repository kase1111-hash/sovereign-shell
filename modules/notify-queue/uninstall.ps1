# Sovereign Shell — Notification Queue uninstaller
$ErrorActionPreference = "Stop"
$configDir = "$env:APPDATA\SovereignShell\notify-queue"

Write-Host "Uninstalling Notification Queue..." -ForegroundColor Cyan

if (Test-Path $configDir) {
    $confirm = Read-Host "Remove configuration and history at $configDir? (y/N)"
    if ($confirm -eq 'y') {
        Remove-Item -Recurse -Force $configDir
        Write-Host "  Configuration and history removed." -ForegroundColor Green
    } else {
        Write-Host "  Configuration preserved." -ForegroundColor Yellow
    }
}

Write-Host "Notification Queue uninstalled." -ForegroundColor Green
