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
