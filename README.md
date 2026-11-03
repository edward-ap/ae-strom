# Æ Strøm

Compact native Rust radio with Tæt UI and a Tauri WebView. Technical project/crate: ae-strom / ae_strom, version 0.1.0.

## Run

Double-click **ae-strom.exe** in the project root. It is the complete application, with native audio, UI, fonts and assets embedded. No launcher, flags, Node server or separate web folder is needed. Normal mode is **live**, startup is **Stopped**, and playback begins only with Play.

This is a portable layout. Settings/stations remain editable in **local-data/player/config.json beside the EXE**; WebView data is generated there in its own subfolder. The explicitly transferred profile is retained without alteration. Personal settings are not compiled into the EXE or committed. Move the EXE and local-data/player together to retain your stations; moving only the EXE creates a separate fresh portable profile. The folder must be writable. There is no silent fallback to AppData or preview data.

The development target/release EXE uses the project-root profile when the normal Cargo source markers exist. A distribution in an ordinary folder uses the folder beside its EXE. --data-dir <absolute-directory> remains an explicit override. Fake playback is available only with **--fake**, using separate local-data/preview by default. The normal Windows WebView2 runtime is required; no application DLL bundle, React or JS runtime dependency is shipped.

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

## Interface and framework

- A–Z banks with ten slots; active playback and Settings browsing are separate.
- Play/Stop, volume/mute, scrolling title and full-title Ctrl+C outside editors.
- Ten-band EQ/preamp, custom presets and per-station assignments.
- Frameless always-on-top window with noninteractive chrome drag and compact mode preserving state/drafts.
- Physical A–Z keys, 1–9/0 slots, +/- volume and * mute, with editor/composition guards.

Normal 725×36, compact 102×36, Settings 725×258, EQ 725×257 logical px. Noto Sans 12 px is bundled. Specialized SpectrumAnalyzer and ChannelIndicator remain app-owned on one BaseControl identity.

Tæt UI is pinned to published **taet-ui@0.6.0-rc.4**, components ESM, source bfd3ddcddf1cdc539397e5d2ac5ed3cb976e72c2. Runtime assets, public declarations, source hashes and npm integrity are checked. Technical dui CSS/DOM names are retained. See [BUILD.md](docs/BUILD.md), [ARCHITECTURE.md](docs/ARCHITECTURE.md), [CONSUMER_NOTES.md](docs/CONSUMER_NOTES.md) and [VERIFICATION.md](docs/VERIFICATION.md).
