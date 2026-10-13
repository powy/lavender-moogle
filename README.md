<p align="center">
  <img src="assets/branding/lavender-moogle.png" width="192" alt="Lavender Moogle logo">
</p>

# Lavender Moogle

A personal helper that performs the next logical action of a rotation when a key (or a controller button) is pressed. It was built for FFXIV, with a first attempt on WoW.

## Purpose

The goal is to remove tedious button pressing in repetitive content (farming, reclears) in MMOs, and to make that content playable with a controller and only a few buttons when I can't use the computer (typically, a baby sleeping on me). Technically, it does three things: it gets game information out of a WoW addon, relays FFXIV events from ACT, and hands the next action to AutoHotkey through a Windows shared memory.

The rotations are written by me, are very incomplete, and are tuned to my own playstyle. **If you are looking for a full auto-battler, this is not the right project:** complete ones already exist.

### Fair use

Automating a rotation sits in a gray area of the games' rules. My own rules, which I ask you to follow too:

- personal use only, with your own code;
- never where it affects other players: no FFXIV parsing or ranking, no PvP, no WoW competitive modes.

### Status

- **FFXIV** (this repository): works for the supported jobs, but is likely to become obsolete with the Evercold expansion and its new job styles.
- **WoW**: set aside. The addon route (reading game information from an addon) has apparently been impossible for several expansions, and Blizzard has since added a native auto-rotation key, which makes the project much less useful. The old addon and screener are no longer in this repository.

## How it works

```
FFXIV / ACT + Triggernometry ──HTTP POST (JSON)──▶ Rust engine ──shared memory──▶ AutoHotkey ──▶ key press
```

1. **Events in.** ACT (Advanced Combat Tracker) reads the FFXIV logs, and its Triggernometry plugin (triggers in `lavender_moogle/lavender_triggers.xml`) sends spell casts, buffs, job gauge, death and reset events as JSON requests to the Lavender Moogle server at `http://127.0.0.1:50017`. Each message carries a `spell_name` and a `job`.
2. **Decision.** The Rust engine keeps the cooldowns, statuses, charges and job gauge of every job. Every 50 ms it runs the rotation of the current job and picks the next action for each of the four rotation variants.
3. **Shared memory.** For each variant, the chosen key binding (modifier + key) and the movement state are written to the `Global\lavender_keys` mapping (16 bytes).
4. **Key press.** `lavender_moogle.ahk` reads the mapping and sends the key when you hold a trigger.

### Triggers

Hold `F13`, `F14`, `F15` or `F16` to run rotation variant 1, 2, 3 or 4. Controller buttons trigger actions too (see `lavender_moogle.ahk`).

Press `Insert` to stop the AutoHotkey script.

### Supported jobs

Bard, Dancer, Dark Knight, Gunbreaker, Monk, Paladin, Scholar and Summoner. Each job has its own `rotation_<job>.rs` in `lavender_moogle/src/`, and `rotation_template.rs` is the starting point for a new one.

### Spell data

`lavender_moogle/moogle_spell_list.xlsx` has one sheet per job. Each row describes a spell or status (key binding, cooldown, charges, cast time, statuses given or removed). The engine loads it at startup from the `lavender_moogle` folder.

## Launch on Windows

Install Rust and AutoHotkey v2, then run `lavender_moogle/start-LavenderMoogle.bat`. The launcher requests administrator privileges and starts Rust and AutoHotkey.

To create a desktop shortcut with the project icon, run this command from the repository root:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\Create-LavenderMoogleShortcut.ps1
```

The shortcut points to this checkout. Recreate it if you move the folder.

## Requirements

- Windows
- [Rust](https://rustup.rs/) toolchain
- [AutoHotkey v2](https://www.autohotkey.com/)
- ACT with the Triggernometry plugin, loaded with `lavender_moogle/lavender_triggers.xml`

## Repository layout

| Path | Content |
|---|---|
| `lavender_moogle/src/` | Rust engine: server, spell state, shared memory, one rotation per job |
| `lavender_moogle/lavender_moogle.ahk` | Reads the shared memory and sends the key presses |
| `lavender_moogle/Start-LavenderMoogle.ps1`, `start-LavenderMoogle.bat` | Launcher for both processes |
| `XInput.ahk` | Controller support library used by the AutoHotkey script |
| `assets/branding/` | Logo, icon and social preview |

## Branding

- `assets/branding/lavender-moogle.png`: original logo, displayed above.
- `assets/branding/lavender-moogle.ico`: Windows icon with sizes from 16 to 256 pixels.

`assets/branding/social-preview.png`: 1280 × 640 banner for the GitHub social preview.

GitHub social preview is configured separately under **Settings > Social preview > Edit**. Use a PNG, JPG or GIF smaller than 1 MB; GitHub recommends 1280 × 640 pixels. The original logo PNG is larger than 1 MB, so use the banner.

## License

Lavender Moogle is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE). Personal use and private experimentation are permitted under its terms; commercial use is not permitted except as allowed by the license's defined permitted purposes.

When redistributing the software, include the license text or its URL and preserve the attribution in [NOTICE](NOTICE). Third-party components remain subject to their own licenses.

Sharing the source of your modifications is appreciated, but is not required by this license.
