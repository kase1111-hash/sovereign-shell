#Requires -Version 5.1
<#
.SYNOPSIS
    Uninstalls the Sovereign Launcher module.
.DESCRIPTION
    - Stops the running launcher process
    - Removes the startup scheduled task
    - Optionally removes config and data
.PARAMETER RemoveData
    If set, also removes the config directory and database.
#>

param(
    [switch]$RemoveData
)

$ErrorActionPreference = 'Stop'
$ModuleName = 'launcher'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"
$TaskName = 'SovereignShell-Launcher'

Write-Host "[uninstall] Sovereign Launcher uninstaller" -ForegroundColor Cyan

# ── Stop running process ─────────────────────────────────────────────
$proc = Get-Process -Name 'sovereign-launcher' -ErrorAction SilentlyContinue
if ($proc) {
    Stop-Process -Name 'sovereign-launcher' -Force
    Write-Host "[uninstall] Stopped running launcher process" -ForegroundColor Yellow
} else {
    Write-Host "[uninstall] No running launcher process found" -ForegroundColor Gray
}

# ── Remove scheduled task ────────────────────────────────────────────
$existingTask = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($existingTask) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "[uninstall] Removed scheduled task: $TaskName" -ForegroundColor Green
} else {
    Write-Host "[uninstall] No scheduled task found: $TaskName" -ForegroundColor Gray
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

# ── Done ─────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[uninstall] Uninstallation complete." -ForegroundColor Cyan
