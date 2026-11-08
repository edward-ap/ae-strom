# Changelog

## 0.1.2 — 2026-11-08

### Playback and errors

- Preserve Play intent when selecting an empty bank or slot. Stay silent there
  and resume on a configured selection; an explicit Stop cancels that intent.
- Keep concrete HTTP, decoder and output-device causes in startup/reconnect
  messages instead of replacing them with a generic interruption message.
- Report Windows certificate validity failures in English and stop retrying
  them. HTTPS verification stays enabled without exceptions or HTTP downgrade.

### Window placement

- Collapse along the right edge and expand from whichever edge keeps the player
  inside the monitor work area, including left-edge and mixed-DPI placement.
- Open Settings and EQ above the strip when there is insufficient space below.
  Preserve the strip position when closing or collapsing.
- Prevent jumps during upward panel transitions by preparing the direction and
  applying Windows size/position together; serialize rapid layout requests.

### Interface

- Make bank letter shortcuts update the Settings page, including an already
  active bank and changes while Settings is hidden. Preserve editor key guards.
- Disable default WebView2 context menus while retaining keyboard Copy/Paste.
- Remove hover hints from all controls and the ticker, including transient EQ
  values. Preserve accessible labels, keyboard control and error descriptions.

### Verification and implementation history

- 111 Rust tests, 112 synthetic browser tests at 100/150% scale, and 40 native
  fake-backend checks cover the accumulated fixes. Publication build evidence
  and platform limits are recorded in [docs/VERIFICATION.md](docs/VERIFICATION.md).
- [40ec49b](https://github.com/edward-ap/ae-strom/commit/40ec49b598673957e65d5e9a9b51f19bf9da0970)
  contains the playback, geometry, Settings, menu, hint and diagnostic fixes.

## 0.1.1 — 2026-11-07

### Playback and metadata

- Read ICY metadata from the audio connection when advertised, keeping track
  titles tied to the session being played. Strip ICY blocks before decoding and
  reconnect audio/metadata together.
- Read native FLAC and Ogg-FLAC container tags when ICY is absent, including
  chained-stream title changes. Use configured metadata URLs and lower-bitrate
  discovery only when the played stream provides no title.
- Fill empty station names from explicit container station tags or secondary
  metadata sources; preserve existing names and track titles.
- Retain opt-in AAC/playback/HTTP buffer timing counters and loopback tests.
  These measurements distinguish application buffering from session divergence;
  they do not establish a universal server-side cause.

### Interface and persistence

- Show station names in unfocused Settings URL fields; reveal the URL on focus.
  Save only edited URL drafts and preserve edits/selection during metadata
  updates, delayed replies and rejected saves.
- Reject failed UI startup instead of reporting readiness, and clean up late
  subscriptions and controls after disposal.
- Use the public Taet lifecycle API in SpectrumAnalyzer, including reentrant
  destruction and mutation guards.
- Serialize and coalesce config writes off the controller thread, flush on
  shutdown, report failures and avoid saves for title-only metadata changes.
- Add sanitized configuration field diagnostics and a read-only `config-check`
  example. Preserve legacy Unicode/padded EQ identities and valid URL path/query
  spaces without weakening structural validation.

### Tools and verification

- Port `icy_peek` from mini-radio: ICY headers, raw blocks, changed titles, JSONL
  logs, reconnects, notes and fresh-session probes without audio playback.
- Remove temporary CMD audio/timing clients and the older live-check executable;
  retain automated Rust/browser tests and synthetic media fixtures.
- Expand coverage for startup failures, Settings drafts, configuration
  compatibility, metadata priority, container chains and worker shutdown.

Detailed checks and historical local deliveries are recorded in
[docs/VERIFICATION.md](docs/VERIFICATION.md). The Windows x64 release includes
the player, metadata-only inspector, license archive, build information and
SHA-256 sums. Earlier commits remain in Git history.

### Implementation commits

| Commit | Change |
| --- | --- |
| [44acf16](https://github.com/edward-ap/ae-strom/commit/44acf16) | Safe configuration diagnostics and legacy-value compatibility. |
| [74b4a1d](https://github.com/edward-ap/ae-strom/commit/74b4a1d) | Playback-session metadata, container tags, background saves, timing coverage and metadata-only icy_peek. |
| [b2ea902](https://github.com/edward-ap/ae-strom/commit/b2ea902) | Settings station names, URL drafts and public UI lifecycle fixes. |
