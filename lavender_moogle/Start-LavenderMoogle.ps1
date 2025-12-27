##############################################################################
# Lavender Moogle Launcher
#
# Purpose: Launch the Rust cargo project and AutoHotkey script for FFXIV
#          rotation automation using shared memory communication.
#
# Requirements:
#   - Must be placed in lavender_moogle/ directory
#   - AutoHotkey v2.0 installed
#   - Rust toolchain (cargo) installed
#   - Run as Administrator for proper functionality
#
# Portability:
#   - Uses only relative paths - works on any computer without modification
#   - Auto-detects AutoHotkey installation location
#   - No hardcoded paths
##############################################################################

[CmdletBinding()]
param()

# Set error action preference
$ErrorActionPreference = "Stop"

##############################################################################
# SECTION 1: Environment Validation
##############################################################################

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Lavender Moogle Launcher" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Get script directory (should be lavender_moogle/)
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "WARNING: This script should be run as Administrator." -ForegroundColor Yellow
    Write-Host "Shared memory operations and keypress simulation may fail without admin privileges.`n" -ForegroundColor Yellow
    $continue = Read-Host "Continue anyway? (y/N)"
    if ($continue -ne 'y' -and $continue -ne 'Y') {
        Write-Host "Exiting..." -ForegroundColor Gray
        exit 0
    }
    Write-Host ""
}

##############################################################################
# SECTION 2: File Path Validation
##############################################################################

Write-Host "Validating project structure..." -ForegroundColor Cyan

# Define required file paths using relative paths
$cargoTomlPath = Join-Path $scriptDir "Cargo.toml"
$ahkScriptPath = Join-Path $scriptDir "lavender_moogle.ahk"
$parentDir = Split-Path -Parent $scriptDir
$xInputPath = Join-Path $parentDir "XInput.ahk"

# Validate files exist
$validationErrors = @()

if (-not (Test-Path $cargoTomlPath)) {
    $validationErrors += "  - Cargo.toml not found at: $cargoTomlPath"
}

if (-not (Test-Path $ahkScriptPath)) {
    $validationErrors += "  - lavender_moogle.ahk not found at: $ahkScriptPath"
}

if (-not (Test-Path $xInputPath)) {
    $validationErrors += "  - XInput.ahk not found at: $xInputPath (required by AHK script)"
}

if ($validationErrors.Count -gt 0) {
    Write-Host "ERROR: Project structure validation failed:" -ForegroundColor Red
    $validationErrors | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    Write-Host "`nPlease ensure the script is placed in the lavender_moogle/ directory." -ForegroundColor Yellow
    exit 1
}

Write-Host "  All required files found" -ForegroundColor Green

##############################################################################
# SECTION 3: Tool Detection
##############################################################################

Write-Host "`nDetecting required tools..." -ForegroundColor Cyan

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Cargo not found in PATH." -ForegroundColor Red
    Write-Host "Please install Rust toolchain from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}
$cargoVersion = cargo --version
Write-Host "  Cargo found: $cargoVersion" -ForegroundColor Green

# Find AutoHotkey v2.0
$ahkSearchPaths = @(
    "C:\Program Files\AutoHotkey\v2\AutoHotkey.exe",
    "C:\Program Files\AutoHotkey\AutoHotkey64.exe",
    "C:\Program Files\AutoHotkey\AutoHotkey.exe",
    "C:\Program Files (x86)\AutoHotkey\v2\AutoHotkey.exe",
    "C:\Program Files (x86)\AutoHotkey\AutoHotkey.exe",
    "$env:LOCALAPPDATA\Programs\AutoHotkey\v2\AutoHotkey.exe",
    "$env:LOCALAPPDATA\Programs\AutoHotkey\AutoHotkey.exe",
    "C:\ProgramData\AutoHotkey\v2\AutoHotkey.exe"
)

$ahkExePath = $ahkSearchPaths | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $ahkExePath) {
    Write-Host "ERROR: AutoHotkey v2.0 not found." -ForegroundColor Red
    Write-Host "Please install from https://www.autohotkey.com/" -ForegroundColor Yellow
    Write-Host "`nSearched locations:" -ForegroundColor Gray
    $ahkSearchPaths | ForEach-Object { Write-Host "  - $_" -ForegroundColor DarkGray }
    exit 1
}
Write-Host "  AutoHotkey found: $ahkExePath" -ForegroundColor Green

##############################################################################
# SECTION 4: Launch Rust Application
##############################################################################

Write-Host "`nStarting Rust application (cargo run)..." -ForegroundColor Cyan
Write-Host "Console window will remain visible for debug output.`n" -ForegroundColor Yellow

try {
    # Launch cargo run in current console window (keeps it visible)
    $cargoProcess = Start-Process -FilePath "cargo" `
                                  -ArgumentList "run" `
                                  -WorkingDirectory $scriptDir `
                                  -NoNewWindow `
                                  -PassThru

    Write-Host "  Cargo process started (PID: $($cargoProcess.Id))" -ForegroundColor Green

} catch {
    Write-Host "ERROR: Failed to start cargo run: $_" -ForegroundColor Red
    exit 1
}

##############################################################################
# SECTION 5: Wait for Initialization
##############################################################################

Write-Host "`nWaiting for Rust application to initialize shared memory..." -ForegroundColor Cyan
Write-Host "This takes 60 seconds to ensure proper startup.`n" -ForegroundColor Yellow

# Progress indication during wait
$waitSeconds = 60
1..$waitSeconds | ForEach-Object {
    Start-Sleep -Seconds 1
    if ($_ % 10 -eq 0) {
        Write-Host "  $_ / $waitSeconds seconds elapsed..." -ForegroundColor DarkGray
    }
}

Write-Host "`n  Wait period complete" -ForegroundColor Green

##############################################################################
# SECTION 6: Launch AutoHotkey Script
##############################################################################

Write-Host "`nLaunching AutoHotkey script (hidden)..." -ForegroundColor Cyan

try {
    # Launch AHK script hidden in background
    $ahkProcess = Start-Process -FilePath $ahkExePath `
                                -ArgumentList "`"$ahkScriptPath`"" `
                                -WindowStyle Hidden `
                                -WorkingDirectory $scriptDir `
                                -PassThru

    Write-Host "  AutoHotkey script started (PID: $($ahkProcess.Id))" -ForegroundColor Green

} catch {
    Write-Host "ERROR: Failed to start AutoHotkey script: $_" -ForegroundColor Red
    # Attempt cleanup
    if ($cargoProcess -and -not $cargoProcess.HasExited) {
        Write-Host "Stopping cargo process..." -ForegroundColor Yellow
        Stop-Process -Id $cargoProcess.Id -Force -ErrorAction SilentlyContinue
    }
    exit 1
}

##############################################################################
# SECTION 7: Completion and Instructions
##############################################################################

Write-Host "`n========================================" -ForegroundColor Green
Write-Host "Lavender Moogle System Started!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host "Cargo Process ID:      $($cargoProcess.Id)" -ForegroundColor Cyan
Write-Host "AutoHotkey Process ID: $($ahkProcess.Id)" -ForegroundColor Cyan
Write-Host "`nTo stop the system:" -ForegroundColor Yellow
Write-Host "  1. Press INSERT key to stop AHK script" -ForegroundColor White
Write-Host "  2. Press Ctrl+C in this window to stop Rust app" -ForegroundColor White
Write-Host "`nThe console will now show Rust debug output..." -ForegroundColor DarkGray
Write-Host "========================================`n" -ForegroundColor Green

# Keep script running so console stays open
Wait-Process -Id $cargoProcess.Id
