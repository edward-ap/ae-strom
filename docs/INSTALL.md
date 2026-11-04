# Install this full build and select settings

The root `ae-strom.exe` is the full application. No launcher, external UI folder or Node runtime is needed. Windows WebView2 Runtime remains required. Normal startup is live-capable but stopped, with no autoplay.

## Safest transition: a fresh folder

1. Close the previous player yourself. Keep its EXE, configs and backups unchanged.
2. Create a new writable folder and copy the new `ae-strom.exe` into it.
3. Choose one settings source deliberately: `<old-folder>/config.json` is the adjacent/Slint-style config; `<old-folder>/local-data/player/config.json` is the previous Dense/Tauri profile. These can contain different selection, volume, geometry, stations or EQ. Modification time alone is not used to select a winner.
4. Copy only the chosen source into the new folder as `config.json`. Do not copy both candidates, and do not copy WebView cache data. The original source remains unchanged.
5. Double-click the new EXE. Compatible legacy normalization is backed up before rewriting the copied destination. Inspect banks, station assignments, EQ, volume/mute and position before starting playback. Close and reopen to confirm persistence.

The existing installation is still recoverable because none of its files were replaced. This procedure chooses no profile automatically on the user's behalf.

## Updating the same folder instead

Preserve the previous EXE and both configs first. Replace the EXE only while its player is closed. If both recognized configs differ without a matching migration receipt, the new app shows CONFIG-CONFLICT and preserves both files.

- To retain the previous Dense/Tauri profile, preserve/rename the adjacent `config.json` so it is no longer a candidate. Default startup migrates the retained `local-data/player/config.json` into a new adjacent config; the previous source stays unchanged.
- To retain the adjacent config, preserve the previous Dense/Tauri config under an unrecognized backup filename, or use an explicit copied profile with `--data-dir`. Keep an additional original copy of the adjacent legacy file before normalization.
- `--data-dir <absolute-directory>` disables discovery of the other candidate only for launches using that option. It is not a permanent one-shot preference; an ordinary double-click will still detect any remaining unacknowledged conflict.

Moving/renaming a candidate must be the user's deliberate choice. No union of two station collections is performed. Complete selection, backup and receipt rules are in [CONFIG_STORAGE.md](CONFIG_STORAGE.md).

## Data and rollback

Normal config: `config.json` beside the EXE. Windows browser storage: `%LOCALAPPDATA%/local.edwardap.ae-strom/webview/profile-<id>/EBWebView`. AppData also keeps profile locks and migration receipts. This does not make settings depend on browser cache contents.

The delivery preserved the previous project EXE under ignored `local-data/config-storage-delivery-rollback-*`. Close the new player before copying the saved EXE back. The previous `local-data/player` was not modified by delivery. If you have used the new player, keep its adjacent config too: an older binary's default profile may differ, so restoring an EXE alone does not merge newer settings back into the old profile.

No real installation was updated by the agent. No live radio check was run for this delivery. The visible conflict warning and clean/migration cases are user-reported passes; the complete native fake script, native offscreen/mixed-DPI matrix, other physical error dialogs and macOS remain unverified. Script Execution Policy was not changed or bypassed after the earlier denial.
