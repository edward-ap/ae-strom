# Architecture

Settings station TextInputs present the saved station name while unfocused and
the URL while focused; an unknown name falls back to the URL. Presentation swaps
use silent public control mutations and never count as edits. URL drafts and
in-flight commits are keyed by browsed bank/slot, independent of the displayed
text. Enter/blur commits only a real draft, coalesces an identical pending save,
and preserves rejected/newer edits. Successful replies acknowledge saves even
when their older snapshots are correctly rejected by the revision guard. State
updates do not replace a focused editor's value or selection.

Playback requests `Icy-MetaData: 1` on its audio GET. A valid positive
`icy-metaint` selects metadata from that same response: a streaming reader removes
ICY framing before Symphonia and sends normalized titles with the audio request
ID. No parallel metadata connection is opened, even when a cached ICY/JSON hint
exists or the station temporarily sends only empty metadata blocks. Framing and
title deduplication reset with each audio reconnect.

Without ICY, playback first consumes Symphonia's probe/container metadata logs.
ARTIST plus TITLE, or a complete TITLE, supplies the current track. All new
revisions are consumed, including metadata accompanying Ogg logical-stream
changes; artwork-only revisions do not hide title revisions. Only when both ICY
and a usable container title are absent is the metadata provider enabled. A later
container title retires an already-running fallback; a new Ogg link without a
title can enable it again. Configured
`metadataType`/`metadataUrl` are then fallback hints, including FLAC plus an AAC
sibling. Existing sibling discovery and JSON fallback remain available; the
provider does not re-probe the audio URL already checked by playback. In-band
success does not overwrite a configured fallback URL. The session owner retires
the previous fallback on a capability change and rejects callbacks from old
requests/sources/generations. Titles follow decoder read-ahead, so a small lead
relative to audible PCM remains possible. This routing does not establish or
enforce the station's absolute live edge.

Station names are independent of track titles. The audio response's icy-name
continues to supply the station name regardless of icy-metaint. Explicit
container station fields (STATION, STATION_NAME, STATIONNAME, RADIO_NAME,
RADIOSTATIONNAME, ICY-NAME and ID3 TRSN aliases) and secondary-source names fill
only an empty config name; the controller checks emptiness at event application
time. ARTIST, TITLE, ALBUM, ORGANIZATION and PUBLISHER are not guessed as station
names. No new persisted config fields or frontend contracts are introduced.

Rust owns playback state, the serialized action queue, sessions, metadata, DSP, decoder/resampler and persistence. The WebView owns presentation, panels and unsaved editor drafts. The only transport-aware UI module is `web/bridge.js`: Tauri invoke for commands and radio-state/spectrum listeners for updates. JSON contracts are hand-maintained in Rust Action/Snapshot/Spectrum and `web/contracts.d.ts`.

Tauri commands enqueue actions to the bounded 256-message controller queue. Replies wait in spawn_blocking; blocking session retirement runs on radio-session-owner. A 50 ms controller cadence publishes spectrum while playback is requested; metadata/state update through events. Revisions reject stale snapshots and generation/cancellation guards reject old-session callbacks. The five spectrum amplitudes cross IPC; PCM remains native Rust.

Configuration writes run on one radio-config-writer thread, outside the controller. A single pending config replaces intermediate edits; writes wait for 300 ms of quiet, capped at one second of continuous edits. Title-only metadata and unchanged station names/strategies do not schedule saves. Background write failures/recovery are published in snapshots. Shutdown bypasses the debounce, waits behind any older write, and flushes the latest config; a failed flush leaves the controller and writer available for correction/retry. Abrupt termination can lose pending edits. Atomic replacement and file sync remain in the storage layer.

All ordinary controls are Tæt UI live instances. The public components bundle has one foundation identity. SpectrumAnalyzer and ChannelIndicator are application-owned BaseControl extensions, with their own geometry/CSS/timers and owner teardown. Spectrum uses public lifecycle overrides and an app-owned mutation guard, without BaseControl underscore hooks. No second framework/BaseControl or runtime JS dependency is added. The framework JS/CSS/fonts/types are immutable pinned delivery bytes.

UI readiness requires both event subscriptions, the initial snapshot, bundled fonts and successful native layout. Initialization errors reject readiness, release controls/subscriptions and reach the host bootstrap failure handler; no ui_ready signal is sent. Destroying the owner during initialization prevents subsequent layout/reveal work.

Config supports historical presets and sparse stationBanks layouts, compatible EQ-name normalization, URL-edge trimming and atomic saves. Default startup can discover the previous local-data/player/config.json and migrate through validation, verified backup and an AppData receipt while preserving the source. Conflicting profiles fail closed. Explicit --data-dir disables previous-profile discovery. See [CONFIG_STORAGE.md](CONFIG_STORAGE.md) and [CONFIG_COMPATIBILITY.md](CONFIG_COMPATIBILITY.md) for current selection, validation and recovery rules.

Startup mode/profile routing is in src/launch.rs; src/profile.rs prepares/locks the selected profile. Default is live/stopped, with portable config.json beside the EXE; the normal Cargo release layout shares the project-root config. WebView data, locks and migration receipts live separately in platform app-local data. Fake is explicit and isolated under local-data/preview by default. There is no launcher dependency or implicit alternate-profile fallback.
