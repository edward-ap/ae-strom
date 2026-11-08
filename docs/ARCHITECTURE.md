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


## Native menus and bank hotkeys

The Windows host disables WebView2 AreDefaultContextMenusEnabled on its UI
thread and reads the flag back before releasing the UI-ready barrier. This
changes no keyboard accelerator, clipboard or Taet event handling. The native
smoke startup evidence records defaultContextMenusEnabled=false. Other platform
menu policies are unchanged.

Physical bank-letter hotkeys also load that bank into Settings, even when the
panel is closed or playback already uses that bank. Settings bank buttons still
browse without changing playback. Editor/composition/modifier guards remain in
force; typing in a URL field does not select a bank.


In live playback, an empty selected slot retires its audio and metadata session but retains
requested_playing when Play was enabled. The snapshot reports status=stopped
and No stream URL without an error; it remains armed to start a configured slot
on the next selection. Stop clears the intent while waiting, and choosing a
station while stopped does not start it. This intent is not persisted across
application restarts. Existing terminal playback-error handling is unchanged.


Native layout changes keep the player-strip vertical anchor fixed and prefer
the physical right edge when the new width fits the monitor work area. The host measures outer/client bounds and converts the desired logical size
to physical bounds, including frame insets, DPI rounding and negative monitor
coordinates. Windows applies position and size together with SetWindowPos on
the UI thread. Host layout transactions are serialized, and the UI captures and
queues each requested layout. setCompact returns the layout completion promise
for native verification. This does not change startup placement or audio state.


Settings/EQ placement uses the work area of the monitor containing the player
strip, excluding reserved taskbar space. A panel opens below when it fits and
above otherwise when space permits; constrained displays use the roomier side
and bounded placement. The native layout reply supplies panelsAbove and the UI
orders the same live controls around the strip, without recreating editors.
Closing panels or collapsing restores the strip's own y coordinate. Position
persistence records the player strip rather than the top of an upward panel.
Opening direction is evaluated on layout changes; dragging an open panel does
not continuously flip its direction.


Native layout has a read-only preparation request so the WebView can install
its upward viewport anchor before the bounds commit. The upward strip stays
fixed to the client viewport bottom even when panel DOM is already hidden;
that anchor is removed only after the viewport has collapsed. Browser preview
keeps its document-flow canvas. The native bridge alone marks viewport hosting.
The visible Windows commit uses one UI-thread SetWindowPos without activation
or Z-order changes; hidden bootstrap retains Tauri's initial sizing path.
Panel offsets use extra client height, excluding the non-client frame. The
Windows 0.62.2 dependency is the same version already locked through Tauri.


Horizontal layout prefers preserving the right edge. If that would leave the
monitor work area, it preserves the left edge when possible, otherwise clamps
the position into the work area. This applies to micro-player expansion and
panel layout changes, using physical outer widths including native frame/DPI
rounding. Monitor gaps and taskbar exclusions are not treated as usable space.
A window wider than the work area is left-aligned as a best-effort fallback;
its canonical UI dimensions are not scaled. Smoke records horizontalAnchor so
validation checks the chosen edge instead of requiring right anchoring always.


Playback reconnect status preserves the last concrete cause: HTTP/setup,
stream read, decoder reset/decode, or CPAL output creation/start/callback. HTTP
and probe errors retain their source chain, while transport URLs are already
removed by the existing reqwest boundary. The generic retry text is only a
fallback when no cause exists. Successful loop entry clears the old cause;
explicit Stop wins. This changes diagnostics, not retry intervals or timeouts.


The player does not expose native hover titles on controls or the ticker, and
EQ value popups are disabled. Accessible labels, error descriptions, slider
values and keyboard editor behavior remain available. App-owned label updates
clear titles after icon changes; ticker rendering clears the title recreated
by ScrollingLabel.setText. The pinned framework remains unchanged.

HTTPS retains the transport's default certificate verification. There are no
per-station bypasses or automatic HTTPS-to-HTTP downgrades. Windows certificate
validity code CERT_E_EXPIRED (0x800B0101 / -2146762495) becomes the terminal
English message "Server certificate has expired or is not yet valid" instead
of repeated reconnect attempts. Schannel/native-tls hides the raw io::Error,
so recognition uses the numeric code retained in the error chain, independent
of the OS message language. Original details remain in diagnostic logging.
Other transport errors retain the existing classification and verification.
