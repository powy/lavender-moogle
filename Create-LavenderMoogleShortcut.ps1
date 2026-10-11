[CmdletBinding()]
param(
    [string]$DestinationDirectory = [Environment]::GetFolderPath('Desktop')
)

$ErrorActionPreference = 'Stop'
$launcher = Join-Path $PSScriptRoot 'lavender_moogle\start-LavenderMoogle.bat'
$icon = Join-Path $PSScriptRoot 'assets\branding\lavender-moogle.ico'
foreach ($file in @($launcher, $icon)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
        throw "Required file not found: $file"
    }
}
if (-not (Test-Path -LiteralPath $DestinationDirectory -PathType Container)) {
    throw "Shortcut destination does not exist: $DestinationDirectory"
}
$shortcutPath = Join-Path $DestinationDirectory 'Lavender Moogle.lnk'
if (Test-Path -LiteralPath $shortcutPath) {
    throw "A shortcut already exists: $shortcutPath"
}
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $launcher
$shortcut.WorkingDirectory = Split-Path -Parent $launcher
$shortcut.IconLocation = "$icon,0"
$shortcut.Description = 'Launch Lavender Moogle'
$shortcut.Save()
Write-Host "Created shortcut: $shortcutPath"
