<p align="center">
  <img src="assets/branding/lavender-moogle.png" width="192" alt="Lavender Moogle logo">
</p>

# Lavender Moogle

A Windows assistant for FFXIV rotations. Rust selects actions from spell events and job state; AutoHotkey sends the configured key presses through a shared-memory interface.

## Launch on Windows

Install Rust and AutoHotkey v2, then run `lavender_moogle/start-LavenderMoogle.bat`. The launcher requests administrator privileges and starts Rust and AutoHotkey.

To create a desktop shortcut with the project icon, run this command from the repository root:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\Create-LavenderMoogleShortcut.ps1
```

The shortcut points to this checkout. Recreate it if you move the folder.

## Branding

- `assets/branding/lavender-moogle.png`: original logo, displayed above.
- `assets/branding/lavender-moogle.ico`: Windows icon with sizes from 16 to 256 pixels.

GitHub social preview is configured separately under **Settings > Social preview > Edit**. Use a PNG, JPG or GIF smaller than 1 MB; GitHub recommends 1280 × 640 pixels. The original PNG is larger than 1 MB.
