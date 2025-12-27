@echo off
REM ##########################################################################
REM Lavender Moogle Launcher - BAT wrapper for PowerShell script
REM
REM Purpose: Simple double-click launcher for the Lavender Moogle system
REM          This BAT file calls PowerShell to run the main PS1 script,
REM          which handles auto-elevation and launches cargo + AHK
REM
REM Portability: Uses relative paths - works on any computer
REM ##########################################################################

REM Get the directory where this BAT file is located and change to it
cd /d "%~dp0"

REM Launch PowerShell with the script (PS1 handles auto-elevation)
powershell.exe -ExecutionPolicy Bypass -File "%~dp0Start-LavenderMoogle.ps1"

REM Pause if there was an error (so user can see error messages)
if errorlevel 1 (
    echo.
    echo An error occurred. Press any key to exit...
    pause >nul
)
