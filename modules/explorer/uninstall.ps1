#Requires -Version 5.1
<#
.SYNOPSIS
    Uninstalls the Sovereign Explorer.
.PARAMETER RemoveData
    If set, also removes the config directory and data.
#>

param(
    [switch]$RemoveData
)

$ErrorActionPreference = 'Stop'
$ModuleName = 'explorer'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"

Write-Host "[uninstall] Sovereign Explorer uninstaller" -ForegroundColor Cyan

# ── Stop running process ─────────────────────────────────────────────
$proc = Get-Process -Name 'sovereign-explorer' -ErrorAction SilentlyContinue
if ($proc) {
    Stop-Process -Name 'sovereign-explorer' -Force
    Write-Host "[uninstall] Stopped running process" -ForegroundColor Yellow
}

# ── Remove data (optional) ──────────────────────────────────────────
if ($RemoveData) {
    if (Test-Path $ConfigDir) {
        Remove-Item -Path $ConfigDir -Recurse -Force
        Write-Host "[uninstall] Removed config and data: $ConfigDir" -ForegroundColor Green
    }
} else {
    Write-Host "[uninstall] Config preserved at: $ConfigDir" -ForegroundColor Gray
}

Write-Host ""
Write-Host "[uninstall] Uninstallation complete." -ForegroundColor Cyan
