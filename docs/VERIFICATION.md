# Single full executable verification

Æ Strøm 0.1.0, Tauri 2.12.1 and pinned published taet-ui 0.6.0-rc.4. Canonical executable: **ae-strom.exe in the project root**, the complete application, **18593280 bytes**, SHA256 **3a2201d8558b8c2aa152533cf42cae0c4aa4485216ec010c818b19e8abb84807**. The same full binary is at target/release/ae-strom.exe. No launcher is required.

## Behavior and data

Double-click starts real live mode, **Stopped**. Fake is explicit --fake only. All UI/assets/fonts/audio code are embedded; only normal Windows libraries and the installed WebView2 runtime remain external. A copied single EXE was run successfully without adjacent sources, Node/npm, fonts or web folders.

The portable data directory is **local-data/player beside the EXE**. The project-root delivery therefore uses the already transferred profile without copying, migrating, normalizing or overwriting it. Its hash was verified unchanged during the earlier single-executable migration. The present idle UI update did not open or write that profile. All existing stations, banks, EQ assignments/custom settings and other persisted fields remain. Personal settings are external editable data, never built into the EXE or Git. Exact private URL strings were scanned against the main EXE with zero matches. Git source/provenance checks also exclude configs/logs/binaries. The original player/profile and legacy configuration were not changed.

Cargo's project release build shares that same project-root profile when source markers exist. Moving only the EXE to another writable directory creates that directory's own fresh portable profile; move local-data/player/config.json too to retain stations. An absolute --data-dir override remains available. Normal startup never substitutes empty preview/AppData data. Unwritable/invalid profiles report a startup error. Earlier launcher/main binaries and a profile backup are preserved outside Git in the ignored rollback directory.

## Final gates

| Gate | Result |
| --- | --- |
| Browser | **76/76**, DPR 1/1.5; strict types, 28 JS syntax files and pinned vendor integrity PASS |
| Rust | Earlier **36 unit + 7 doc** and Clippy cover the unchanged Rust source; fmt and locked offline application release build rerun for this UI change |
| Native explicit fake | **36/36** final Tauri/WebView2 UI checks |
| Native single EXE | **7 checks × 2 starts**, default live/stopped, portable profile, embedded fonts/assets and persisted settings |
| Restart | First run loaded volume 17 and saved 18; second loaded 18 and saved 19; station banks/custom EQ retained |
| No autoplay | Native startup snapshot stayed stopped/requestedPlaying=false; engine diagnostics inactive; startup probes never requested Play |
| Earlier live backend | **2/2** approved bounded Play/Stop phases, app volume 10%, non-silent PCM/output callbacks in both, 4072 ms total |
| Cleanup | audio/http/metadata/sessionOwners all **0** after shutdown; system volume/device unchanged |
| Framework | Exact rc.4 runtime/types/fonts/licenses unchanged; one BaseControl, no new JS runtime dependency |

Seven existing Clippy style warnings remain. Audio/decoder/DSP/controller/config domain logic was preserved. The actual single-EXE native probes used a fresh synthetic profile marked explicitly for the test seam and ran from a different cwd; no personal profile was opened by these tests. A permitted Get-Process name inventory found no competing player immediately before the bounded audio test. No WMI/CIM retry or security change was used.

Native fake/single-EXE checks exercise real transport/DOM/window metadata, not physical OS input. Live evidence is a bounded Rust backend/device-callback check using the same current audio engine, not a claim of human-heard quality. User-heard sound quality, long listening, physical IME/accessibility/clipboard, actual multi-monitor DPI transitions, native 100%, non-Windows, installers and signing remain manual/pending. Prior startup geometry and current UI retain hidden-black font/layout-ready reveal; first compositor-frame observation is not claimed.

## Evidence and readiness

Ignored local-data/followup contains single-tests-final.log, single-doc-final.log, single-clippy-final.log, single-build-final.log, single-ui.log, single-native-final.log, single-live-summary.json and single-install.json. Single-EXE save/restart evidence: `local-data/single exe smoke 20261103-195830-325/result.json`. Final fake evidence: `local-data/native-smoke-20261103-195941-437/native-ui.json`. Private profile preservation/rollback hashes are kept only under local-data. SOURCE_PROVENANCE.json records public source and EXE hashes. delivery.json records the local commit and canonical path. No installer, shortcut, remote, push or publication was created. This is a runnable local single-EXE candidate.

## Current idle ticker update

The centered static -= Æ Strøm =- label appears only when stopped without an error. Play/Stop, stopped station selection, stale metadata, connection/reconnection/error priority, full-title copy, moving long titles, compact/drafts and forced 70 px ticker clipping pass at browser DPR 1/1.5. Native fake start/Stop checks and two default-live/stopped starts verify the same branding. Only app JavaScript/scoped CSS and acceptance harnesses changed; framework/vendor and Rust domain/host/config sources are byte-identical to the previous source record.

The optional gradient was trialed only in a synthetic preview, not shipped. Current evidence is under local-data/idle-brand-* and idle-brand-preview; screenshots at 100/150% were inspected. No personal-profile read/write or live audio was performed in this iteration. Earlier live backend evidence above is historical, with the same unchanged audio engine. Physical OS input/IME, native 100%, actual live metadata/listening and other platform limits remain as stated. The prior EXE and replaced reference PNGs are preserved in ignored idle-brand rollback.

## Current font audit

All app typography declarations now use the standard Tæt tokens, including native option DOM and hidden status. Actual custom-font use is confirmed by Chromium and loaded font/resource metadata in a copied single EXE on Windows. See FONT_AUDIT.md for the precise face/source, fallback and native select/title-tooltip exceptions. Mac/WKWebView was not executed and parity is not claimed. The current audit did not access personal config or start audio; the existing external player kept running.

## Windows WebView2 prerequisite notification

The app now calls the public `tauri::webview_version()` before Tauri Builder/profile/controller initialization. Failure or an empty/zero/malformed version yields the sanitized WEBVIEW2-RUNTIME native startup message with Microsoft's Evergreen download URL. No installer/download/browser-opening code is added. Existing setup/runtime failures retain their fallback messages.

Injected probe tests cover installed/channel versions, unavailable/error/malformed results and notification-before-exit. These are logic/notification-dispatch tests, not a physical absent-Runtime environment. The installed system Runtime was not removed, disabled or redirected; missing-Runtime GUI presentation remains unverified on a genuinely clean machine.
