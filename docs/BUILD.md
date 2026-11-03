# Build and deployment

Development uses Node 22+, Rust MSVC and Windows C++ tools/SDK. The runnable distribution needs normal Windows system libraries and WebView2. No Node/npm process or separate application DLL bundle is required.

Use npm ci --ignore-scripts, npm run vendor and npm test. Vendor generation uses pinned taet-ui 0.6.0-rc.4 and esbuild 0.28.2; it verifies npm/source integrity, 6 canonical assets, 85 declaration files and the profile facade. Generated maps stay ignored; do not combine another BaseControl/profile instance or edit vendor bytes.

live-audio and custom-protocol are default Cargo features. Run cargo fmt --check, cargo test --release --locked --all-targets, cargo test --release --locked --doc and cargo clippy --release --locked --all-targets. Build the application with cargo build --release --locked --bin ae-strom. Add --offline when sources are cached. Test helpers/examples are optional development artifacts built with --bins --examples.

The full application is target/release/ae-strom.exe. Canonical local delivery is the same full file at the project root, ae-strom.exe. Preserve the old EXE before copying and do not overwrite an active process. No launcher is needed. The former launcher source is removed and its binaries retained only in ignored rollback.

Normal startup defaults live/stopped and portable local-data/player beside the EXE. Cargo's target/release build shares the project-root profile only when Cargo.toml and src/main.rs exist at that root. Ordinary portable folders need no source markers. Explicit --fake uses local-data/preview, and --data-dir overrides the profile with an absolute path. No AppData/preview fallback is used for a normal launch. If the folder cannot be written, startup reports an error rather than substituting another profile.

To retain stations while moving the application, move local-data/player/config.json with the EXE, or use an explicit data directory. WebView cache/profile data is generated in the same profile; private configs/logs/caches and all EXEs stay ignored. No station/profile data is embedded during compilation.

scripts/native-smoke.ps1 runs the genuine Tauri/WebView UI with explicit --fake in a fresh synthetic directory. scripts/single-exe-smoke.ps1 deploys only the full EXE into an isolated folder containing spaces, starts it from an unrelated cwd, and verifies default live/stopped mode, loaded assets, station/EQ schema and settings save/reload across two runs. No audio is played. The internal --startup-smoke path is restricted to marked synthetic fixtures and must not be pointed at personal profiles. Timeouts are uncertain own launches: inspect their PID/evidence before retrying.

The separate ae-strom-live-check development tool requires --confirm-live --url <approved URL> --data-dir <fresh absolute directory>. It is not needed to run the app. Use only an approved stream with no competing player; it performs two bounded phases at app volume 10%, Stop/shutdown and worker checks. The profile-verify example operates only on a fresh audit COPY because loading can normalize that copy. It checks exact JSON roundtrip offline and does not fetch station URLs.
