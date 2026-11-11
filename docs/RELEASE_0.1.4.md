# 0.1.4 cross-platform candidate

This is unpublished local preparation, not a GitHub release or a certification of
all macOS versions. Keep `v0.1.3` and its assets unchanged. A final tag must point
to the exact clean source commit used for both release builds. Do not relabel
old Windows 0.1.3 binaries as 0.1.4.

## Reproducible preparation

Both platforms use Rust 1.96.0 with Cargo.lock. Windows uses Node 22.14.0;
the local Mac currently uses Node 22.23.4. Tæt UI remains pinned to rc.7.

- Windows: `node scripts/release-cargo.js build --release --locked --bins`,
  then `node scripts/check-release.js` for each executable.
- Mac: `npm run build:mac:release`, which runs the pinned Tauri CLI with the
  Apple Silicon target, isolated output directory, locked dependencies,
  macOS deployment target 11.0, and explicit `--no-sign`.
- Shared path remapping removes checkout/home/Cargo/sysroot paths from Rust
  diagnostic strings. Static guards scan bundle files for private paths and
  credential-shaped data, validate Mach-O architecture, Info.plist identity,
  executable mode and system-only dynamic libraries. These are targeted
  safeguards, not a claim that arbitrary unknown secrets can be detected.
- `python3 scripts/source-snapshot.py <new output.zip>` exports tracked and
  nonignored source files, excluding deleted files and personal/build data.
  CANDIDATE_SOURCE.json records exact file hashes and a canonical manifest hash.
  This makes uncommitted local candidate verification traceable, but does not
  substitute for a final release commit or tag.
- `npm run test:release` tests the static guards. Both CI workflows run checks
  only; they contain no signing credentials or release upload step.

## Verification gate

Run full npm/Rust checks and native fake-profile matrix on the exact candidate
on each real platform. A green CI run for v0.1.3 does not certify this candidate.
Recheck the final bundle after packaging, including extraction and native launch.
Record the source manifest/commit, toolchain, architecture, OS, artifact SHA256,
verification scope and signature status in platform-specific BUILD_INFO files.
Do not include local profiles, caches, evidence, host paths or logs in downloads.

## Proposed assets, after a separate publication decision

- ae-strom-0.1.4-windows-x64.exe
- ae-strom-0.1.4-macos-arm64.zip containing the complete Æ Strøm.app
- Windows inspector and launcher, if retained
- Platform-specific licenses and BUILD_INFO JSON, plus SHA256SUMS.txt

The Mac is arm64 only. Intel/Universal and older macOS are not tested.
The declared macOS 11.0 floor is a build setting, not verified compatibility.
The unsigned candidate has only the linker's ad-hoc executable signature.
It is not Developer ID signed or notarized; distributing it as an unsigned
preview requires an explicit decision and Gatekeeper disclosure. Do not change
system security settings as part of installation. A regular signed release
requires a separate Developer ID signing/notarization/stapling workflow.

App-menu Quit/Cmd+Q and player Close await saved settings and allow retry after a
save failure. Repeated close requests coalesce. Dock Quit/OS termination are
best-effort only: the pinned Tao delegate cannot cancel termination after a save
error. Keep this limitation in release notes until native lifecycle support is
implemented and tested.

See MACOS.md for earlier native/audio evidence and THIRD_PARTY.md for license
coverage and explicitly labelled standard-license supplements. Publication remains blocked until
both-platform checks, exact-source rebuild, attribution review and the signing or
unsigned-preview decision are complete.
