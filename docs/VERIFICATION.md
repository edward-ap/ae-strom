# Configured live-audio candidate

Æ Strøm **0.1.0**, Tauri 2.12.1, pinned published **taet-ui@0.6.0-rc.4**, source bfd3ddcddf1cdc539397e5d2ac5ed3cb976e72c2. No framework vendor/core patch or JS runtime dependency was introduced.

## Normal desktop launch

Double-click **ae-strom-player.exe** in the project root. No command-line flags, console window or script host are required. It launches the full `target/release/ae-strom.exe` with explicit live mode and the absolute project-local **local-data/player** profile. Playback starts **Stopped**; use Play. The main GUI run directly without flags remains a silent preview for development.

The user explicitly authorized copying the existing working profile. Its source was read-only, the destination did not previously exist, and every JSON field was preserved exactly. Rust schema validation and exact save/load roundtrip passed on a separate audit copy before installation. All station banks/slots, metadata settings, EQ assignments, custom preamp/bands, volume/mute, active selection, window geometry and polling settings were retained. No backup/overwrite was needed. No personal config, URLs, metadata or transfer reports are in Git. Original sources/EXE and the original legacy configuration remain untouched.

## Final gates

| Gate | Result |
| --- | --- |
| Browser | **64/64**, DPR 1/1.5; syntax, strict types and pinned vendor checks PASS |
| Rust | **30 library + 4 launcher unit tests, 7 doc tests**, fmt, Clippy, locked offline release bins/examples build PASS |
| Native GUI | **33/33** real Tauri/WebView2 fake checks on the final main EXE; prior startup matrix covered four saved-position cases |
| Native launcher | Actual Windows process test with a synthetic child: explicit live flag, absolute profile path and quoting through a directory containing spaces PASS; no user profile or audio opened |
| Profile | Offline validation and exact all-field JSON roundtrip; source hash before/after unchanged; private transfer verification PASS |
| Live backend | **2/2 bounded Play phases with Stop between**, app volume 10%, non-silent PCM and output callbacks in both, total 4106 ms |
| Cleanup | audio/http/metadata/sessionOwners all **0** after shutdown; system volume/device unchanged |

The permitted Get-Process name inventory found no competing radio/player before the live test. WMI/CIM was not retried or bypassed. Only the explicitly user-approved sample stream was contacted; imported station URLs were not fetched. Live evidence was captured privately. PCM/callback evidence establishes decoding and real device callbacks; **user-heard quality is unconfirmed**. Seven existing Clippy style warnings remain.

## Executables

- Normal launcher `ae-strom-player.exe`: **253440 bytes**, SHA256 `7a91253bbc7c1d4aff678ec5b7b8b73c879f5290a094bd9efd501b18d5099b1f`.
- Main GUI `target/release/ae-strom.exe`: **18595840 bytes**, SHA256 `89b129bad8c0272ea6a7b202688fa8c8e47d65c6308db4027e126e2304ed4d96`; live-audio compiled in.

The full main executable can also be invoked explicitly:

```powershell
./target/release/ae-strom.exe --enable-live-audio --data-dir (Join-Path $PWD 'local-data/player')
```

## Preserved UI and remaining acceptance

Normal 725×36, compact 102×36, Settings 725×258 and EQ 725×257 logical px. The protruding status row remains removed; errors use ticker/live feedback. Native creation remains hidden/black until font/UI geometry and restored/clamped/centered monitor position are ready. Noto Sans 12 px is local, integrity-checked and uniform. All accepted Settings/EQ/channel/spectrum controls and interactions remain.

The earlier four-case startup evidence verifies stable exact native geometry before/after show. Immediate per-layout diagnostics may capture a prior Windows size while rapid asynchronous resize messages settle; they are not treated as settled panel-frame acceptance. Native physical mouse/keyboard/IME/accessibility, first-compositor-frame visual observation, multi-monitor hardware transitions and native 100% still require manual checks. Current native host is WebView2 156.0.4314.58, scale 1.5. Longer listening, non-Windows, installers/signing remain pending; this is a local runnable candidate, not a public signed release.

## Evidence and Git

Ignored local-data holds profile-transfer.json, the copied audit profile/roundtrip, followup/live-summary.json, launcher probe evidence, Rust/browser/native logs and delivery.json. Final GUI native record: `local-data/native-smoke-20261103-191945-605/native-ui.json`. SOURCE_PROVENANCE.json records public source and EXE hashes without personal content. No shortcut, system configuration, installation, remote, push or publication was created.
