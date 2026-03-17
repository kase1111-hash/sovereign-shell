#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Sovereign Shell — Windows 10 Debloat Script
    Phase 1: Strip telemetry, advertising, bloatware, and hostile defaults.

.DESCRIPTION
    This script prepares a Windows 10 installation for the Sovereign Shell
    module layer by removing components that serve Microsoft's interests
    over the user's. It creates a restore point first and logs all actions.

    Run with: PowerShell -ExecutionPolicy Bypass -File debloat.ps1
    Flags:
      -DryRun         Show what would be done without making changes
      -SkipRestore     Skip restore point creation
      -Aggressive      Also disable Windows Defender telemetry (use with caution)

.AUTHOR
    Kase — github.com/kase1111-hash
    Maintained via Claude Code

.VERSION
    0.1.0 — 2026-03-12 — Initial scaffold
#>

param(
    [switch]$DryRun,
    [switch]$SkipRestore,
    [switch]$Aggressive
)

# --- Configuration -----------------------------------------------------------

$LogFile = "$env:USERPROFILE\SovereignShell-Debloat.log"
$Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $entry = "[$Timestamp] [$Level] $Message"
    Write-Host $entry
    Add-Content -Path $LogFile -Value $entry
}

function Invoke-Action {
    param([string]$Description, [scriptblock]$Action)
    if ($DryRun) {
        Write-Log "[DRY RUN] Would execute: $Description"
    } else {
        try {
            Write-Log "Executing: $Description"
            & $Action
            Write-Log "  -> Success"
        } catch {
            Write-Log "  -> FAILED: $_" -Level "ERROR"
        }
    }
}

# --- Phase 0: Snapshot --------------------------------------------------------

Write-Log "=========================================="
Write-Log "Sovereign Shell Debloat — v0.1.0"
Write-Log "=========================================="
Write-Log "DryRun: $DryRun | SkipRestore: $SkipRestore | Aggressive: $Aggressive"

if (-not $SkipRestore) {
    Invoke-Action "Create system restore point" {
        Enable-ComputerRestore -Drive "C:\" -ErrorAction SilentlyContinue
        Checkpoint-Computer -Description "Pre-SovereignShell-Debloat" -RestorePointType MODIFY_SETTINGS
    }
}

# --- Phase 1: Telemetry and Advertising ---------------------------------------

Write-Log "--- Phase 1: Telemetry and Advertising ---"

# Disable telemetry service
Invoke-Action "Disable Connected User Experiences and Telemetry (DiagTrack)" {
    Stop-Service -Name "DiagTrack" -Force -ErrorAction SilentlyContinue
    Set-Service -Name "DiagTrack" -StartupType Disabled
}

# Disable dmwappushservice (WAP Push Message Routing)
Invoke-Action "Disable WAP Push Message Routing Service" {
    Stop-Service -Name "dmwappushservice" -Force -ErrorAction SilentlyContinue
    Set-Service -Name "dmwappushservice" -StartupType Disabled
}

# Set telemetry level to Security (0 = off on Enterprise, minimal elsewhere)
Invoke-Action "Set telemetry to minimum level" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\DataCollection"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "AllowTelemetry" -Value 0 -Type DWord
}

# Disable advertising ID
Invoke-Action "Disable Advertising ID" {
    $path = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "Enabled" -Value 0 -Type DWord
}

# Disable Cortana
Invoke-Action "Disable Cortana" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Search"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "AllowCortana" -Value 0 -Type DWord
}

# Disable Start Menu suggestions / ads
Invoke-Action "Disable Start Menu suggested apps" {
    $path = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager"
    $properties = @{
        "SystemPaneSuggestionsEnabled"      = 0
        "SubscribedContent-338388Enabled"   = 0
        "SubscribedContent-338389Enabled"   = 0
        "SubscribedContent-310093Enabled"   = 0
        "SubscribedContent-338393Enabled"   = 0
        "SilentInstalledAppsEnabled"        = 0
        "SoftLandingEnabled"                = 0
        "RotatingLockScreenEnabled"         = 0
        "RotatingLockScreenOverlayEnabled"  = 0
    }
    foreach ($prop in $properties.GetEnumerator()) {
        Set-ItemProperty -Path $path -Name $prop.Key -Value $prop.Value -Type DWord -ErrorAction SilentlyContinue
    }
}

# Disable feedback notifications
Invoke-Action "Disable feedback frequency" {
    $path = "HKCU:\SOFTWARE\Microsoft\Siuf\Rules"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "NumberOfSIUFInPeriod" -Value 0 -Type DWord
}

# Disable tips and tricks notifications
Invoke-Action "Disable Windows Tips" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\CloudContent"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "DisableSoftLanding" -Value 1 -Type DWord
    Set-ItemProperty -Path $path -Name "DisableWindowsConsumerFeatures" -Value 1 -Type DWord
}

# --- Phase 2: Bloatware Removal -----------------------------------------------

Write-Log "--- Phase 2: Bloatware Removal ---"

# UWP apps to remove — these serve Microsoft, not the user
$BloatApps = @(
    "Microsoft.3DBuilder"
    "Microsoft.BingFinance"
    "Microsoft.BingNews"
    "Microsoft.BingSports"
    "Microsoft.BingWeather"
    "Microsoft.GamingApp"
    "Microsoft.GetHelp"
    "Microsoft.Getstarted"
    "Microsoft.MicrosoftOfficeHub"
    "Microsoft.MicrosoftSolitaireCollection"
    "Microsoft.MixedReality.Portal"
    "Microsoft.OneConnect"
    "Microsoft.People"
    "Microsoft.SkypeApp"
    "Microsoft.Wallet"
    "Microsoft.WindowsAlarms"
    "Microsoft.WindowsCommunicationsApps"  # Mail & Calendar
    "Microsoft.WindowsFeedbackHub"
    "Microsoft.WindowsMaps"
    "Microsoft.Xbox.TCUI"
    "Microsoft.XboxApp"
    "Microsoft.XboxGameOverlay"
    "Microsoft.XboxGamingOverlay"
    "Microsoft.XboxIdentityProvider"
    "Microsoft.XboxSpeechToTextOverlay"
    "Microsoft.YourPhone"
    "Microsoft.ZuneMusic"
    "Microsoft.ZuneVideo"
    "Microsoft.Todos"
    "Microsoft.PowerAutomateDesktop"
    "Microsoft.549981C3F5F10"  # Cortana standalone app
    "MicrosoftTeams"
    "Clipchamp.Clipchamp"
    "king.com.CandyCrushSaga"
    "king.com.CandyCrushSodaSaga"
    "SpotifyAB.SpotifyMusic"
    "Disney.37853FC22B2CE"
    "Facebook.Facebook"
    "BytedancePte.Ltd.TikTok"
)

# Apps to KEEP — functional, non-hostile
$KeepApps = @(
    "Microsoft.WindowsCalculator"
    "Microsoft.WindowsStore"           # Keep for driver updates if needed
    "Microsoft.ScreenSketch"           # Snipping Tool
    "Microsoft.WindowsNotepad"
    "Microsoft.WindowsTerminal"
    "Microsoft.DesktopAppInstaller"    # winget
    "Microsoft.Paint"
)

foreach ($app in $BloatApps) {
    Invoke-Action "Remove UWP app: $app" {
        Get-AppxPackage -Name $app -AllUsers -ErrorAction SilentlyContinue |
            Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue
        Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue |
            Where-Object { $_.DisplayName -eq $app } |
            Remove-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue
    }
}

# --- Phase 3: Service Hardening -----------------------------------------------

Write-Log "--- Phase 3: Service Hardening ---"

$DisableServices = @(
    @{ Name = "DiagTrack";              Desc = "Connected User Experiences and Telemetry" }
    @{ Name = "dmwappushservice";       Desc = "WAP Push Message Routing" }
    @{ Name = "RetailDemo";             Desc = "Retail Demo Service" }
    @{ Name = "MapsBroker";             Desc = "Downloaded Maps Manager" }
    @{ Name = "lfsvc";                  Desc = "Geolocation Service" }
    @{ Name = "SharedAccess";           Desc = "Internet Connection Sharing" }
    @{ Name = "RemoteRegistry";         Desc = "Remote Registry" }
    @{ Name = "WMPNetworkSvc";          Desc = "Windows Media Player Sharing" }
    @{ Name = "WSearch";                Desc = "Windows Search (will be replaced by search-daemon)" }
    @{ Name = "XblAuthManager";         Desc = "Xbox Live Auth Manager" }
    @{ Name = "XblGameSave";            Desc = "Xbox Live Game Save" }
    @{ Name = "XboxNetApiSvc";          Desc = "Xbox Live Networking" }
    @{ Name = "XboxGipSvc";             Desc = "Xbox Accessory Management" }
)

foreach ($svc in $DisableServices) {
    Invoke-Action "Disable service: $($svc.Name) ($($svc.Desc))" {
        Stop-Service -Name $svc.Name -Force -ErrorAction SilentlyContinue
        Set-Service -Name $svc.Name -StartupType Disabled -ErrorAction SilentlyContinue
    }
}

# Tame Windows Update — notify only, no forced restarts
Invoke-Action "Set Windows Update to notify-only (no auto-restart)" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "NoAutoRebootWithLoggedOnUsers" -Value 1 -Type DWord
    Set-ItemProperty -Path $path -Name "AUOptions" -Value 2 -Type DWord  # 2 = Notify before download
}

# --- Phase 4: Privacy Lockdown ------------------------------------------------

Write-Log "--- Phase 4: Privacy Lockdown ---"

# Disable Activity History
Invoke-Action "Disable Activity History" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\System"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "EnableActivityFeed" -Value 0 -Type DWord
    Set-ItemProperty -Path $path -Name "PublishUserActivities" -Value 0 -Type DWord
    Set-ItemProperty -Path $path -Name "UploadUserActivities" -Value 0 -Type DWord
}

# Disable Timeline
Invoke-Action "Disable Timeline" {
    Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\System" -Name "EnableActivityFeed" -Value 0 -Type DWord -ErrorAction SilentlyContinue
}

# Disable Cloud Clipboard
Invoke-Action "Disable Cloud Clipboard sync" {
    $path = "HKCU:\SOFTWARE\Microsoft\Clipboard"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "EnableClipboardHistory" -Value 1 -Type DWord   # Keep local clipboard history
    Set-ItemProperty -Path $path -Name "EnableCloudClipboard" -Value 0 -Type DWord     # Kill cloud sync
}

# Disable handwriting data sharing
Invoke-Action "Disable handwriting error reporting" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\HandwritingErrorReports"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "PreventHandwritingErrorReports" -Value 1 -Type DWord
}

# Disable input personalization (speech, inking, typing)
Invoke-Action "Disable input personalization / speech data sharing" {
    $path = "HKCU:\SOFTWARE\Microsoft\InputPersonalization"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "RestrictImplicitInkCollection" -Value 1 -Type DWord
    Set-ItemProperty -Path $path -Name "RestrictImplicitTextCollection" -Value 1 -Type DWord

    $path2 = "HKCU:\SOFTWARE\Microsoft\InputPersonalization\TrainedDataStore"
    if (-not (Test-Path $path2)) { New-Item -Path $path2 -Force | Out-Null }
    Set-ItemProperty -Path $path2 -Name "HarvestContacts" -Value 0 -Type DWord
}

# Disable Customer Experience Improvement Program
Invoke-Action "Disable CEIP" {
    $path = "HKLM:\SOFTWARE\Policies\Microsoft\SQMClient\Windows"
    if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
    Set-ItemProperty -Path $path -Name "CEIPEnable" -Value 0 -Type DWord
}

# Block known telemetry endpoints via hosts file (conservative list)
Invoke-Action "Block telemetry endpoints via hosts file" {
    $hostsPath = "$env:SystemRoot\System32\drivers\etc\hosts"
    $telemetryHosts = @(
        "vortex.data.microsoft.com"
        "vortex-win.data.microsoft.com"
        "telecommand.telemetry.microsoft.com"
        "telecommand.telemetry.microsoft.com.nsatc.net"
        "oca.telemetry.microsoft.com"
        "oca.telemetry.microsoft.com.nsatc.net"
        "sqm.telemetry.microsoft.com"
        "sqm.telemetry.microsoft.com.nsatc.net"
        "watson.telemetry.microsoft.com"
        "watson.telemetry.microsoft.com.nsatc.net"
        "redir.metaservices.microsoft.com"
        "choice.microsoft.com"
        "choice.microsoft.com.nsatc.net"
        "settings-sandbox.data.microsoft.com"
        "watson.live.com"
        "statsfe2.ws.microsoft.com"
        "corpext.msitadfs.glbdns2.microsoft.com"
        "compatexchange.cloudapp.net"
        "a-0001.a-msedge.net"
        "statsfe2.update.microsoft.com.akadns.net"
    )

    $marker = "# --- Sovereign Shell Telemetry Block ---"
    $currentHosts = Get-Content $hostsPath -Raw -ErrorAction SilentlyContinue

    if ($currentHosts -notlike "*$marker*") {
        $block = "`n$marker`n"
        foreach ($h in $telemetryHosts) {
            $block += "0.0.0.0 $h`n"
        }
        $block += "# --- End Sovereign Shell Block ---`n"
        Add-Content -Path $hostsPath -Value $block
    }
}

# --- Aggressive Mode (optional) -----------------------------------------------

if ($Aggressive) {
    Write-Log "--- Aggressive Mode ---"

    Invoke-Action "Disable Windows Defender sample submission" {
        $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows Defender\Spynet"
        if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
        Set-ItemProperty -Path $path -Name "SubmitSamplesConsent" -Value 2 -Type DWord  # 2 = Never send
        Set-ItemProperty -Path $path -Name "SpynetReporting" -Value 0 -Type DWord       # 0 = Disabled
    }

    Invoke-Action "Disable Windows Error Reporting" {
        $path = "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting"
        if (-not (Test-Path $path)) { New-Item -Path $path -Force | Out-Null }
        Set-ItemProperty -Path $path -Name "Disabled" -Value 1 -Type DWord
    }
}

# --- Summary ------------------------------------------------------------------

Write-Log "=========================================="
Write-Log "Debloat complete."
Write-Log "Log saved to: $LogFile"
Write-Log "Restart recommended to apply all changes."
Write-Log "=========================================="

if ($DryRun) {
    Write-Log "This was a DRY RUN. No changes were made."
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Restart your machine" -ForegroundColor White
Write-Host "  2. Run 'scripts/install-modules.ps1' to begin installing Sovereign Shell modules" -ForegroundColor White
Write-Host "  3. See CONSTITUTION.md for governance and update methodology" -ForegroundColor White
