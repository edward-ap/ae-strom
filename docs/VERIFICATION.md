# Single full executable verification

This is an append-only verification history. Older delivery hashes, temporary
tools and profile paths below describe those earlier iterations; the latest
dated entry and current README describe the maintained build and tools.

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

## Local review fixes, 2026-11-07

Startup now rejects synchronous/asynchronous subscription failures, initial-snapshot failures and native-layout failures instead of fulfilling readiness. Regressions exercise the actual main/bridge bootstrap with synthetic injected failures: no ui_ready signal, shutdown requested, owned controls removed. Late subscription resolution releases its listener; destruction while the initial snapshot is pending rejects readiness without late layout work. Spectrum now relies on public BaseControl lifecycle overrides and LifecycleError, with no inherited underscore hooks; veto, disable/enable, reentrant destroy and post-destroy mutations are covered.

Configuration writes are serialized on one background worker, with a replaceable pending config, 300 ms debounce and a one-second scheduling cap. Tests block an in-flight save while submitting a burst, verify latest-state flush ordering/coalescing, retry after failed flush, and background error/recovery publication while stopped. Title-only metadata and unchanged station names/strategies no longer schedule writes. Architecture/consumer/storage notes describe current behavior and the existing profile migration paths.

Checks passed: npm test (syntax, public types, vendor/icons and 92 Chromium checks at DPR 1/1.5); all-target Rust tests (61 library + 11 integration, examples compile); seven doc tests; cargo fmt --check; standard release Clippy with the same seven inherited warnings and none in the new code. After the final startup catch cleanup, its focused 14-browser-check suite passed again. An initial lifecycle test used a subscription as a function; corrected to its public dispose method. The first doc-test attempt failed on a missing rlib during overlapping Cargo commands; the subsequent isolated serial run passed 7/7 without source changes.

No Tæt source/vendor bytes, real user profiles or existing config-compatibility source changes were modified by this work. Tests used fake playback or loopback PCM sinks; real audio and native WebView2/physical-input acceptance were not run. This is a local candidate, with no push/publication.

Release build passed with cargo build --release --locked --bin ae-strom. The project-root ae-strom.exe was replaced and its SHA256 verified against target/release: da054d8ad89a6241009f75b58bc1b0489aafea13821791af1f1b7845a8ea39cf (18,838,528 bytes). The previous root executable and delivery record are retained under ignored local-data/review-fixes-20261107-0798b655/. The candidate was not launched and no real profile was opened. Final format and whitespace checks passed.

## Deutschrock CMD diagnostics and icy_peek port, 2026-11-07

Added ae-strom-icy-watch (production AudioEngine, independent direct-ICY observer,
Metal EQ) and icy_peek (metadata only). The latter preserves the old mini-radio
tool's positional URL, header/raw-block/title inspection, using Strom's existing
transport and parser rather than another HTTP dependency. Both default to the
user-selected Deutschrock URL. Neither reads or writes a player profile. CMD
launchers, timestamped JSONL capture and manual mark/probe/audio-only restart/full
restart/quit commands are described in ICY_DIAGNOSTICS.md.

Shared HTTP DEBUG traces add request IDs, caller roles, response headers/session
IDs, remote/final URLs, received/read bytes and termination reasons. Direct-ICY
traces include raw nonempty blocks before title normalization and their byte
positions; the audio decoder identifies its own HTTP request. Existing transport
timeouts, retry policy, audio decoding and GUI-default WARN logging are retained.
This is a capture tool, not a change to the dual-connection playback architecture
or proof of the station's reported long-running fault.

All-target release tests passed 78/78: 61 library, four CLI argument cases and 13
integration cases. New subprocess tests bind only loopback, deliberately close an
ICY response after one block, verify reconnect to a new title, raw block/header/
session/EOF logging, monotonic log order, manual markers, a fresh probe without
restarting the original observer, and zero remaining workers at shutdown. The
existing PCM-sink regressions cover audio output behavior without a real device.
Formatting and standard release Clippy passed, with the same seven inherited
warnings and no new ones. Final help/topology wording was clarified after the full
test run; Clippy/build check that final source. No UI/vendor files were changed in
this diagnostic iteration, so its earlier browser results were not rerun.

No public-radio playback, native sound-device acceptance or hour-long station
reproduction was run. The operator must supply human-heard mismatch markers during
a manual session; the tool does not identify or record the audible music. Private
profiles and the existing local configuration work remain untouched.

Both release executables were built and copied to the project root, with hashes
verified against target/release. ae-strom-icy-watch.exe: 7,073,792 bytes, SHA256
193ff507d69a99a8f356dbac2d359baff02c2ad1dd17e83ee029e6968d7252c6.
icy_peek.exe: 7,073,792 bytes, SHA256
aa89c5749ed067bc776e68863ec87bb5e9fc40ccc9f7970c0fa88ea7f955b9e2.
The delivery record is in ignored local-data/icy-tools-delivery-20261107-b13ac57a/.
Both CMD launchers passed --help through cmd.exe without opening network/audio;
final formatting and whitespace checks passed. No live test was started.

## Opt-in AAC tempo clocks, 2026-11-07

Added passive timing collection through AudioEngine::new_with_timing for the CMD
client. The normal engine constructor leaves it disabled. Independent clocks now
cover raw ADTS header duration, demux packet/time-base duration, decoded frames at
their reported rate, resampler input/output duration, device callback duration,
and samples actually consumed from PCM. Underruns distinguish empty-buffer zeros
from genuine silence/mute. Snapshots expose formats/rate changes, sync loss,
decode errors, packet discontinuities and buffer occupancy. Clocks are scoped to
one HTTP connection and frozen on exit; CLI rates reset across reconnects and use
monotonic window deltas. The JSONL retains full counters while the console shows
compact TEMPO lines. Startup/read-ahead and live-edge limits are documented in
ICY_DIAGNOSTICS.md.

All-target release tests passed 88/88 (67 library, eight CLI, 13 integration).
New tests cover fragmented/CRC/truncated ADTS framing, raw header rate changes
versus the pinned demuxer's fixed initial time base, core-versus-decoded clock
conversion, muted/silent/empty PCM, reconnect-window isolation and 0.9 versus 1.0
s/s reports. A paced 44.1-kHz loopback PCM source passes through the real decoder
and resampler into the 48-kHz device-free sink, with source/decoded/generated/
consumed durations agreeing. Synthetic ADTS payloads validate framing clocks, not
AAC decoder fidelity. Standard Clippy passed with the same seven inherited
warnings; formatting and whitespace checks passed. No browser files changed.

The original running console process and its drifted session have been left
untouched. The new build is delivered as ae-strom-icy-timing.exe with
icy-timing.cmd alongside it, so the operator can switch sessions deliberately.
Live timing measurements require that new process; previous logs lack these
clocks and do not establish a sample-rate or tempo defect.

The release CMD build was copied under the separate ae-strom-icy-timing.exe name
and verified: 7,188,480 bytes, SHA256
021ec5b9cdf090f3726dc62276875cccbd52b70d250f1cec1cc08d8b1029083a.
The original ae-strom-icy-watch.exe still hashes to
193ff507d69a99a8f356dbac2d359baff02c2ad1dd17e83ee029e6968d7252c6;
its running process was not stopped or replaced. Delivery metadata is retained in
ignored local-data/icy-tempo-delivery-20261107-5acef5b2/. CMD --help passed without
network/audio. Live timing results will be recorded separately after the operator
switches to the instrumented process.

The operator started the instrumented process (PID 111740), initially reporting
agreement on Erdling - Frei Wie Der Wind, then the audible transition to Erik
Cohen - Gelsenkirchener Barock. A warm-up-excluded 110.8784-second interval showed
110.8521 seconds of demuxed AAC (0.9997625 s/s), raw ADTS read duration at 1.0018567
s/s (read-ahead endpoint variation), decoded duration at 0.9997625, generated
output at 0.9999543, and device/consumed PCM at 1.0000144 s/s. No interval underruns,
decode errors, header rate changes or ADTS sync losses occurred. The input/core
rate was 22,050 Hz, output 192,000 Hz stereo, with 0.49 seconds remaining in PCM.
The private snapshot/summary is in ignored
local-data/icy-tempo-baseline-20261107-f2a4a6d5/. This establishes normal initial
tempo for that interval, not the cause or absence of the later long-session drift.

## Opt-in HTTP ingress and control station, 2026-11-07

Extended the CMD-only timing path to observe raw ADTS duration at the moment
reqwest yields each audio body chunk, before enqueueing. Atomic accounting tracks
producer-pending, queued, reader-pending, delivered and discarded bytes, peaks and
producer backpressure, including partial reads and cancellation. The console now
compares ingress speed with decoder/output clocks and estimates HTTP, decoder
read-ahead and total application pending media. Per-request checks prevent mixing
connections; ICY-framed bodies are excluded from bare-ADTS measurements. Error
traces retain nested HTTP failure details and timeout/body/decode flags, rather
than only the generic response-body error message. Transport queue capacity,
timeouts, retry and playback behavior are unchanged.

All-target release tests passed 92/92 (69 library, ten CLI, 13 integration).
New tests cover multiple minutes of framing received before any consumer read,
full-queue accounting, partial reads, cancellation cleanup, the ICY guard and
reporting an ingress/egress duration gap without blaming the PCM queue. These use
synthetic ADTS framing or loopback traffic, without a real sound device. Formatting
and whitespace checks passed. No UI or profile changes were made in this
diagnostic iteration.

The user supplied Frequence3 AAC as a sequential control station. Separate
icy-http.cmd and icy-frequence3.cmd launchers select the instrumented binary and
the corresponding default/control URL. Delivery leaves the earlier running
timing executable untouched. The counters begin after reqwest yields a body
chunk; they cannot distinguish upstream buffering in reqwest/TLS/OS/network from
server-side queues, or independently identify the station's live edge.

The operator ran the Frequence3 control in PID 98372, starting with
TimeMachine1985 - Honda Prelude, and reported that a fresh browser still played
the same music note for note after approximately half an hour. At capture the
client had run 2,820.883 seconds (about 47 minutes). Over a warm-up-excluded
2,756.284-second interval, source/decoded speed was 1.0000063 s/s and consumed
PCM 1.0000022 s/s; HTTP ingress was 1.0002254 s/s. HTTP pending duration was
5.10839 seconds at both interval endpoints, ranging from 3.1115 to 5.8050 seconds
across snapshots, with a peak retained byte count of 48,881. PCM stayed at
0.49 seconds. There were no interval underruns, decode errors, ADTS rate changes,
sync losses, HTTP errors or reconnects. Raw and decoded rates were 22,050 Hz and
output 192,000 Hz stereo. Evidence is retained under ignored
local-data/icy-frequence3-control-20261107-56bdc508/. This control argues against
a universal engine tempo error; it does not exclude client behavior specific to
Deutschrock, and the earlier Deutschrock run lacked HTTP ingress instrumentation.

The instrumented Deutschrock run in PID 46360 started with Joachim Witt - Das
Leben in mir, confirmed audibly by the operator. At 1,587.619 seconds (26.5 minutes)
the operator reported no lag and client audio approximately 3-4 seconds ahead of
browser playback. Over 1,523.187 warm-up-excluded seconds, source/decoded speed was
0.9999969 s/s, consumed PCM 1.0000017 and HTTP ingress 1.0002103. HTTP pending
duration was 2.414875 seconds at both endpoints, ranging from 1.9505 to 3.0186;
total received-but-unplayed media was approximately 3.32 seconds at capture. There
were no interval underruns, decode errors, rate changes or framing losses, and
audio remained on its initial connection. Five observer connections did fail:
new error details identify nested TimedOut / isTimeout=true errors, rather than
evidence of an explicit server-initiated disconnection. The transport has a
three-second read timeout. Those reconnects did not reproduce the earlier audio
delay in this run. Evidence is under ignored
local-data/icy-deutschrock-http-20261107-46360/browser-comparison*. This is a
passing interval, not a demonstrated fix or an established cause of prior drift;
browser playback is a separate session rather than an absolute live-edge clock.

The same Deutschrock run subsequently reproduced session divergence at roughly
39-41 minutes. The operator started the GUI from F:/miniradio/ae-strom.exe at
16:59:25 UTC and reported both heard audio and ticker as Kaerbholz - Ewig Leben,
then In Extremo - Frei Zu Sein; the existing CMD audio played Die Toten Hosen -
Disco. Transient ICY-only probes from the unchanged CMD binary returned Kaerbholz
at 17:02:13 UTC and In Extremo at 17:03:17 UTC, agreeing with the newer GUI while
the established observer progressed from Erik Cohen - Totenspinnengeist to Die
Toten Hosen - Disco. All inspected connections used the same station URL and
remote 194.97.151.145:443. The old observer continued receiving/consuming bytes;
the disagreement was not only a frozen UI title. The GUI binary is an older,
different hash from the reviewed workspace build, but divergence between two
metadata connections in the same CMD binary does not depend on that difference.

At 2,479 seconds, over a 2,414.572-second warm-up-excluded window, source AAC
speed remained 0.9999910 s/s and played PCM 1.0000033. HTTP pending duration was
2.5078 seconds and total application pending about 3.0624 seconds, with no rate
changes, sync losses or decode errors and no audio reconnect. This excludes
minutes retained in the measured pipeline after reqwest yielded body chunks at
that capture. It does not distinguish reqwest/TLS/OS/network/server buffering or
establish whether the programme sequence is chronologically delayed versus
session-specific. Neither audio nor the existing observer was restarted by the
fresh probes. Evidence is retained under ignored
local-data/icy-deutschrock-http-20261107-46360/confirmed-session-divergence*.

The operator subsequently heard Kaerbholz, then In Extremo in the old CMD audio,
supporting a delayed sequence rather than merely unrelated current songs. The
In Extremo heard marker was captured 493.518 seconds after the fresh probe had
already returned that title. This is about eight minutes between observations,
not an exact song-onset offset. At the marker, the CMD observer showed Frei.Wild
- Allein Nach Vorn, HTTP pending was 2.5078 seconds and total application pending
3.0163 seconds. A separate 12-second, metadata-only connection also returned
Frei.Wild and consumed all 344,022 received bytes at teardown.

Passive WebSocket reads bound to the ORIGINAL AUDIO Sessionid returned Kaerbholz
at 17:09:12 UTC and In Extremo at 17:11:34 UTC, matching the operator's delayed
audio observations independently of the in-band observer. This is strong evidence
for progression associated with that server session, rather than minutes held
after reqwest yields chunks. The WebSocket protocol and start_time_unix semantics
are undocumented; timestamps are retained as raw evidence and are not treated as
authoritative matched onset times. Exact server/transport queue mechanics remain
unresolved. Evidence is retained in server-session-title-evidence.json,
lagged-audio-and-observer-websocket.json, fresh-icy-twelve-seconds.jsonl and
in-extremo-late-transition.json under the same ignored incident directory. The
long-running audio and observer connections remain unchanged by these captures.

Final release Clippy completed with the same seven inherited warnings and no new
ones. Both CMD launchers passed --help without network/audio. The delivered
ae-strom-icy-http.exe matches target/release: 7,187,968 bytes, SHA256
4ec98e51e178166cb368fa56ec2e6b966fdaedaccbe07c31f738edf09b63c07f.
Earlier watch/timing binaries retain their recorded hashes; PID 111740 remained
running during delivery. The private delivery record is under
local-data/icy-http-delivery-20261107-b927fcb7/. No new live playback was started.

## In-band-first playback metadata, 2026-11-07

Playback now requests ICY on its actual audio GET. Positive valid icy-metaint
selects that connection exclusively, with a streaming deinterleaver feeding only
audio bytes to Symphonia and normalized metadata events to the session owner.
Request IDs and existing station-generation guards reject stale events. Audio
reconnects reset ICY framing/title state and re-evaluate capability. The separate
provider starts only after the audio response advertises no ICY, skips a redundant
probe of that audio URL and retains explicit hints, FLAC sibling search and JSON
fallback. Its independent cancellation token and serialized source-epoch gate
prevent a retired fallback from cancelling the session or replacing in-band
metadata. In-band success preserves configured fallback URLs. Passive ingress
ADTS timing now excludes ICY bytes even on framed audio responses.

All-target release tests passed 101/101 (78 library, ten console, 13 integration).
New coverage exercises byte-fragmented/empty/truncated ICY framing, exact audio
bytes after stripping, request counts, explicit hint precedence, no fallback for
empty advertised ICY, audio/metadata reconnect together, both capability-change
directions and fallback teardown. A self-generated 1,834-byte stereo-silence FLAC
fixture is decoded by the production decoder into the device-free sink while a
configured separate ICY source supplies titles. No public-radio playback or sound
device is used by these checks. Standard release Clippy completed with six
inherited warnings and no new ones; the previous too-many-arguments warning was
removed by making provider strategy selection an instance method. Formatting and
whitespace checks passed. UI/vendor files were not changed in this iteration.

The old diagnostic CMD audio process was stopped with q at the operator's request;
its final log reports zero audio, HTTP, metadata and session-owner workers.
Historical diagnostic EXEs are kept intact. Their independent observer remains a
forensic comparison tool, not the production in-band-first routing. This change
associates titles with the played session; it does not establish a universal live
edge or automatically repair a station session's multi-minute programme delay.

The no-default-features library suite also passed 54/54 (one inherited
irrefutable-pattern warning). The normal full GUI release was rebuilt and copied
to the project root: 18,969,088 bytes, SHA256
dfbe3705cd33c6266fc4e73b2eade57a187aee9ebf3867fb4934656fdea20dea,
matching target/release/ae-strom.exe. Its predecessor is backed up in ignored
local-data/icy-inband-delivery-20261107-0903d3fd/. The executable in F:/miniradio
was not replaced. Its real config passed the read-only offline compatibility
checker with zero network requests/audio and identical before/after SHA256;
no profile preparation, import or save was performed. The new project-root build
can explicitly use that existing profile with --data-dir F:/miniradio. No new
native GUI or public-radio playback session was started during delivery.

## Frequence3 FLAC container metadata discovery, 2026-11-07

After the operator observed immediate titles in MusicBee, bounded read-only
probes of the actual FLAC URL found that two DNS-advertised backends
(193.106.119.145 and 193.106.119.46) return audio/ogg with Ogg-FLAC identification,
no icy-metaint, and a Vorbis Comment packet containing TITLE=Anita Ward - Ring My
Bell. The vendor string is Icecast-Frequence3|Net-Radio|Keyyo. The third advertised
backend (213.246.45.50) returned HTTP 502 during these probes. Each successful
inspection consumed only 8,417 bytes; no audio playback was started. Raw header
and parsed metadata evidence is under ignored
local-data/flac-metadata-probe-20261107-1794073693/alternate-dns-servers.json.

Thus absence of ICY does NOT establish absence of same-connection track metadata
for this FLAC station. The pinned Symphonia Ogg-FLAC mapper already reads Vorbis
comments into MetadataLog, but Strom's engine does not yet consume that metadata
API. The just-delivered in-band-first change handles ICY, not container titles;
it still selects the external fallback on this Ogg-FLAC response. Supporting
container revisions and chained-stream changes before external fallback is a
separate pending correction. These probes establish available stream tags, not
the specific internal mechanism used by the closed-source MusicBee player.

## Container metadata before external fallback, 2026-11-07

Implemented the next priority: audio-connection ICY, then Symphonia probe/container
track tags, then the existing configured metadata provider and lower-bitrate
sibling discovery. The fallback decision now waits until container probing has
completed. ARTIST and TITLE are normalized into a track title; a full TITLE alone
also works. Revision cursors consume every new metadata revision, preserving
titles across artwork-only revisions without reusing an old artist on a new Ogg
link. ResetRequired processes the new metadata before rebuilding the decoder.
A later container title retires an already-running fallback; a new logical stream
without a usable title permits fallback again. Configured fallback hints are
preserved when container metadata succeeds.

Station names remain independent. icy-name is read regardless of icy-metaint.
Explicit container station fields and secondary-source names are applied with a
fill-empty policy checked by the controller against the CURRENT config. These
updates cannot replace an existing name or clobber an already displayed track
with a name-only event. Explicit station tags may supply a missing name even
when ICY remains authoritative for the track. ARTIST, TITLE, ALBUM, ORGANIZATION
and PUBLISHER are not guessed as station names. UI/config schemas are unchanged.

All-target release tests passed 110/110 (87 library, ten console, 13 integration).
New cases cover native FLAC comments, real demuxing/decoding of chained Ogg-FLAC
with title changes over one HTTP connection, ICY precedence over actual container
tags, late-tag fallback retirement, title loss enabling fallback, station-only
tags not suppressing track discovery, picture-only revisions, name persistence
and preservation of nonempty names/current titles. All audio is self-generated
silence delivered by loopback into the device-free sink. Standard release Clippy
completed with six inherited warnings and no new ones. Formatting and whitespace
checks passed. No browser/vendor source changed; their existing checks were not
repeated. No public-radio playback or native GUI acceptance was started.

The no-default-features library suite passed 55/55 (one inherited irrefutable
pattern warning). The full GUI release was rebuilt and copied to the project
root, matching target/release: 18,984,960 bytes, SHA256
a3cce18a0aa05b278c4905637646fb8977e715d8f005d1d8e257a7149de42453.
The previous in-band-only executable is backed up under ignored
local-data/container-tags-delivery-20261107-c09219fa/. The installed executable
and real config in F:/miniradio retain their previous hashes; no real profile
was loaded, imported or saved during this iteration. Historical diagnostic EXEs
remain unchanged. The new project-root executable may use the existing profile
explicitly with --data-dir F:/miniradio.

## Station names in Settings URL fields, 2026-11-07

Unfocused station fields show the saved station name, or the URL when unnamed.
Focus reveals the URL using the public TextInput focus/value APIs; blur restores
the label. Name/URL presentation swaps are silent and cannot create URL saves.
Only per-bank/slot URL drafts are committed, with matching in-flight commits
coalesced across Enter and blur. Failed/newer drafts remain available, and a
successful older reply clears its matching draft even when a newer metadata
snapshot has already superseded that reply's revision. The synthetic bridge now
matches Rust URL edits by clearing cached station name and metadata hints.

The full npm test pipeline passed: syntax, strict bridge types, pinned Taet
integrity, icons and 102/102 browser checks at 100% and 150% scale. Added checks
exercise mouse/keyboard focus, no-save display toggles, metadata updates while
editing, native URL selection preservation, actual URL-only saves, discovery
labels, delayed/stale replies across banks and rejected drafts. The Settings
screenshot was inspected; existing compact geometry, dragging, keyboard and
selection-paint checks also passed. UI-only changes did not modify backend or
vendor source; Rust suites were not repeated. The full GUI release was rebuilt.
No public-radio/audio-device or native GUI acceptance was started.

The rebuilt executable was copied to the project root and verified against
target/release: 18,985,472 bytes, SHA256
7dd413934da9c7f2dcf2c96e83495f5671ca3d186ce5c1821608090c2ee798df.
The predecessor and delivery record are under ignored
local-data/settings-station-names-20261107-6a335f9d/. The real config and installed
executable in F:/miniradio retain their prior hashes.

## CMD cleanup and source publication checks, 2026-11-07

Removed temporary icy-watch/timing/http/frequence3 CMD launchers, their root
audio executables and the shared console playback/timing code. Removed the older
tracked live-check source and Cargo target. The remaining bins are ae-strom and
icy_peek; the latter retains headers/raw blocks/titles, JSONL logs, notes,
fresh-session probes, reconnects and joined shutdown, with no audio engine in
its console path. Library timing counters and automated regressions remain.
Current tool instructions are in ICY_DIAGNOSTICS.md; historical entries above
describe tools that no longer ship.

Final checks passed:

- npm test: syntax for 36 JS files, strict public types, pinned vendor integrity,
  icons and 102/102 browser checks at 100%/150% scaling.
- cargo test --release --locked --all-targets: 102 tests (87 library, two
  icy_peek argument tests and 13 integration tests); examples compile.
- cargo test --release --locked --doc: 7/7.
- cargo test --release --locked --no-default-features --lib: 55/55, with the
  inherited irrefutable-pattern warning.
- cargo clippy --release --locked --all-targets: succeeds with five inherited
  warnings; no new lint warnings. Format and Git whitespace checks pass.

Pre-commit full runs exposed two test races. The station-only-container test
now checks one audio GET and JSON-only fallback requests instead of depending
on the poller's second request timing. The late-container-title fixture now
holds the second Ogg link until the test observes the fallback title, ensuring
it actually exercises fallback retirement. The focused late-tag check and the
subsequent full all-target run passed. No production behavior was changed to
make these checks pass. Rust logs are kept in ignored local-data/cleanup-*.log.

The rebuilt metadata-only icy_peek.exe was checked with --help, copied beside
its launcher and verified against target/release: 4,160,000 bytes, SHA256
013a4d7de882c049c107accaaa1d0ce74f67c78d7a2923e7eb82f8bbd19a21b2.
The previous inspector is retained under ignored
local-data/cmd-cleanup-7d3478e3cac4406c9761d468b885bb1c/. The user's running main
player was left running and its executable was not replaced. These checks used
synthetic loopback/fake playback; no public station or real audio device was
opened by this cleanup. Personal profiles, logs and binaries are excluded from
the source commits. CHANGELOG.md records the changes and implementation commits;
this source update does not create a new binary release.

## Windows release 0.1.1, 2026-11-07

The publication candidate advances Cargo/Tauri/npm package identity and the
matching lockfile root versions to 0.1.1. The existing v0.1.0 tag and assets are
preserved. Both EXEs were built with the reviewed release-cargo.js path-remapping
recipe from the v0.1.1 source inputs, into the isolated publication target.

The remapped all-target suite passed 102/102 and remapped doc tests passed 7/7;
formatting passed. The immediately preceding 102-browser, 55-no-audio and Clippy
checks cover the unchanged implementation/UI (five inherited Clippy warnings).
The version update introduces no new dependencies. Both executables pass the
static Windows x64/CodeView/personal-path/credential check and contain 0.1.1
version resources. The GUI binary has zero matches for station URLs read from
the local profiles; the inspector intentionally retains its public default URL.

- ae-strom.exe: 18,953,728 bytes, SHA256
  35cbf2f6309e09f98713a96e1268690c22ea36929a74bcbf73ecf15d907de755.
- icy_peek.exe: 4,147,200 bytes, SHA256
  1e3bb7e8ba3aef575fb2231deb5a435898a0c14770a2907317d5a2d8b572a7f8.

The release assets also include icy_peek.cmd, BUILD_INFO.json, SHA256SUMS.txt and
ae-strom-0.1.1-licenses.zip. The license archive is byte-identical to the previous
archive after checking that all locked dependencies and archived application
notices are unchanged. BUILD_INFO records the precise source commit and checks.
Ignored local-data/publication-0.1.1-*.log contains the build/test evidence;
local-data/publication-release-v0.1.1 contains the staged upload assets. No new
native GUI, physical input or live audio acceptance is claimed. The running
player, local installation and station profiles were not replaced or saved.

The all-target run relinked the publication executables after the initial build;
the hashes above record the final staged upload bytes, which passed the static
publication check again. PE timestamps/debug identifiers are not reproducible.


## Default WebView menus and Settings bank hotkeys, 2026-11-08

The Windows host turns off AreDefaultContextMenusEnabled and reads it back on
WebView2's owning UI thread before releasing the existing UI-ready barrier.
Initialization failures use the existing sanitized startup-failure path. Native
smoke evidence confirms defaultContextMenusEnabled=false. Keyboard editing and
clipboard handling are unchanged; there is no document-wide key interception.

Bank letter hotkeys also load the selected bank in Settings. This includes
selecting an already-active bank after browsing a different bank and selecting
while Settings is hidden. Settings bank buttons remain browse-only; typing,
composition and modifier guards in editors continue to apply.

The full npm pipeline passed syntax, types, vendor/icons and 104/104 Chromium
checks at 100/150% scaling. The editor shortcut regression now asserts that
Ctrl+X actually clears the field and Ctrl+V restores its exact URL, as well as
checking that Ctrl+C does not invoke ticker copying. Added bank regressions
cover hidden Settings, same-active-bank selection and typing in a URL editor.
The native fake smoke passed 36/36 with no audio device or OS-input automation;
its actual WebView2 readback is in ignored
local-data/native-smoke-20261108-174050-831/native-ui.json. The locked release
build, formatting and Git whitespace checks pass. Rust library/domain sources
were not changed, so their preceding suites were not repeated.

The rebuilt project-root EXE matches target/release and has a backup/delivery
record in ignored local-data/context-menu-bank-*/. The running player and its
profile under F:/miniradio were not replaced or stopped. These changes are local
and unreleased; the published v0.1.1 remains the preceding release.


## Playback intent through empty banks, 2026-11-08

The live controller keeps requested_playing=true when selection reaches a slot
without a stream URL. It retires the previous session, reports stopped/No stream
URL without an error, and starts the next configured selection automatically.
Explicit Stop while waiting clears that intent; selecting while stopped stays
silent. Play on an empty slot can arm playback for the next configured choice.
The existing terminal playback-error policy and synthetic Fake mode are unchanged.

Two new loopback/TestSink regressions passed, decoding synthetic PCM without an
audio device: configured A -> empty W/Z -> configured A/B resumes, empty slots
leave zero HTTP/metadata/audio workers, mute/volume survive, Stop while empty
prevents a later auto-start, and Play while empty arms it again. The full locked
offline all-target suite passed 104/104 (89 library, two CLI, 13 integration).
Formatting, Git whitespace checks and the final locked release build passed.
UI/host source from the preceding 104-browser/36-native check is unchanged in
this iteration; those checks were not repeated. Prior local fixes remain intact.

The delivery record and any replaced project-root EXE backup are under ignored
local-data/empty-bank-delivery-*/. The rebuilt candidate is available under
target/release/ae-strom.exe; project-root replacement is performed only when
that executable is not running. No live player or real profile was stopped or
changed, and no GitHub release was updated by this local fix.


## Right-anchored compact resizing, 2026-11-08

Native resizing now measures physical outer geometry and offsets x by the
actual width difference, preserving the right/top edge. Logical sizing remains
DPI-aware. Layout IPC operations share a host mutex; the frontend captures each
layout's compact flag/dimensions and serializes requests. setCompact exposes its
completion promise so the native smoke can await both transitions.

Eight positioning tests passed, including new compact/expanded roundtrips at
100/150/200% DPI, negative monitor coordinates, inward expansion after moving
compact to a monitor's right edge, and overflow bounds. The release build,
formatting and Git whitespace checks passed. The full npm pipeline passed
106/106 browser checks plus syntax/types/vendor/icons; the added delayed-IPC
regression confirms rapid toggles execute in order with their original sizes.

Native fake smoke passed 36/36 and the strengthened PowerShell geometry gate
confirmed identical physical right/top edges for every recorded resize, with
both real width-decrease and width-increase transitions present. Evidence is in
ignored local-data/native-smoke-20261108-180835-319/native-ui.json. No OS drag or
physical mixed-monitor/DPI transition was automated; those coordinate cases
were covered by pure tests. No real audio or personal-profile writes occurred.

The delivery record and replaced project-root EXE backup are under ignored
local-data/right-anchor-delivery-*/; root replacement is skipped if it is
running. The installed/running player is left alone. Earlier menu, bank and
empty-slot playback-intent fixes remain in this local candidate. This iteration
does not change the published GitHub release.


## Settings/EQ opening direction, 2026-11-08

The host selects the monitor at the player-strip center and uses its work area,
including taskbar exclusions. It preserves that strip's y and the physical right
edge while sizing Settings/EQ above or below. Its layout reply now supplies
panelsAbove; the UI reorders the same controls with app CSS. Movement persistence
stores the strip position even when the expanded window begins above it.

Nine positioning tests passed, including lower work-area boundaries at multiple
DPI, negative monitor origins, Settings/EQ heights, collapse restoration and a
constrained-height case. The locked release build, formatting and Git whitespace
checks pass. The full npm pipeline passed syntax/types/vendor/icons and 108/108
Chromium checks at 100/150% scaling, including upward panel order and expand/
collapse recovery with persistent controls.

Two native fake runs passed 38/38 each: the ordinary smoke and a fresh synthetic
profile placed with its strip at the work area's bottom-right edge. The second
run confirmed Settings and EQ really rendered above the strip, expanded windows
fit inside the native work area, all recorded strip/right-edge anchors stayed
fixed and saved x/y matched the strip's startup coordinates. Evidence is under
ignored local-data/native-smoke-20261108-183037-158/ and
local-data/panels-bottom-6e12c391da4849f09dd8fbc439f84c44/. Neither run opened a
real audio device or read/wrote a personal player profile.

The candidate is target/release/ae-strom.exe; the project-root copy is replaced
only when not running, with the predecessor and hashes retained under ignored
local-data/panels-direction-delivery-*/. No running player was stopped. Earlier
local fixes remain included; GitHub publication is unchanged.


## Intermediate upward-panel jitter, 2026-11-08

Earlier endpoint checks missed the visual transition and used outer height as
panel height, inadvertently including a non-client Windows frame. The current
calculation separates client panel pixels from the collapsed frame (the native
fixture exposed a 13px vertical difference). A read-only layout preparation
selects direction before commit. Native upward strips remain attached to the
viewport bottom during opening/closing IPC, and visible Windows position/size
change in one UI-thread SetWindowPos. Hidden startup keeps Tauri's sizing path.

Nine geometry tests pass, with an added non-client-frame case. The locked offline
release build, formatting and Git whitespace checks pass. Full npm checks pass
syntax/types/vendor/icons and 110/110 browser checks at 100/150% scale. The new
regression holds native resize replies pending and samples frames during both
opening and closing, ensuring the strip remains on the old viewport edge until
the viewport actually changes.

Final normal and bottom-edge native fake runs pass 39/39 each. The bottom run
has 20 intermediate upward samples with the strip at the client viewport edge;
all physical strip/right anchors match, visible bounds are atomic and persisted
strip y is unchanged. Evidence is in ignored
local-data/native-smoke-20261108-195800-029/ and
local-data/panel-jitter-bottom-5088b170e8f547afaeb1246dfb0df4d9/.
An initial native run exposed a Stop assertion racing its state event; the
harness now awaits the returned snapshot revision rather than assuming one RAF
has delivered IPC. This does not alter playback behavior.

The final candidate/root-copy status, checksum and previous-EXE backup are in
ignored local-data/panel-jitter-delivery-*/. No running player or real profile
was stopped or modified. No new registry packages were added: Windows 0.62.2
was already in Cargo.lock and is now an explicit target-specific dependency.
The public GitHub release remains unchanged by this local fix. These are
programmatic frame/geometry checks, not a human-reviewed desktop video.


## Horizontal expansion constrained to the work area, 2026-11-08

The compact player now expands left while preserving its right edge only when
that placement fits. At the left work-area edge it instead preserves its left
edge and expands right. If neither edge-preserving placement fits, x is clamped
so the requested width remains inside the current monitor work area. The
existing atomic resize and vertical strip/client-frame handling remain intact.

Eleven geometry tests passed. New cases cover compact-at-left/right, interior
right-anchor preference, collapse behavior at 100/150/200% DPI, actual 150% native
frame widths, negative monitor origins, inset taskbar work areas, neither-side
fallback, out-of-range coordinates and a best-effort oversized-window case.
The release build, formatting and Git whitespace checks passed.

Native fake smoke passed 39/39 plus selected-edge/work-area assertions for each
recorded resize: ignored
local-data/native-smoke-20261108-200706-875/native-ui.json. The exact moved-compact
left-edge coordinates are covered by pure geometry tests; no physical OS drag
was automated. Web UI/IPC code is unchanged in this iteration, so the preceding
110-browser check was not repeated. No real audio or personal profile was used.

The candidate/root-copy result, hash and backup are in ignored
local-data/horizontal-fit-delivery-*/. Replacement is skipped for a running EXE.
Earlier local fixes remain included; the running player and published GitHub
release were not changed.


## Startup retry diagnosis, 2026-11-08

The operator clarified that the message can occur while a station cannot start,
not only after interruption. Inspection found that retry setup unconditionally
replaced HTTP/codec/device causes with a generic stream/output-interrupted text;
the CPAL callback also discarded its concrete error. Current code retains the
cause in the ticker and warning output, including HTTP/probe source chains,
output creation/start/callback errors and decoder/read/reset errors. Retry
intervals, connection/read timeouts and playback routing remain unchanged.

A loopback HTTP 502 regression confirms a temporary startup failure reports 502
and retry status with Play intent retained; selecting a working synthetic PCM
source starts playback and clears the stale cause. The focused check and full
locked offline all-target suite passed: 110 tests (95 library, two CLI and 13
integration). Formatting, Git whitespace and the final locked build pass.
UI assets are unchanged, so the preceding 110-browser checks were not repeated.

Read-only public-stream diagnostics used no output device: A-9 Deutschrock's
native icy_peek probe received HTTP 200, about 306 KB and two ICY blocks over
12 seconds without an HTTP error. The particular failed GUI startup was not
captured, so this does not identify its cause or prove decode/output success.
Evidence: ignored local-data/a9-start-probe-86d10e1d01754c898af73e0839ffa067/.

FREQUENCE 3 FLAC resolved to three addresses in a direct bounded HTTP probe:
213.246.45.50 returned 502; 193.106.119.46 and 193.106.119.145 returned 200,
audio/ogg, and OggS bytes. A separate native Rust-transport probe got two 200
responses from 193.106.119.46 and then a request TimedOut before response headers.
Those network failures precede any audio-device/decoder setup. Evidence is under
ignored local-data/a4-start-probe-0ee723e6e96b4b1eaf352ea40baa233b/. The FLAC
inspector's expected no-ICY result is not an audio failure. These probes do not
establish that every reported GUI failure has the same cause.

The local candidate, checksum and predecessor backup are recorded under ignored
local-data/retry-cause-delivery-*/. Root replacement is skipped when running.
No live player was stopped, no private profile was edited and no GitHub release
was updated. DNS/server failover behavior was not changed by this diagnostics fix.


## Hover hints and strict certificate validity errors, 2026-11-08

Removed native title attributes from controls and ticker updates, stopped
setting icon-button tooltip text, and disabled EQ hover/focus/drag value popups.
ARIA names, status/error descriptions, slider values, URL editing and keyboard
clipboard behavior are retained. No pinned Tæt vendor assets were modified.
Browser coverage exercises panel switches, URL focus/blur, bank changes, dynamic
Play/Mute/Expand labels, track/error updates, and EQ pointer/keyboard interaction.

HTTPS certificate verification remains enabled, with no slot exception and no
automatic downgrade to HTTP. The Windows CERT_E_EXPIRED numeric error becomes
the terminal English message "Server certificate has expired or is not yet
valid"; original details remain in logs. Numeric recognition does not depend
on localized Windows prose. Microsoft documents the code as a validity-period
failure: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wcce/45119343-8b87-412d-b19a-d06c51edb28e .

A read-only public TLS probe of jfm1.hostingradio.ru:14536 failed normal
verification as expired. Inspection of the public certificate (without sending
an HTTP request) showed a SAN covering *.hostingradio.ru and an expiry of
2026-10-23 17:43:25 UTC. The local UTC date was 2026-11-08. The certificate-only
inspection used an unverified handshake to read the rejected certificate; this
was not applied to the player or its transport. No output device was opened.

The locked offline all-target Rust suite passed 111 tests: 96 library, two CLI,
and 13 integration tests. The added test covers the real OS-code formatting,
localized/hex representations, terminal classification, the exact English
message, and preservation of unrelated transport causes. The release build
passed. Native fake smoke passed 40/40 plus work-area/anchor assertions:
ignored local-data/native-smoke-20261108-223958-889/native-ui.json. Native
evidence uses programmatic focus and fake playback, not OS input or live audio.

The full npm suite passed: syntax (36 files), strict types, pinned-vendor and
icon integrity, and 112 browser tests at 100/150% scale. Updated local EXE delivery
and predecessor backup are recorded in ignored local-data/hints-certificate-delivery-*/;
replacement is skipped if that EXE is running. No live player was stopped,
no personal profile was edited, and no commit/push or GitHub release update
was performed in this iteration.


## Windows release 0.1.2, 2026-11-08

Release identity is 0.1.2 in Cargo, Tauri, npm and both lockfile roots. Commit
40ec49b contains the accumulated playback, window placement, Settings, menus,
hints and diagnostic fixes; the release commit adds version/history updates
and synchronizes the native smoke Stop check with the UI command's own reply.

The remapped publication all-target suite passed 111 tests and doc tests 7/7.
Clippy passed with the same five inherited warnings. The 112-browser suite at
100/150% scale covers the unchanged production UI; since that full run only
the native test harness and package version changed. Syntax (36 files), types,
pinned vendor/icons and formatting checks passed after the version update.

Two initial publication smoke attempts failed the idle assertion: the old
harness sent Stop through the UI and again directly to the backend, observing
a different reply from the UI action clearing the injected error. The final
harness captures and awaits the single UI Stop, then checks rendered state
with a bounded deadline and records the observed ticker/snapshot on failure.
Two consecutive runs of the final staged bytes passed 40/40 plus native
work-area and selected-edge assertions, using fake playback and no audio.
Evidence:
- local-data/native-smoke-publication-0.1.2-dc2217440f0444a0869bffa2eef4f9be/native-ui.json
- local-data/native-smoke-publication-0.1.2-9b47817390ba438d86f57b61779e5b78/native-ui.json

Both final Windows x64 EXEs pass the static personal-path, credential-pattern
and CodeView/PDB checks and contain 0.1.2 product-version resources:
- ae-strom.exe: 19038208 bytes; SHA256 a36c5d06567a13dba542cc1ed6510535eba5a9bfb632991629a833f1e1d5135e.
- icy_peek.exe: 4155392 bytes; SHA256 d7ac5927bb7b15f87f8c32f65e885ebbc644158d7ae36a807eb9b0c74f1d2e53.

The license archive is reused after all 536 non-root locked package entries
and five application notices matched the prior release. The added direct
Windows 0.62.2 dependency was already in the archived Windows graph (359
components). No third-party package version or pinned framework changed.

Staged files are in ignored local-data/publication-release-v0.1.2. BUILD_INFO
records the exact release source commit; SHA256SUMS covers both EXEs, CMD
wrapper, license archive and build information. Earlier release assets remain
untouched. The running player and personal profiles were not replaced or
opened; physical OS-input, audio-device and macOS acceptance are not claimed.

## Tæt UI rc.7 consumer upgrade, 2026-11-10

The app now pins published taet-ui 0.6.0-rc.7. The npm tarball SHA-512 matches
the lockfile and its SHA-256 is recorded in provenance/taet-ui-rc7-source.json.
All 306 selected source/package/license files match clean upstream commit
53c5b177a5b68f1c1779afe97316480c65101840, allowing only Windows text line endings
when comparing to the checkout; provenance retains exact published byte hashes.
One foundation/components bundle, six runtime assets and 85 declarations were
regenerated and verified together. A second canonical regeneration passed.
The old rc.4 provenance remains historical, not an active build input.

The production app/controller code is unchanged. Built-in text/select editors
remain compatible with the new strict revision tracking; the two custom
BaseControl extensions are displays, not independent validation editors.
The ticker now receives the upstream font-completion remeasurement fix.
A new synthetic regression delays the actual bundled font response, checks the
final ticker animation distance/duration, and destroys the UI before later font
notifications. Focused branding/font tests passed 8/8 at 100/150% scale.

Rust formatting passed. The isolated, locked, offline publication all-target
suite passed 111 tests; documentation tests passed 7/7. Publication clippy passed
with the same five inherited warnings. The release candidate built successfully
and passed static personal-path, credential-pattern and CodeView/PDB checks:
- local-data/publication-target/release/ae-strom.exe
- 19049472 bytes
- SHA256 fa2978a7d8d27c12cf010b93269e01be55cc07082ab41fae6cb104e79e035e1e

The smoke harness now accepts -ExecutablePath so this exact staged executable
can be checked without copying over a running or installed player. Two launches
passed 40/40 plus compact/expand work-area and selected-edge geometry assertions:
- local-data/native-smoke-20261110-151624-641/native-ui.json
- local-data/native-smoke-20261110-152003-570/native-ui.json
Both used separate synthetic profiles, no audio device and no physical OS input.
The root player and personal profiles were not replaced or opened.

The first browser pass was 110/112: its only failures were the two stale rc.4
version-string expectations. These were updated to rc.7 without weakening the
shared BaseControl identity or other behavioral assertions. An overlapping
aggregate retry stopped before browser execution because the first run still
owned port 5186; the final run is sequential. A temporary focused config first
needed its web-server cwd corrected; its completed run passed as noted above.
Exact published vendor declaration bytes include CRLF, so ordinary git diff
--check labels those CR bytes as trailing whitespace. With the command-local
core.whitespace=cr-at-eol interpretation, diff --check passes; no repository
configuration or canonical vendor bytes were changed to silence that warning.

This entry describes local upgrade verification, not a published release.
Application version is still 0.1.2 pending a separate release-version update.
Native IME, real audio/device behavior, physical input and macOS acceptance are
not claimed by these synthetic checks.

Final sequential npm test completed successfully: syntax (36 files), strict
TypeScript, canonical vendor integrity, icon integrity and 114/114 Chromium
browser tests at 100/150% scale (5.7 minutes). This includes the updated branding
pin, both delayed-font/teardown cases, existing Settings draft/currentness,
keyboard/editor guards, EQ, compact layout and Spectrum lifecycle regressions.

## Windows release 0.1.3 preparation, 2026-11-10

Cargo, Tauri, npm and both lockfile roots are now version 0.1.3. The production
UI is byte-for-byte unchanged from the passing 114-test rc.7 browser suite above;
only release metadata and documentation changed afterward. Syntax (36 files),
strict types, vendor/icon integrity and Rust formatting passed again on 0.1.3.
The remapped, locked, offline all-target Rust suite passed 111 tests and doc tests
7/7 again. Clippy passed with the same five inherited warnings.

Both final staged Windows x64 executables contain product version 0.1.3 and pass
static personal-path, credential-pattern and CodeView/PDB guards:
- ae-strom.exe: 19049472 bytes; SHA256 836df1e1dfb636b61af9845884475f4e62a5b208d5593a9d33a64df5bd4b1024.
- icy_peek.exe: 4155392 bytes; SHA256 9b5c01de750aa119907ddc7800c4de322f77502ae680c5c464a8beace1b1c9cc.

All 536 non-root Cargo lock entries are unchanged from 0.1.2. All five archived
application/framework/font license notices match the current files byte-for-byte;
the 359-component Rust license inventory is preserved. The 0.1.3 archive updates
only the top-level application-version heading in its notices.

Release artifacts are staged separately in local-data/publication-release-v0.1.3.
Existing release assets and the installed/running root player remain untouched.
BUILD_INFO.json and SHA256SUMS.txt identify the final source commit and artifacts.
This preparation does not itself publish a tag, commit or GitHub release remotely.

The final staged 0.1.3 player passed two native fake-backend launches, each 40/40
plus work-area, selected-edge and strip-anchor geometry assertions:
- local-data/native-smoke-20261110-154909-838/native-ui.json
- local-data/native-smoke-20261110-154932-515/native-ui.json
No live audio or physical OS-input testing is implied.


## macOS development port, 2026-11-11

See [MACOS.md](MACOS.md) for the Apple Silicon build, isolated native WKWebView matrix, Chromium/WebKit checks, tested executable hash and remaining manual/audio limitations. This is local development evidence, not a published Mac release or Windows re-certification.
