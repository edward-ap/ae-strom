# Æ Strøm application icons

`AE-Strom.png` is the user's unmodified original: 1254×1254 RGB, with an opaque black exterior. `source.png` is the reviewed 1254×1254 RGBA master with transparent exterior corners. Its navy tile, antenna/waves and luminous blue rim are retained. Both source hashes and the exact edit prompt are recorded in [source-provenance.json](source-provenance.json).

The transparent master was made with built-in image_gen, following the same background-extraction approach as the companion Reader. The tool edit is not claimed pixel-identical to the original; the original artwork is preserved separately. Visual inspection covered white/gray/black backgrounds at small sizes. Building or regenerating formats uses the committed master and needs no image service or API key.

## Reproduce desktop formats

```text
npm ci --include=dev --ignore-scripts
npm run icons
npm run test:icons
```

The generator is pinned to **@tauri-apps/cli 2.11.4**, the Reader's tool version. Its CLI renders into ignored `.gen`, then the app script publishes desktop files only. Mobile/Appx staging scaffolds are not shipped because those targets are not configured. Source/master hashes must match the reviewed provenance pair before generation.

Published assets:

- PNG: 16, 24, 32, 48, 64, 128, 256 (`128x128@2x.png`) and 512 (`icon.png`).
- Windows ICO: six PNG frames, 16/24/32/48/64/256 px.
- ICNS: PNG retina/non-retina representations through 1024 px, plus legacy 16/32 RGB/alpha-mask pairs.
- `web/app-icon.png`: the same 32 px PNG, linked as the offline preview favicon.

After resize, alpha values 0–1 are normalized to 0 to remove 1/255 filter residue at tiny exterior corners. RGB values and alpha 2–255 are preserved. Legacy ICNS masks receive the same cleanup. ICNS members are sorted by their four-byte type for deterministic container output. Unchanged files are not rewritten; repeated generation must preserve both published hashes and mtimes. This is format-stage cleanup, not a new drawing or a repaint of either source.

[manifest.json](manifest.json) records every generated file, decoded RGBA hash, corner alpha, container representation and window-PNG fingerprint. The Node checker verifies all 25 alpha representations. Independent Pillow decoding was also used during the update, but Python/Pillow is not a dependency of normal generation or application builds.

## Native identity and limits

Tauri bundle targets reference the new PNG/ICO/ICNS files. On Windows Tauri 2.12 obtains the default window icon from the production EXE resources; a standalone test harness has no such resource. The Rust regression checks the compile-time PNG embedding, and a separate PE audit compares every actual RT_GROUP_ICON/RT_ICON payload with the generated ICO. All six Windows frames must match, have transparent corners, and leave no unused old icon resource.

Browser tests check the new favicon/MIME/decoded alpha at 100/150% scale. Actual Windows window/taskbar appearance and Explorer cache refresh are manual checks; macOS Dock/bundle rendering has not been executed. ICNS parsing/transparency validation is not macOS acceptance. No desktop shortcut, icon cache, user profile or running player is modified by generation.

Existing migration-provenance snapshots describe historical source/artifacts; current icon identity is this directory's source provenance and manifest. Toolbar action symbols are separate UI controls and are intentionally unchanged.
