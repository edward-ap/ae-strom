# Changelog

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
