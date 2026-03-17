#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Sovereign Shell — Module Installer
    Installs, registers, and configures individual Sovereign Shell modules.

.DESCRIPTION
    Reads module manifests and executes per-module install.ps1 scripts.
    Can install all modules or a specific one.

    Usage:
      install-modules.ps1                    # Install all modules with status >= alpha
      install-modules.ps1 -Module launcher   # Install specific module
      install-modules.ps1 -List              # Show all modules and their status

.VERSION
    0.1.0 — 2026-03-12
#>

param(
    [string]$Module = "",
    [switch]$List,
    [switch]$Force
)

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ModulesDir = Join-Path $RepoRoot "modules"
$ConfigRoot = "$env:APPDATA\SovereignShell"

# Ensure config root exists
if (-not (Test-Path $ConfigRoot)) {
    New-Item -ItemType Directory -Path $ConfigRoot -Force | Out-Null
}

function Get-ModuleManifest {
    param([string]$ModulePath)
    $manifestPath = Join-Path $ModulePath "manifest.toml"
    if (-not (Test-Path $manifestPath)) { return $null }

    # Simple TOML parser for our known structure
    $manifest = @{}
    $currentSection = ""
    foreach ($line in Get-Content $manifestPath) {
        $line = $line.Trim()
        if ($line -match '^\[(.+)\]$') {
            $currentSection = $Matches[1]
            if (-not $manifest.ContainsKey($currentSection)) {
                $manifest[$currentSection] = @{}
            }
        }
        elseif ($line -match '^(\w+)\s*=\s*"(.+)"$') {
            $manifest[$currentSection][$Matches[1]] = $Matches[2]
        }
        elseif ($line -match '^(\w+)\s*=\s*(\d+)$') {
            $manifest[$currentSection][$Matches[1]] = [int]$Matches[2]
        }
    }
    return $manifest
}

# List mode
if ($List) {
    Write-Host "`nSovereign Shell Modules:" -ForegroundColor Cyan
    Write-Host ("-" * 60)
    foreach ($dir in Get-ChildItem -Path $ModulesDir -Directory) {
        $m = Get-ModuleManifest $dir.FullName
        if ($m) {
            $status = $m["module"]["status"]
            $color = switch ($status) {
                "stable"      { "Green" }
                "beta"        { "Yellow" }
                "alpha"       { "Yellow" }
                "development" { "DarkYellow" }
                "scaffold"    { "DarkGray" }
                default       { "White" }
            }
            Write-Host ("  {0,-20} v{1,-10} [{2}]" -f $m["module"]["name"], $m["module"]["version"], $status) -ForegroundColor $color
            Write-Host ("  {0}" -f $m["module"]["description"]) -ForegroundColor Gray
            Write-Host ""
        } else {
            Write-Host ("  {0,-20} [NO MANIFEST]" -f $dir.Name) -ForegroundColor Red
        }
    }
    exit 0
}

# Install mode
$targetModules = if ($Module) {
    @(Join-Path $ModulesDir $Module)
} else {
    Get-ChildItem -Path $ModulesDir -Directory | Select-Object -ExpandProperty FullName
}

foreach ($modPath in $targetModules) {
    $modName = Split-Path -Leaf $modPath
    $manifest = Get-ModuleManifest $modPath

    if (-not $manifest) {
        Write-Host "[$modName] No manifest.toml found — skipping" -ForegroundColor Yellow
        continue
    }

    $status = $manifest["module"]["status"]
    if ($status -eq "scaffold" -and -not $Force) {
        Write-Host "[$modName] Status is 'scaffold' — skipping (use -Force to override)" -ForegroundColor DarkGray
        continue
    }

    $installScript = Join-Path $modPath "install.ps1"
    if (-not (Test-Path $installScript)) {
        Write-Host "[$modName] No install.ps1 found — skipping" -ForegroundColor Yellow
        continue
    }

    Write-Host "[$modName] Installing v$($manifest["module"]["version"])..." -ForegroundColor Cyan
    try {
        & $installScript
        Write-Host "[$modName] Installed successfully" -ForegroundColor Green
    } catch {
        Write-Host "[$modName] Installation failed: $_" -ForegroundColor Red
    }
}

Write-Host "`nDone. Use -List to see module status." -ForegroundColor Cyan
