# Build and deployment

## Prerequisites and build inputs

The verified platform is Windows x64. Use a Rust MSVC toolchain, Microsoft C++ Build Tools (Desktop development with C++) and a Windows SDK. Rust/Cargo 1.96.0, Node 22.14.0 and npm 11.8.0 were checked. Node 22+ is required only for vendoring, optional icon generation, preview and JS/type/browser development checks. A compatible WebView2 Runtime is needed for running the native app. Follow [Tauri's Windows prerequisites](https://v2.tauri.app/start/prerequisites/); no system tools are installed by this project's build.

Committed inputs include Cargo.toml/Cargo.lock, package.json/package-lock.json, the application `web` files, pinned `web/vendor` assets/types/licenses, provenance manifests, Tauri config and icon. Personal configs, EXEs, node_modules, target, generated schemas and caches are ignored and absent from a clean clone.

## Cargo-only production build

From the repository root:

```text
cargo build --release --locked --bin ae-strom
```

All UI files are already committed. There is no pre-build npm step or required frontend build server; Cargo embeds `web` into the release EXE. `live-audio` and `custom-protocol` are defaults. Use `--offline` only after the locked registry sources have been downloaded. The normal Cargo command does not require Node or node_modules.

The output is `target/release/ae-strom.exe`, one complete application. Preserve an old EXE and close its player before copying the new file to the project root or another portable folder. Do not overwrite an active process. Runtime needs normal Windows system libraries and WebView2; no Node process, separate UI folder or launcher is shipped.

## Publication candidate with remapped build paths

A normal Cargo build can retain absolute source locations from the builder's Cargo registry and Rust toolchain. For a publication candidate, use Node 22+ and a clean environment without custom RUSTFLAGS, CARGO_ENCODED_RUSTFLAGS or RUSTDOCFLAGS:

```text
node scripts/release-cargo.js build --release --locked --bin ae-strom
node scripts/release-cargo.js test --release --locked --all-targets
node scripts/release-cargo.js test --release --locked --doc
node scripts/check-release.js local-data/publication-target/release/ae-strom.exe
```

The wrapper supplies Rust's supported `--remap-path-prefix` for both Windows separator forms, mapping the user home, checkout, Cargo home and compiler sysroot to generic paths. On Windows it also sets MSVC's PDB alternate path to a filename. It changes no application features, config paths or audio settings. The separate output is `local-data/publication-target/release/ae-strom.exe`; existing root/target delivery files are untouched. An explicit CARGO_TARGET_DIR can select another isolated output directory.

Path remapping is best effort, not an assertion of anonymity or byte-identical builds. Scan the final EXE (including ASCII and UTF-16 strings, PE debug paths and known private URLs), verify its resources and tests, and record its checksum before publication. Do not ship PDB/build directories, private profiles or logs. See [Rust path remapping](https://doc.rust-lang.org/rustc/remap-source-paths.html) for linker, environment-variable and generated-code limitations.

## Reproduce pinned Tæt vendor assets

The published package is **taet-ui@0.6.0-rc.4**. It has no JS runtime dependencies. In this application it is declared under devDependencies because its source is consumed by the vendoring step; the running UI imports the committed ESM file, not the npm package.

```text
npm ci --include=dev --ignore-scripts
npm run vendor
npm run test:vendor
git diff --exit-code -- web/vendor package.json package-lock.json provenance/taet-ui-rc4-source.json
```

`npm ci` installs the exact package/tool versions and verifies locked npm integrity without updating manifests. Keep optional dependencies enabled for the esbuild platform binary; the tested `--ignore-scripts` installation does not need a postinstall download fallback. A separate package installation is unnecessary. Lifecycle hooks remain disabled; explicitly requested npm run commands still run.

The generator verifies all 304 pinned published source files, builds the components/foundation ESM profile using esbuild 0.28.2, rejects external runtime imports, and verifies six runtime assets, 85 declarations and the public type facade. The CSS build copies Noto Sans and OFL licensing. Source maps remain under ignored `.gen`.

The manifest is a required committed input. If `web/vendor` or its manifest is missing/corrupt, recover it from the same Git revision, preserving any intentional local edits first; then run the commands above. The generator reproduces an already-reviewed pin and will not invent new canonical hashes.

## Framework version changes

There is no automatic framework-upgrade script. A package-version bump alone is rejected. A reviewed upgrade must coordinate:

- package.json and package-lock.json;
- the published-source package/version/commit/integrity/files in `provenance/taet-ui-rc4-source.json` (and references if renamed);
- canonical runtime/type hashes and identity in `web/vendor/manifest.json`;
- version/profile guards and build inputs in `scripts/vendor-taet.js` and `scripts/check-vendor.js`;
- the framework summary/references in SOURCE_PROVENANCE.json and current documentation;
- regenerated runtime/types/font/licenses and consumer compatibility checks.

Validate the candidate release in isolation before changing the pin. Keep exactly one components/foundation runtime and BaseControl identity for the app-owned SpectrumAnalyzer/ChannelIndicator. Preserve license notices. Do not relax hash guards or silently patch vendor bytes to make an update pass.

## Development checks

After npm ci, install the matching Playwright browser if absent:

```text
npx playwright install chromium
npm test
cargo fmt --check
cargo test --release --locked --all-targets
cargo test --release --locked --doc
cargo clippy --release --locked --all-targets
```

Browser installation is separate from WebView2 and is needed only for browser tests. npm test covers syntax/types/pinned vendor and 78 synthetic browser checks at 100/150% scaling. The current inherited Clippy warnings mean warnings-as-errors is not a passing gate. `npm run preview` starts a synthetic browser preview. Optional test helpers/examples are built with `--bins --examples`; they are not runtime application dependencies.

## Application icon generation

The user's original artwork and reviewed transparent master are committed under icons. `@tauri-apps/cli` 2.11.4 is an additional exact devDependency used only for `npm run icons`; it does not change the Rust Tauri runtime or add a runtime frontend dependency. Run `npm run test:icons` to verify all desktop PNG/ICO/ICNS representations and the favicon. See [icons/README.md](../icons/README.md) for alpha cleanup, source provenance and repeatability. Cargo-only builds still need no npm command because the generated assets are committed.

## Settings and optional native acceptance

Normal startup is live-capable but Stopped. Config is adjacent `config.json`; Windows WebView2 data is in per-profile user AppData. A Cargo target/release executable uses the project-root profile only when the Cargo source markers are present. A plain portable copy uses its own directory. `--fake` is explicit and defaults to separate local-data/preview; `--data-dir` explicitly selects an absolute profile. No profile fallback occurs after a validation/write/conflict error. See [INSTALL.md](INSTALL.md) and [CONFIG_STORAGE.md](CONFIG_STORAGE.md).

Native acceptance scripts are optional developer tools and require an environment where PowerShell script execution is permitted. Do not change or bypass policy to run them. The current environment blocks them; completed Rust/browser checks must not be reported as complete native coverage.

`scripts/native-smoke.ps1` uses fake playback in a fresh profile. `scripts/single-exe-smoke.ps1` tests a synthetic portable migration and save/restart from another working directory, never requesting audio. Its internal startup-smoke marker is restricted to synthetic fixtures. A timeout is an uncertain own launch: inspect its process/evidence before retrying. Manual native checks can use isolated candidate copies as described by delivery evidence.

The optional live-check tool requires explicit approved stream input and an isolated profile; it is not part of clean-clone build verification. The profile-verify example operates on an explicitly selected copy, checks canonical JSON save/load roundtrip without network, and reports legacy normalization differences. Normal `load_or_default` decoding is read-only; startup preparation owns verified migration writes. No personal station data is needed to reproduce the binary.

## Reproducibility limits

Lockfiles and vendor manifests pin dependency/source inputs; toolchains/system SDKs are external prerequisites. Cargo can use a pre-existing registry cache for an offline clean-clone build. Browser tests can use an already-installed matching Playwright cache. A clean checkout is not a newly provisioned machine. Windows PE/debug timestamps and PDB GUIDs can change whole-EXE SHA despite identical code/resources; bitwise-identical executables are not promised. macOS and physical native OS-input parity have not been established by these Windows checks.

## Clean-clone check, 2026-11-04

Two fresh local Git clones of source commit `54492a45bfae3acc85e8d35da685132e400612f6` were checked. Neither contained ignored profiles, binaries, node_modules or previous targets. The Cargo-only clone built the full release EXE in a new target directory with Node absent from its child PATH and no node_modules; locked Cargo registry sources were already cached. A separate developer clone used a fresh npm cache and successfully ran npm ci, vendor regeneration, type/syntax/integrity checks and all 76 browser tests. Regeneration changed no tracked file or lockfile.

The matching Playwright browsers were already installed on this machine. The documented browser-install command was checked with `--dry-run`; a new browser download and full machine/toolchain provisioning were not performed. No player EXE, native acceptance PowerShell script, radio session or personal profile was opened. The full Windows EXE rebuilt successfully from tracked inputs under these prerequisites; a fresh OS and bitwise-identical PE output are not claimed.

## Public CI

The Windows checks workflow pins official checkout/setup-node actions by commit, Rust 1.96.0 and Node 22.14.0. It runs locked builds, Rust regressions/doc tests/fmt/Clippy and synthetic Chromium UI tests, then scans the complete EXE for user-profile/build paths, common credential signatures and non-filename CodeView/PDB records. It uses read-only repository permissions, no personal profiles or real stations, and does not publish releases. Passing CI is not native OS-input, audio-device or macOS acceptance.
