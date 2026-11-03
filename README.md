# Æ Strøm

A compact radio player: native Rust audio, Tæt UI controls and a Tauri WebView.
Technical project/crate names: `ae-strom` / `ae_strom`. Version 0.1.0 is a local test candidate.

The default launch is a silent synthetic preview. Real playback requires an explicit `--enable-live-audio` launch and a selected station. No previous player/profile is discovered or imported automatically.

## Build and run

Windows prerequisites: Rust MSVC and Windows C++ tools/SDK, Node 22+ for development, WebView2 for the desktop app. The packaged EXE uses no Node/npm server, React, web audio or remote UI assets.

```powershell
npm ci --ignore-scripts
npm run vendor
npm test
cargo test --release --locked --features live-audio
cargo clippy --release --locked --all-targets --features live-audio
cargo build --release --locked --features live-audio --bins
./target/release/ae-strom.exe --fake
```

Browser preview: `npm run preview`. Native fake acceptance: `./scripts/native-smoke.ps1` after building.

## Profiles and listening

New default roots are `%LOCALAPPDATA%/AeStrom/preview` and `%LOCALAPPDATA%/AeStrom/player` on Windows; the latter is used only with `--enable-live-audio`. Each contains its own config and WebView data. `--data-dir <absolute-directory>` overrides the root. The old player's data and app identity are not reused.

After closing any competing radio session, an intentional listening launch is:

```powershell
$profileDirectory = Join-Path $PWD 'local-data/listening'
./target/release/ae-strom.exe --enable-live-audio --data-dir $profileDirectory
```

It starts Stopped. Configure a chosen station in Settings, set app volume to 10% for initial listening, then use Play. This project includes no personal station list or mandatory live station default. Copying/importing an existing personal profile requires a separate explicit decision. An explicit `--import-config <selected-copy.json>` path exists, but is never invoked automatically.

## Interface

- A–Z banks, ten slots each; Settings browsing and active playback remain separate.
- Play/Stop, volume/mute, scrolling track title and full-title Ctrl+C outside editors.
- Ten-band EQ and preamp, built-in/custom presets and per-station assignments.
- Frameless always-on-top window; drag through noninteractive top chrome.
- Compact mode preserves the same session, values and drafts, showing expand, Play/Stop and spectrum.
- Physical A–Z keys; 1–9/0 slots;+/− volume;* mute. No global shortcut registration.

Geometry is preserved: normal 725×36, Settings 725×258, EQ 725×257 and compact 102×36. The channel frame is 40×26 and EQ Preset fields 110 px wide.

## Framework and source

Tæt UI is pinned exactly to `0.6.0-rc.4`, components ESM profile, release source `bfd3ddcddf1cdc539397e5d2ac5ed3cb976e72c2`. npm integrity, published source-file hashes, matching JS/CSS/fonts/licenses and public declarations are checked. `taet-ui` and esbuild are development dependencies; UI runtime dependencies are zero. Reproduction and limits: [BUILD.md](docs/BUILD.md), [ARCHITECTURE.md](docs/ARCHITECTURE.md), [CONSUMER_NOTES.md](docs/CONSUMER_NOTES.md), [VERIFICATION.md](docs/VERIFICATION.md).

This repository is local; no installer, GitHub repository or publication is implied. Native physical input, IME, accessibility, long listening soak and non-Windows acceptance remain separate gates.
