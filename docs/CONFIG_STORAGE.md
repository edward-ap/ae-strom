# Configuration storage and migration

Normal double-click startup is live-capable but stopped. It never starts a station automatically.

## Locations

- `ae-strom.exe` and `config.json` are portable application/settings files.
- Windows WebView2 state: `%LOCALAPPDATA%/local.edwardap.ae-strom/webview/profile-<id>/EBWebView/`.
- Profile locks: the same AppData application directory, under `profile-locks`.
- Migration receipts: that AppData application directory, under `migrations`. Receipts contain paths, not station values. They reference exact-byte verified config backups.
- `config.json.pre-migration-<unique-id>.bak` preserves source bytes before a startup format/legacy migration. Keep these backups until the new profile is accepted.

The profile ID deterministically separates absolute profile directories; it is not a security hash. WebView2 owns the browser directory's internal names and may create cache/component/profile files even for an offline UI. Existing `local-data/player/webview` data is left untouched. It is not the station database.

On macOS/Linux the Tauri platform app-local-data resolver is used, but this change has only been executed on Windows. Do not infer identical WebView engine layout or writable macOS app-bundle portability.

## Selection rules

| Situation | Startup behavior |
| --- | --- |
| No adjacent config and no previous profile | Create a validated empty config, then open stopped. |
| Only adjacent config | Validate, normalize compatible legacy values, back up before any needed rewrite. |
| Only previous `local-data/player/config.json` | Copy through validation into adjacent config. Preserve old file and browser directory. Keep a verified backup and receipt. |
| Both configs normalize to the same values | Use adjacent config; acknowledge the unchanged previous file with a verified backup. |
| Both differ, no matching receipt | Stop with conflict error; write neither config. |
| Receipt matches unchanged previous file | Use adjacent config, including subsequent station/settings edits. |
| Previous file changes after migration, both now differ | Stop with conflict error; preserve both. |
| `--data-dir <absolute-directory>` | Use only this directory's config. Ignore EXE-adjacent/previous candidates. WebView state stays in separate AppData. |
| Explicit import | Read-only source; create a new destination, or leave an equivalent existing destination byte-for-byte unchanged. Refuse a different destination. |

All settings participate in conflict comparison, including current selection, volume, mute and geometry. This avoids choosing a winner by modification time or silently discarding newer settings. Trailing wholly empty station slots are ignored by canonical sparse-bank comparison.

## Existing installation transition

Close the old player yourself before updating. Keep the prior EXE, adjacent config, previous profile and backups recoverable. Replace only the EXE with the complete new binary.

If the folder contains two different configs, choose deliberately:

1. To keep the previous Dense/Tauri profile, preserve/rename the adjacent `config.json` first. Default startup then migrates `local-data/player/config.json` without changing that source.
2. To keep the adjacent Slint config, launch once with `--data-dir "<EXE directory>"`. This explicitly selects it and applies compatible normalization with a verified backup. Retain the other profile. Continue using the explicit directory while the other conflicting source remains, or preserve/move the unused previous config after verifying the chosen result.
3. To compare without changing either, copy one config into a fresh directory and use `--data-dir` for that copy. Import into a fresh directory with `--data-dir "<new profile>" --import-config "<source config>"`.

Do not merge two different station collections or overwrite a config based only on timestamps. No automatic union is performed.

## Compatibility and write guarantees

Both `presets` and sparse `stationBanks` layouts are supported. Exactly one station-storage field is required. Unknown fields, misspellings, duplicate JSON keys, invalid nested fields, duplicate bank/slot identities and malformed required data are rejected before any save.

Legacy URL whitespace is trimmed only at the edges. Custom EQ names colliding with built-ins are renamed without losing values; old exact built-in references retain their original meaning. Unknown EQ references and malformed URLs still require correction instead of silently becoming `Off`.

`load_or_default` performs in-memory normalization only; reading a file never rewrites it. Startup separately verifies compatibility and writable storage, creates and reads back a unique backup, checks that the destination did not change, atomically saves, and verifies the saved config by loading it again. Existing canonical JSON with different whitespace/key order is not rewritten merely for formatting. Normal UI persistence can later serialize current settings in the canonical layout.

The new app holds an exclusive per-profile OS file lock in AppData for its lifetime. Older binaries do not participate in that protocol: close them before migration and avoid external concurrent config editing. A read-only/unwritable config, inaccessible AppData or failed backup produces a startup error; no alternate profile is substituted.

## Recovery and cache

To restore, close the player, preserve the current config, and copy the desired verified `.bak` to `config.json`. Explicitly select that directory if a conflicting previous profile remains. If a receipt or its referenced backup is lost/corrupt, conflict checks fail closed; choose a source explicitly.

Keep migration receipts while retaining a different historical previous profile. Browser caches can be recreated in a new session, but never delete an active WebView2 user data folder. Removing cache directories does not remove the Rust config. No automatic deletion of old browser data or historical profiles is performed by this change.

## Verification status, 2026-11-04

The source implementation passes 54 Rust regressions, seven doc tests, formatting and standard Clippy. Seven pre-existing Clippy warnings remain in inherited code; warnings-as-errors is not a passing gate. The final source snapshot also passes browser syntax/types/pinned vendor integrity and all 76 browser checks at 100/150% scaling.

Real legacy data was exercised only on isolated copies, with verified original backups, complete private station/EQ comparisons and unchanged original hashes. Tests cover read-only decode versus explicit format rewrite, missing/unknown/duplicate fields, normalized legacy values, conflicts, repeat launch/save, receipt/backup tampering, profile locks and filesystem write failures. No system ACLs were changed.

The first real Windows WebView2 fake run opened the AppData browser directory and passed 35 of 36 checks. Its existing error-rendering assertion failed without identifying which subcondition failed. The test harness was changed to await the actual error DOM mutation and record each geometry/clipping/text condition; production UI behavior was not changed. A retry via ordinary PowerShell was denied by system script Execution Policy before the script ran. The corrected native run, standalone migration/restart and native startup matrix are therefore pending, not passed. Persistent execution policy was not changed; no further bypass was attempted after that denial.

The full build is delivered as the root `ae-strom.exe` and under `target/release/ae-strom.exe`, with a verified previous EXE retained in ignored `local-data/config-storage-delivery-rollback-*`. No working profile was migrated by the tests or delivery. The agent did not launch live radio or automate physical OS input. macOS execution, compositor first-frame capture and real ACL-denial experiments remain unverified. User-reported manual results are distinguished below.

## Startup notification correction

The user confirmed clean startup and migration, but conflict refusal showed no visible explanation. Tauri 2.12.1 executes the setup hook during its Ready event; an Err escaping setup is turned into a runtime panic instead of returning to the application's outer Result handler. That outer handler's Windows MessageBox was therefore skipped.

All application setup errors are now handled inside that deferred boundary. A sanitized Windows MessageBox is requested before controlled exit; setup returns Ok so Tauri does not replace the failure with its own panic. The same notifier handles argument/path/runtime errors outside setup. The dialog has an error icon, foreground/topmost flags and a stable error code. Raw serde values, station/config contents, private paths and underlying error strings are not interpolated into the notification or stderr. A failed Win32 dialog call logs only a safe error code.

New tests cover notification-before-exit for every error category, successful setup with no notification, secret sentinel redaction, and actual synthetic conflict/JSON/read-only/profile-lock/AppData failures reaching the notification boundary without changing files. Total: 54 passing Rust tests; seven doc tests and standard Clippy pass, with the same seven inherited warnings. Frontend syntax/types/vendor checks pass; the UI assets are unchanged from the last 76 passing browser checks.

The new candidate and fresh manual error fixtures are recorded in ignored `local-data/startup-error-verification.json`. The user subsequently confirmed seeing the corrected conflict startup warning. This confirms visible notification; an exact dialog code, exit status and the other error-dialog categories were not explicitly reported. No blocked PowerShell script was rerun and no execution policy was changed. The project root EXE was promoted only after that confirmation and final safe checks; the real installation EXE and all real configs remain untouched. Merely failing to open a player window is explicitly not an error-notification pass.

## Manual acceptance and delivery

| Case | Evidence |
| --- | --- |
| Clean startup | User-reported pass on the configuration-layout candidate. |
| Previous-profile migration/restart | User-reported pass on the configuration-layout candidate. |
| Conflict refusal | Both synthetic input files still match their manifest hashes. |
| Visible conflict warning | User-reported pass after the deferred setup notification fix. |
| Other startup error dialogs | Unit-tested routing/privacy; physical dialog checks not reported. |
| Complete native fake script / native placement matrix | Still not rerun because PowerShell script execution is blocked. |

The final rebuilt executable is 18,642,944 bytes, SHA256 `1c6132b7d33da8063123f494a26006247b9c5adeb7e942e9cad0a5ef4467517d`. The manually checked notification candidate had SHA256 `13f3c44288fd0320d1153c86dc7a9f65a6b859822bf718a458f8ed32a4d1fcde`. A byte-level PE comparison found exactly 24 differing bytes, confined to COFF/debug timestamps and the RSDS PDB GUID; every other byte, including code and embedded resources, matches. Whole-EXE bitwise reproducibility is therefore not claimed.

See [installation and explicit profile selection](INSTALL.md). Delivery did not launch the application, change a real installation config, choose between two existing profiles, or clear old WebView data. Immediate binary rollback is available using the saved previous EXE; the unchanged previous profile is still retained. Keep new settings as well if rolling back after further use.
