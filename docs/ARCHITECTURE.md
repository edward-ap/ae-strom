# Architecture

Rust owns playback state, the serialized action queue, sessions, metadata, DSP, decoder/resampler and persistence. The WebView owns presentation, panels and unsaved editor drafts. The only transport-aware UI module is `web/bridge.js`: Tauri invoke for commands and radio-state/spectrum listeners for updates. JSON contracts are hand-maintained in Rust Action/Snapshot/Spectrum and `web/contracts.d.ts`.

Tauri commands enqueue actions to the bounded 256-message controller queue. Replies wait in spawn_blocking; blocking session retirement runs on radio-session-owner. A 50 ms controller cadence publishes spectrum while playback is requested; metadata/state update through events. Revisions reject stale snapshots and generation/cancellation guards reject old-session callbacks. The five spectrum amplitudes cross IPC; PCM remains native Rust.

All ordinary controls are Tæt UI live instances. The public components bundle has one foundation identity. SpectrumAnalyzer and ChannelIndicator are application-owned BaseControl extensions, with their own geometry/CSS/timers and owner teardown. No second framework/BaseControl or runtime JS dependency is added. The framework JS/CSS/fonts/types are immutable pinned delivery bytes.

Config retains sparse stationBanks/legacy import compatibility, custom EQ collision handling, URL trimming on explicit import and atomic save behavior. Naming migration changes only crate references and app identity/data root; it does not change audio or JSON semantics. No automatic discovery/migration of prior profiles exists.
