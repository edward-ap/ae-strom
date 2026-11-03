# Migration and verification

Æ Strøm **0.1.0** is a local Windows test candidate. Project/crate: ae-strom / ae_strom. Native title: Æ Strøm. App identifier: local.edwardap.ae-strom. Tauri is pinned to 2.12.1 and Tæt UI to published **0.6.0-rc.4**, components ESM, source commit bfd3ddcddf1cdc539397e5d2ac5ed3cb976e72c2. npm integrity, source hashes and canonical runtime assets are recorded in provenance/taet-ui-rc4-source.json and web/vendor/manifest.json.

## Final checks

| Gate | Result |
| --- | --- |
| Browser | **64/64**, Chromium DPR 1/1.5; URL selection, controls, drafts, spectrum, startup/font gating, errors inside chrome |
| Syntax / types / vendor | 24 JS files, strict public API declarations, 6 pinned runtime assets and 85 matching declarations plus facade; PASS |
| Rust | **30 unit + 7 doc**, fmt, Clippy and locked offline release build PASS |
| Native Windows | **33/33 fake checks in each of 4 launches**, actual WebView2 156.0.4314.58, scale 1.5 |
| Native startup | Absent, valid, offscreen and edge-clamped saved positions; hidden before readiness, stable position/size before and after show |
| Visual | Local Settings/EQ screenshots inspected; standard font loaded at browser DPR 1/1.5; Unicode font fixtures captured |
| Original preservation | All 151 original source hashes and old EXE hash unchanged; no personal profile read/imported/copied |
| Rust domain preservation | 19 files outside native host/lib exports identical after crate-reference renaming; audio/controller/config logic unchanged |
| Dependencies | All 537 Rust package versions/checksums retained; no JavaScript runtime dependency in package.json |

Seven pre-existing Clippy style warnings remain. No automatic fixes changed audio/DSP code. Published framework/vendor bytes are unmodified. App-owned SpectrumAnalyzer and ChannelIndicator share the one framework BaseControl. npm, TypeScript, esbuild and Playwright are development tools and are not required by the packaged UI at runtime.

## User follow-up fixes

- The visible status row is removed from layout. Selecting an empty slot cannot extend the 36 px chrome. Genuine errors appear inside the ticker and on accessible control descriptions/live region, including compact mode; no extra panel opens.
- The native window is created hidden with black host/WebView background. A typed ui_ready IPC waits for actual bundled font readiness and settled UI geometry. Host setup readiness uses a condition variable, not a timer. Monitor-aware physical positioning and logical sizing complete before show. Valid coordinates are retained; edge positions clamp; absent/offscreen coordinates center on the primary monitor. Pure tests also cover negative monitors, mixed DPI and extreme coordinates.
- The standard bundled Noto Sans family and 12 px size are applied consistently to URL/select fields, ticker, buttons, bank/channel text, panels and EQ labels/values. Previously most already inherited Noto Sans, but URL text was 10 px, toolbar 14 px, body 11 px and annotations 9–10 px. The unchanged WOFF2 is verified by hash, loaded locally, and awaited before reveal. The EQ scale label box was widened from 36 to 44 px to fit the standard text without enlarging the window.

Normal **725×36**, compact **102×36**, Settings **725×258**, EQ **725×257** logical px. The 110 px Settings presets, 24×24 native-select arrow areas, 40×26 fixed channel frame and existing UI interactions are retained. The earlier pre-follow-up six static images matched the source app; final font/error changes are intentional and are not claimed pixel-identical.

## Binary and launch

`target/release/ae-strom.exe`: **18595840 bytes**, SHA256 `0301161123ed613453bd99b0a68191dcf08f2cb6716deafea0b3b0e450bafde4`. Compiled live-audio capability is opt-in; default mode is fake/stopped.

```powershell
./target/release/ae-strom.exe --fake
```

Explicit isolated preview:

```powershell
$profileDirectory = Join-Path $PWD 'local-data/manual-preview'
./target/release/ae-strom.exe --fake --data-dir $profileDirectory
```

Default Windows roots are `%LOCALAPPDATA%/AeStrom/preview` and, only in explicit live mode, `%LOCALAPPDATA%/AeStrom/player`. Each keeps separate config/WebView data. No old data is discovered or imported automatically. Initial pre-follow-up binary rollback is kept outside Git in local-data/followup/rollback.

## Limits and evidence

No real audio was started. The old player was active at migration start; a later default-permission process-inventory query was denied, so its current activity was not assumed cleared and live testing remained deferred. No other process, system volume, output device or personal config was changed. This does not block fake/native verification.

Native results are real Tauri IPC/DOM/window metadata, not physical mouse/keyboard tests or observation of the first compositor frame. They establish hidden creation, font readiness, exact pre-show dimensions and stable position, plus the final 36 px native chrome; a human should still confirm absence of a visible flash. Immediate per-command layout diagnostics can report the previous asynchronous Windows size during rapid panel switches; they are not treated as settled resize acknowledgements. Panel content geometry is asserted in the DOM/browser, not claimed as physical native panel-frame acceptance. The current host has 1 monitor(s); other monitor/DPI configurations are covered by pure tests, not claimed physically exercised. Native 100%, actual IME/accessibility, non-Windows, installers/signing and prolonged listening remain pending.

Final Rust/browser logs and synthetic startup profiles are ignored under local-data/followup. Startup evidence: `local-data/followup/startup-matrix-20261103-185040/summary.json`. Screenshots are local evidence, never published or put in Library. SOURCE_PROVENANCE.json records source and binary hashes; the initial local Git commit and delivery.json identify the reviewed snapshot. No remote, push, tag or publication was created. Ready for local manual evaluation, not a signed/public release.
