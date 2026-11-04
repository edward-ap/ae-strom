# Æ Strøm

Compact native Rust radio with Tæt UI and a Tauri WebView. Technical project/crate: ae-strom / ae_strom, version 0.1.0.

## Run

Double-click **ae-strom.exe** in the project root. It is the complete application, with native audio, UI, fonts and assets embedded. No launcher, flags, Node server or separate web folder is needed. Normal mode is **live**, startup is **Stopped**, and playback begins only with Play.

Settings and stations live in **config.json next to the EXE**. The directory must be writable. Move the EXE and this file together to retain settings. Personal station data is never embedded in the binary or Git.

WebView2 browser data is generated separately under **%LOCALAPPDATA%/local.edwardap.ae-strom/webview/profile-<id>/** on Windows. Each absolute profile location gets its own browser directory and lock. Browser caches are distinct from the Rust station config. The Windows WebView2 runtime remains required; Node, React and separate UI files are not required at runtime.

The previous **local-data/player/config.json** is supported without modifying it. If the adjacent config is absent, startup validates and migrates that previous profile, keeps a verified `.bak`, and records the migration in AppData. On subsequent launches the adjacent config wins while the acknowledged previous file remains unchanged. If both files already differ without a matching migration record, or the previous file later changes, startup reports a conflict and preserves both files. See [configuration migration and recovery](docs/CONFIG_STORAGE.md).

The development target/release EXE uses the project-root config when normal Cargo source markers exist. A plain distribution uses its own EXE directory, independently of the working directory. **--data-dir <absolute-directory>** explicitly selects that directory's config and disables previous-profile discovery. Fake playback remains opt-in with **--fake**, using separate local-data/preview by default; its WebView cache is still in per-profile AppData. **--import-config <source-file>** validates into a fresh destination and never overwrites a different existing config. There is no fallback to an empty or alternate profile after a parse, schema, conflict or write error.

## Build and test

Use Rust MSVC/Windows SDK and Node 22+ for development. Lockfiles are required.

```powershell
npm ci --ignore-scripts
npm run vendor
npm test
cargo fmt --check
cargo test --release --locked --all-targets
cargo test --release --locked --doc
cargo clippy --release --locked --all-targets
cargo build --release --locked --bin ae-strom
```

live-audio and custom-protocol are default features. The build output is target/release/ae-strom.exe. Copy the full EXE to the project root after preserving the previous version and checking it is not active. Test tools/examples can be built with --bins --examples; they are not application dependencies.

Browser preview is `npm run preview` and stays synthetic. `scripts/native-smoke.ps1` runs explicit --fake checks in a fresh profile. `scripts/single-exe-smoke.ps1` copies only the full EXE into an isolated portable folder and verifies default live/stopped mode plus save/restart without playing audio. Its internal startup-smoke flag accepts only a marked synthetic fixture profile.

## Ultra-compact mode

Click the **double-chevron button to the left of Play/Stop** to collapse the same player window to **102 × 36 logical pixels**. Only the expand chevron, Play/Stop button and five-band spectrum remain visible. Physical pixel dimensions depend on Windows display scaling.

Playback continues in the same Rust session. The current station, volume/mute, EQ settings and unsaved editor drafts are retained; collapsing does not stop or reconnect the stream. Click the chevron again to restore the full player and its previous panel. Settings, EQ and the remaining controls are available after expansion. The compact background/spectrum can still be used to drag the window.

## Screenshots

These reference images use synthetic preview data, with no personal station list. Replace the files under **docs/images** to update the README images; keep the same names so the relative links continue to work.

Normal player:

![Æ Strøm normal player](docs/images/player.png)

Ultra-compact mode (102 × 36 logical pixels):

![Æ Strøm ultra-compact mode](docs/images/compact.png)

Station settings:

![Æ Strøm station settings](docs/images/settings.png)

Equalizer:

![Æ Strøm equalizer](docs/images/equalizer.png)

## Interface and framework

When stopped without an error, the ticker shows **-= Æ Strøm =-** centered and still. Connection/reconnection messages, errors and playing track titles retain the normal ticker behavior.

- A–Z banks with ten slots; active playback and Settings browsing are separate.
- Play/Stop, volume/mute, scrolling title and full-title Ctrl+C outside editors.
- Ten-band EQ/preamp, custom presets and per-station assignments.
- Frameless always-on-top window with noninteractive chrome drag and compact mode preserving state/drafts.
- Physical A–Z keys, 1–9/0 slots, +/- volume and * mute, with editor/composition guards.

Normal 725×36, compact 102×36, Settings 725×258, EQ 725×257 logical px. Noto Sans 12 px is bundled. Specialized SpectrumAnalyzer and ChannelIndicator remain app-owned on one BaseControl identity.

Tæt UI is pinned to published **taet-ui@0.6.0-rc.4**, components ESM, source bfd3ddcddf1cdc539397e5d2ac5ed3cb976e72c2. Runtime assets, public declarations, source hashes and npm integrity are checked. Technical dui CSS/DOM names are retained. See [BUILD.md](docs/BUILD.md), [ARCHITECTURE.md](docs/ARCHITECTURE.md), [CONSUMER_NOTES.md](docs/CONSUMER_NOTES.md) and [VERIFICATION.md](docs/VERIFICATION.md).

Typography: bundled standard Noto Sans variable at 12 px. [Font audit and native/macOS limits](docs/FONT_AUDIT.md) records actual loading/rendering evidence and intentional OS fallback.
