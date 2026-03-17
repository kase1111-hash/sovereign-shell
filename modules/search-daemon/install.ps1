#Requires -Version 5.1
<#
.SYNOPSIS
    Installs the Sovereign Search Daemon.
.DESCRIPTION
    - Copies the default config if none exists
    - Registers the daemon as a scheduled task (runs at login, restarts on failure)
.PARAMETER BinaryPath
    Path to the built sovereign-search-daemon.exe.
#>

param(
    [string]$BinaryPath = "$PSScriptRoot\target\release\sovereign-search-daemon.exe"
)

$ErrorActionPreference = 'Stop'
$ModuleName = 'search-daemon'
$AppData = [Environment]::GetFolderPath('ApplicationData')
$ConfigDir = Join-Path $AppData "SovereignShell\$ModuleName"
$ConfigFile = Join-Path $ConfigDir 'config.toml'
$TaskName = 'SovereignShell-SearchDaemon'

Write-Host "[install] Sovereign Search Daemon installer" -ForegroundColor Cyan

# ── Verify binary ────────────────────────────────────────────────────
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at: $BinaryPath`nBuild first with: cargo build --release"
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

# ── Register startup task ────────────────────────────────────────────
$existingTask = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($existingTask) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "[install] Removed existing scheduled task" -ForegroundColor Yellow
}

$action = New-ScheduledTaskAction -Execute $BinaryPath
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit ([TimeSpan]::Zero) `
    -RestartCount 3 `
    -RestartInterval (New-TimeSpan -Minutes 1)

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action $action `
    -Trigger $trigger `
    -Settings $settings `
    -Description "Sovereign Shell Search Daemon — file indexing and search service" | Out-Null

Write-Host "[install] Registered startup task: $TaskName" -ForegroundColor Green

# ── Done ─────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "[install] Installation complete." -ForegroundColor Cyan
Write-Host "  Config: $ConfigFile" -ForegroundColor Gray
Write-Host "  Binary: $BinaryPath" -ForegroundColor Gray
Write-Host "  Startup: Scheduled task '$TaskName' (runs at login)" -ForegroundColor Gray
Write-Host ""
Write-Host "  To start now: & '$BinaryPath'" -ForegroundColor White
