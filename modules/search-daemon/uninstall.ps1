#Requires -Version 5.1
<#
.SYNOPSIS
    Uninstalls the Sovereign Search Daemon.
.DESCRIPTION
    - Stops the running daemon process
    - Removes the startup scheduled task
    - Optionally removes config, data, and index
.PARAMETER RemoveData
    If set, also removes the config directory, database, and index.
#>

param(
    [switch]$RemoveData
)

$ErrorActionPreference = 'Stop'
$ModuleName = 'search-daemon'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"
$TaskName = 'SovereignShell-SearchDaemon'

Write-Host "[uninstall] Sovereign Search Daemon uninstaller" -ForegroundColor Cyan

# ── Stop running process ─────────────────────────────────────────────
$proc = Get-Process -Name 'sovereign-search-daemon' -ErrorAction SilentlyContinue
if ($proc) {
    Stop-Process -Name 'sovereign-search-daemon' -Force
    Write-Host "[uninstall] Stopped running daemon process" -ForegroundColor Yellow
} else {
    Write-Host "[uninstall] No running daemon process found" -ForegroundColor Gray
}

# ── Remove scheduled task ────────────────────────────────────────────
$existingTask = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($existingTask) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "[uninstall] Removed scheduled task: $TaskName" -ForegroundColor Green
} else {
    Write-Host "[uninstall] No scheduled task found" -ForegroundColor Gray
}

# ── Remove data (optional) ──────────────────────────────────────────
if ($RemoveData) {
    if (Test-Path $ConfigDir) {
        Remove-Item -Path $ConfigDir -Recurse -Force
        Write-Host "[uninstall] Removed config and data: $ConfigDir" -ForegroundColor Green
    }
} else {
    Write-Host "[uninstall] Config and data preserved at: $ConfigDir" -ForegroundColor Gray
    Write-Host "  To remove, re-run with -RemoveData" -ForegroundColor Gray
}

Write-Host ""
Write-Host "[uninstall] Uninstallation complete." -ForegroundColor Cyan
