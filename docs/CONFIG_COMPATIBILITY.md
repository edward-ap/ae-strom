# Configuration compatibility diagnostics

These changes build on v0.1.0; they do not constitute a new binary release. A report containing only CONFIG-VALUES does not identify which value failed; it is not proof of a particular EQ or URL problem.

## Safe diagnostics

Startup and the offline checker report a known field path and a static reason, for example `stationBanks[1].slots[0].eq`. Array indices are zero-based. Sparse-bank errors refer to the original document arrays, not flattened padding. Values, station/preset names, URLs, credentials, unexpected field names and raw serde/OS messages are not echoed. JSON syntax/duplicate errors include line/column instead of an input excerpt.

## Compared with the Slint loader

The old loader deserialized values and normalized banks, but performed no equivalent semantic validation. Acceptance by that loader alone does not establish a recoverable EQ definition or safe gain value.

| Area | Current policy / observed difference |
| --- | --- |
| Layout | Exactly one of `presets` or `stationBanks`; unknown/duplicate fields fail closed. Slint ignored unknown fields and could choose/skip ambiguous entries. |
| Station objects | Known old fields `name`, `url`, `eq`, `metadataType`, `metadataUrl` remain required strings; no invented values. |
| Bank/slot | One A-Z letter (case accepted), keys 1-9/0; duplicates reject instead of old last-write-wins. Missing `slots` remains an error: although old deserialization had a default, the old serializer always wrote it. A missing populated array cannot safely be inferred empty. Explicit `slots: []` works. |
| Counts/selection | At most 260 slots. Existing last-bank/last-slot clamping and minimum ten-slot padding remain the legacy normalization; no extra station is truncated. |
| Volume/types | Volume 0-100; integer/boolean/string and storage-width requirements are reported at the field. No numeric coercion. |
| Existing custom EQ names | Long UTF-8 names and surrounding spaces are now preserved exactly, including bindings and curves. New editor-created names retain the 128-byte/nonreserved/trimmed creation rule. Blank/control names and ASCII case-insensitive collisions still reject. |
| Reserved built-in EQ names | Existing deterministic legacy renaming retains curves and exact old bindings; this pre-existing migration behavior is unchanged and original input remains recoverable through the existing backup/import contract. |
| Unknown EQ reference | Rejected at the station's `eq`; no automatic Off curve, guessed definition or deleted assignment. Slint could resolve an unknown name as Off. A real missing curve requires the owner's explicit decision or original definition. |
| EQ gains | Ten numeric bands, finite gains/preamp within -15..15 dB. Slint loading did not enforce the range; arbitrary old values are not silently clipped or expanded into new DSP semantics. |
| Stream/metadata URL | Empty is allowed; otherwise HTTP(S), host, at most 16384 bytes, no control characters. Valid literal path/query spaces and Unicode are now accepted through URL parsing and retained; HTTP clients encode them when making a request. Host spaces remain invalid. Existing outer URL trimming remains unchanged. |
| Metadata hint | Exactly empty, ICY or JSON (case-sensitive). Unknown hints are reported without discarding them. Slint could ignore unknown hints and rediscover metadata. |
| Other state | `currentUrl` and `apiEndpoint` remain strings without added semantic URL checks. Optional `lastBank` and nullable/optional `jsonPollIntervalSec` keep their old defaults; polling still applies the runtime minimum. Geometry uses the existing integer types and separate monitor safety logic. |

The compatibility changes deliberately preserve data, not guessed intent. Case-colliding custom curves, missing EQ definitions, unsupported gains or metadata strategies may need a separate decision after inspecting the actual diagnostic.

## Read-only offline check

```text
cargo build --release --locked --example config-check --target-dir local-data/config-check-build
local-data/config-check-build/release/examples/config-check.exe "<absolute path to config-copy.json>"
```

The checker reads and decodes the explicitly named file, confirms it did not change during the check, and prints only aggregate counts or sanitized field/reason diagnostics. It does not call profile preparation, save, import or migration; creates no config/output beside the source; opens no window; makes no network request; starts no audio. It is distinct from `profile-verify`, which writes a roundtrip file beside an explicitly selected audit copy.

Have the owner copy the failing config and run this checker privately. Share the Field/Reason output first; do not post the config or URLs in a public issue. If a file is needed, transfer a copy privately with permission and leave the original intact. Editing/redacting URL syntax or EQ names can remove the very failure being diagnosed, so a successful check of a redacted file is not evidence about the original.

Synthetic regressions cover long/padded Unicode EQ identity, legacy path/query spaces, sparse-field locations, unknown EQ/collisions, metadata and gain errors, structural types, duplicate/malformed JSON, source preservation and save/load roundtrip. Real failing third-party data, native OS dialog appearance and live playback are not claimed tested by these regressions.
