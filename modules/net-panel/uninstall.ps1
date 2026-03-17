# Sovereign Shell — Network Panel uninstaller
$ErrorActionPreference = "Stop"
$configDir = "$env:APPDATA\SovereignShell\net-panel"

Write-Host "Uninstalling Network Panel..." -ForegroundColor Cyan

if (Test-Path $configDir) {
    $confirm = Read-Host "Remove configuration at $configDir? (y/N)"
    if ($confirm -eq 'y') {
        Remove-Item -Recurse -Force $configDir
        Write-Host "  Configuration removed." -ForegroundColor Green
    } else {
        Write-Host "  Configuration preserved." -ForegroundColor Yellow
    }
}

Write-Host "Network Panel uninstalled." -ForegroundColor Green
