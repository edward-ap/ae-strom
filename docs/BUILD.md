# Build and verification

Use Node 22+, Rust MSVC, the Windows SDK/C++ tools and installed WebView2. Lockfiles are required. Run `npm ci --ignore-scripts`, then `npm run vendor`, then `npm test`. esbuild's standard platform package is installed as a development dependency; no framework lifecycle script is run.

Vendor regeneration validates the exact installed taet-ui npm integrity and 304 published source files, requires esbuild 0.28.2 and builds one foundation+components ESM profile. It verifies all six canonical output hashes before replacing assets. The matching 85 declaration files and full profile type facade are also checked. Maps stay in ignored `.gen/`; the runtime delivery is the six assets listed in `web/vendor/manifest.json`. Do not combine another profile or old framework instance.

`cargo fmt --check`, `cargo test --release --locked --features live-audio`, `cargo clippy --release --locked --all-targets --features live-audio` and `cargo build --release --locked --features live-audio --bins` verify/build Rust. Add `--offline` when registry sources are cached. Unit tests use fake playback or a loopback PCM sink; the live-check binary is a separate opt-in tool.

The GUI EXE is `target/release/ae-strom.exe`; it defaults to fake/stopped. `scripts/native-smoke.ps1` creates a fresh ignored synthetic profile, starts only this EXE with `--fake --smoke`, checks real Tauri IPC/WebView geometry and identity, saves evidence and exits. It never stops another player. A timeout is an uncertain launch: inspect its PID/evidence before retrying.

The separate `ae-strom-live-check` binary requires `--confirm-live --url <chosen URL> --data-dir <new absolute directory>`. It uses app volume 10%, two bounded Play/Stop phases and orderly shutdown. Use it only with explicit live-test authorization and no competing player. No live URL is baked into the project.

Default data root is AeStrom, with separate preview/player subdirectories; explicit --data-dir overrides it. Config/webview directories, logs, npm caches, target, generated files and test evidence remain ignored. No original profile is copied during build or tests.

`scripts/startup-smoke.ps1` runs four sequential native fake launches using only newly generated synthetic profiles, checks hidden-before-show geometry and writes ignored evidence. Additional available monitors are tested when present; it never reads an existing user profile.

`scripts/startup-smoke.ps1` runs four sequential native fake launches using only newly generated synthetic profiles, checks hidden-before-show geometry and writes ignored evidence. Additional available monitors are tested when present; it never reads an existing user profile.

## Configured desktop launcher

`cargo build --release --locked --features live-audio --bins --examples` produces the main GUI, ae-strom-player launcher and offline profile-copy verifier example. Copy the launcher to the project root as `ae-strom-player.exe`. It starts the main GUI with --enable-live-audio and the absolute local-data/player profile, Stopped. The launcher also works alongside the GUI in a portable bundle, using local-data/player beside them. Unit tests cover root/release/portable layouts and missing main EXE.

`scripts/launcher-smoke.ps1` compiles a tiny local synthetic child with the installed Rust compiler and verifies real Windows launcher argument forwarding through a path containing spaces. It does not open the configured player profile or any audio device. The offline `profile-verify` example must receive only a fresh audit COPY: its loader can normalize that copy. It checks exact JSON save/load and never requests station URLs.
