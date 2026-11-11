# macOS development port

This is a local Apple Silicon port based on `v0.1.3` / source
`1a3ebb07bb2dfd27eacb4c5bfbaf04ab1d57becb`, retaining pinned Tæt UI
`0.6.0-rc.7`. It has not been published or notarized.

## Build and run

Prerequisites: Apple Silicon Mac, Xcode command-line tools, Rust/Cargo,
Node 22+ and npm. Verification used macOS 26.5.1, Rust 1.99.1 and Node 22.23.4.
The bundle declares macOS 11.0 minimum, but older macOS and Intel hardware have
not been tested.

```sh
npm ci --include=dev --ignore-scripts
npm run build:mac
```

Output: `target/release/bundle/macos/Æ Strøm.app`.
The bundled executable is `Contents/MacOS/ae-strom`; no Node server or separate
web folder is required at runtime. This local command deliberately skips
Developer ID signing and notarization. No security settings are modified.
For distribution, prepare signing/notarization and a macOS-specific third-party
license inventory before publishing; do not reuse Windows build certification.

Open the app normally. It starts stopped with an empty profile on first use.
Real audio begins only after Play. Configuration is not imported automatically.

## Profiles and native integration

- Default live profile: `~/Library/Application Support/local.edwardap.ae-strom/player/config.json`.
- Default fake profile: `~/Library/Application Support/local.edwardap.ae-strom/preview/config.json`.
- `--data-dir <absolute-directory>` retains explicit isolated profile behavior.
- The app never writes config into its `.app` bundle. Replacing the bundle does
  not replace settings. Explicit imports retain validation/conflict safeguards.
- Profile locks remain in application data. WKWebView uses a nonpersistent store
  because all persistent player state belongs to Rust, not browser storage.
- Cmd+C or Ctrl+C outside editors copies the full title through NSPasteboard.
  Native text editing, composition and modifier guards remain separate.
- Startup failures use a sanitized native NSAlert, not an invisible stderr-only
  message when launched from Finder.
- App-menu Quit/Cmd+Q uses the same awaited save path as the player close button.
  A save failure keeps that route open for correction. Dock Quit and OS session
  termination retain last-chance shutdown, but cannot cancel termination on a
  save error in the pinned Tao runtime. Use the player close button or Cmd+Q
  when resolving a storage error; forced termination can lose pending writes.
- Startup uses the monitor work area, excluding the menu bar and Dock. A single
  synchronous AppKit frame update avoids Tao's deferred resize/position race.
- The ICY console support module is under `src/diagnostics/`: the pinned Tauri
  CLI incorrectly treated `src/bin/support/` as an executable during bundling.
  Its bytes/behavior are otherwise unchanged, and Cargo binary targets are explicit.

## Verification, 2026-11-11

Executed on an Apple Silicon Mac, with isolated synthetic profiles and
loopback PCM test sinks. No radio stream, personal station file or audio output
device was used.

- `cargo fmt --check`, `git diff --check`: passed.
- `cargo test --release --locked --all-targets`: 115 passed (100 library, two
  inspector unit, one icon, ten config, two inspector integration tests).
- `cargo test --release --locked --doc`: seven passed.
- `cargo clippy --release --locked --all-targets`: passed with five inherited
  warnings in audio/config/metadata code, none in the new Mac module.
- `npm test`: syntax, types, vendor integrity, icons and all 120 Chromium tests
  passed (60 at 1x, 60 at 1.5x).
- WebKit 1x and 2x: 59 passed per project. One Chromium-only CDP glyph inspection
  is explicitly skipped per WebKit project; other font/geometry checks run.
- Native WKWebView Retina 2x: five scenarios with 40 UI checks each, plus host
  assertions for initial geometry before show, strip/edge anchoring, work-area
  bounds, actual resized client dimensions and profile isolation. Scenarios:
  fresh, restored, offscreen, clamped, and bottom-edge upward panels.
- The final `.app` was also launched through macOS LaunchServices (`open -n -W`)
  with an isolated fake profile: all 40 native UI checks passed.
- Mach-O architecture is arm64; linked libraries are system frameworks/libraries;
  Info.plist passed `plutil -lint`; no config.json is included in the bundle.

Final tested app executable SHA256:
`a3a5a7ab5b928dc81e1841c44686a267f73119878801304b93f4f112da5bf616`.

Local evidence is ignored by Git: `evidence/browser-report`,
`evidence/webkit-report`, `evidence/webkit-retina-report`,
`local-data/native-matrix-1794405097970`, and
`local-data/mac-launchservices-1794405146985`.

Reproduce native fake checks:

```sh
npm run test:native -- "target/release/bundle/macos/Æ Strøm.app/Contents/MacOS/ae-strom"
STROM_WEBKIT=1 npx playwright test --project=webkit-100 --project=webkit-200
```

Install matching Playwright browsers with `npx playwright install chromium webkit`
if they are not already cached. The native harness always creates fresh profiles
and passes `--fake`; it never invokes real audio.

## Remaining manual acceptance

Programmatic browser/native IPC results are not physical OS-input or listening
tests. The authorized playback check below covers real CoreAudio Play/Stop. Still verify output-device changes, native pointer drag,
clipboard/menu key delivery, native select popup navigation, IME, visible startup
error dialogs and Cmd+Q save-error behavior. The headless Mac select regression
uses native type-ahead because Home/End does not drive that OS popup. A second
monitor/mixed-DPI migration, Intel build, older macOS and compositor first-frame
capture are not certified. Windows should be rechecked before merging; its native
implementation and vendor runtime were not changed by this port.


## Authorized live playback check, 2026-11-11

After explicit user authorization, the installed app played the repository's
public ROCK ANTENNE Deutschrock AAC stream using a fresh isolated test profile.
The test began at app volume 10%; the UI subsequently changed externally to 33%
and that value was preserved. System volume and the normal player profile were
not changed.

- Read-only CoreAudio process properties confirmed this app was the only active
  output process. Output was MacBook Pro Speakers, stereo 48 kHz. No microphone
  or system-audio recording was performed.
- Native UI Play started the stream and displayed a changing five-band spectrum.
  ICY supplied `Onkel Tom - Bon Scott Hab Ich Noch Live Gesehen`. After Stop and
  another Play, a second connection successfully displayed `Rogers - Steh Auf`.
- Mute/Unmute UI states and compact/expand were exercised. Compact/expand kept
  the original connection and CoreAudio output active.
- Stop returned the idle brand and CoreAudio reported no running output for the
  app. Restart returned running output; final Stop cleared it again. The test
  window was left open and stopped.
- No WARN or ERROR runtime records occurred in the observed sessions.

Evidence: ignored `local-data/mac-live-1794405448/summary.json`, `runtime.log`,
and per-stage CoreAudio status records. This verifies actual network/device
playback, not acoustic quality: no listening, speaker recording, Bluetooth/output
switch, long-run stability, or within-one-connection track-change timing is
certified. Native clipboard, EQ sound quality and IME remain separate checks.


## Sanitized 0.1.4 candidate, 2026-11-11

The candidate uses pinned Rust 1.96.0, matching Windows CI, and the same locked
crate graph. It adds idempotent close handling with retry after a failed save,
cancellation of queued native geometry work after a timeout, and collision-free
parallel launch-test fixtures. The installed earlier development app was not
replaced during candidate preparation.

- Rust all-targets: 118 passed, including three new host lifecycle tests.
- Rust documentation examples: seven passed; fmt and diff whitespace passed.
- Clippy passed with the same five inherited library warnings.
- Full npm checks passed, including eight publication-guard regressions and
  120 Chromium browser checks. WebKit 1x/2x: 118 passed, two expected CDP skips.
- Sanitized native bundle: five fake-profile scenarios, 40 UI checks each.
- ZIP extraction preserved the three app files byte-for-byte; the extracted app
  also passed all 40 fake UI checks when launched via LaunchServices.
- Static publication checks found zero private-path/credential patterns,
  arm64 Mach-O architecture, matching 0.1.4 identity, executable permissions,
  and only system dynamic libraries. The earlier development app is correctly
  rejected by the private-path guard.

Candidate executable SHA256:
`738b0432b3ad161d912a8d1af80cbfc080addddfda88ed04c789ffe525b926ce`.
Candidate ZIP SHA256:
`8b125d6c207e7f8875307f47ea940ea509c449053eb20b662ba8dbd538e19df6`.

Evidence remains ignored under `local-data`: `macos-*-0.1.4` logs/JSON,
`native-matrix-1794407416635`, and `mac-packaged-smoke-0.1.4-1794407503`.
The earlier authorized CoreAudio test above is evidence for the initial port;
this candidate's checks so far use fake playback and do not claim another
listening session. Windows validation and a separate signing/unsigned-preview
publication decision are still pending. See RELEASE_0.1.4.md.
