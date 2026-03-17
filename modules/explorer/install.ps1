#Requires -Version 5.1
<#
.SYNOPSIS
    Installs the Sovereign Explorer.
.DESCRIPTION
    - Copies the default config if none exists
    - Optionally registers file type associations
#>

param(
    [string]$BinaryPath = "$PSScriptRoot\src-tauri\target\release\sovereign-explorer.exe"
)

$ErrorActionPreference = 'Stop'
$ModuleName = 'explorer'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"
$ConfigFile = Join-Path $ConfigDir 'config.toml'

Write-Host "[install] Sovereign Explorer installer" -ForegroundColor Cyan

# ── Verify binary ────────────────────────────────────────────────────
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at: $BinaryPath`nBuild first with: cargo tauri build"
    exit 1
}

$BinaryPath = (Resolve-Path $BinaryPath).Path
Write-Host "[install] Binary: $BinaryPath" -ForegroundColor Gray

# ── Create config directory and copy default config ──────────────────
if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    Write-Host "[install] Created config directory: $ConfigDir" -ForegroundColor Green
}

$DefaultConfig = Join-Path $PSScriptRoot 'config.default.toml'
if ((Test-Path $DefaultConfig) -and (-not (Test-Path $ConfigFile))) {
    Copy-Item $DefaultConfig $ConfigFile
    Write-Host "[install] Copied default config to: $ConfigFile" -ForegroundColor Green
} elseif (Test-Path $ConfigFile) {
    Write-Host "[install] Config already exists, skipping" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[install] Installation complete." -ForegroundColor Cyan
Write-Host "  Config: $ConfigFile" -ForegroundColor Gray
Write-Host "  Binary: $BinaryPath" -ForegroundColor Gray
Write-Host ""
Write-Host "  To run: & '$BinaryPath'" -ForegroundColor White
