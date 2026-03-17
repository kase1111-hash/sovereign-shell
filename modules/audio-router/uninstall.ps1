#Requires -Version 5.1
param([switch]$RemoveData)

$ErrorActionPreference = 'Stop'
$ModuleName = 'audio-router'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"

Write-Host "[uninstall] Sovereign Audio Router uninstaller" -ForegroundColor Cyan

$proc = Get-Process -Name 'sovereign-audio-router' -ErrorAction SilentlyContinue
if ($proc) {
    Stop-Process -Name 'sovereign-audio-router' -Force
    Write-Host "[uninstall] Stopped running process" -ForegroundColor Yellow
}

if ($RemoveData -and (Test-Path $ConfigDir)) {
    Remove-Item -Path $ConfigDir -Recurse -Force
    Write-Host "[uninstall] Removed config: $ConfigDir" -ForegroundColor Green
}

Write-Host "[uninstall] Uninstallation complete." -ForegroundColor Cyan
