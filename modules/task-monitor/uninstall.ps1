#Requires -Version 5.1
<#
.SYNOPSIS
    Uninstalls the Sovereign Task Monitor.
.PARAMETER RemoveData
    If set, also removes the config directory.
#>

param([switch]$RemoveData)

$ErrorActionPreference = 'Stop'
$ModuleName = 'task-monitor'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"

Write-Host "[uninstall] Sovereign Task Monitor uninstaller" -ForegroundColor Cyan

$proc = Get-Process -Name 'sovereign-task-monitor' -ErrorAction SilentlyContinue
if ($proc) {
    Stop-Process -Name 'sovereign-task-monitor' -Force
    Write-Host "[uninstall] Stopped running process" -ForegroundColor Yellow
}

if ($RemoveData -and (Test-Path $ConfigDir)) {
    Remove-Item -Path $ConfigDir -Recurse -Force
    Write-Host "[uninstall] Removed config: $ConfigDir" -ForegroundColor Green
}

Write-Host "[uninstall] Uninstallation complete." -ForegroundColor Cyan
