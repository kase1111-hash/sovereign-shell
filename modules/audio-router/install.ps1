#Requires -Version 5.1
param(
    [string]$BinaryPath = "$PSScriptRoot\src-tauri\target\release\sovereign-audio-router.exe"
)

$ErrorActionPreference = 'Stop'
$ModuleName = 'audio-router'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"
$ConfigFile = Join-Path $ConfigDir 'config.toml'

Write-Host "[install] Sovereign Audio Router installer" -ForegroundColor Cyan

if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at: $BinaryPath`nBuild first with: cargo tauri build"
    exit 1
}

$BinaryPath = (Resolve-Path $BinaryPath).Path

if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    Write-Host "[install] Created config directory: $ConfigDir" -ForegroundColor Green
}

$DefaultConfig = Join-Path $PSScriptRoot 'config.default.toml'
if ((Test-Path $DefaultConfig) -and (-not (Test-Path $ConfigFile))) {
    Copy-Item $DefaultConfig $ConfigFile
    Write-Host "[install] Copied default config to: $ConfigFile" -ForegroundColor Green
}

Write-Host ""
Write-Host "[install] Installation complete." -ForegroundColor Cyan
Write-Host "  Config: $ConfigFile" -ForegroundColor Gray
Write-Host "  Binary: $BinaryPath" -ForegroundColor Gray
Write-Host "  To run: & '$BinaryPath'" -ForegroundColor White
