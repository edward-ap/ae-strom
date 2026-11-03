# Migration summary

The new folder was checked empty before copying. Only public application source, tests, scripts, icon, licenses and pinned framework/type assets were copied. Old local-data, configs, logs, targets, caches, node_modules and generated permission/schema files were not copied. Development dependencies were installed from the standard registry with lifecycle scripts disabled; Rust compiled in a fresh target.

Dense rc.2 was replaced with published taet-ui 0.6.0-rc.4. The generator builds exactly one foundation+components profile from integrity/source-hash checked npm files and requires canonical JS/CSS/font/license hashes. The typed facade mirrors the profile and includes icons. Imports and product branding use Æ Strøm/ae-strom; technical dui CSS/DOM contracts remain intentionally unchanged. Specialized spectrum/channel controls stay app-owned on the same BaseControl.

Cargo package/default-run/lib and crate/doc imports were renamed; Rust dependency versions/checksums and controller/audio/config semantics were preserved. The subsequent user fixes add a pure startup-position module, a font/layout readiness handshake and nonprotruding UI feedback; see VERIFICATION.md. The subsequent user fixes add a pure startup-position module, a font/layout readiness handshake and nonprotruding UI feedback; see VERIFICATION.md. Tauri productName/window title are Æ Strøm, identifier is local.edwardap.ae-strom, default data root is AeStrom. No automatic profile discovery/migration was added. An explicit import is a separate user decision.

The local Git repository ignores generated artifacts and all private profiles/configs/logs/caches. No remote, push, tag or publication is created. Build/test/readiness details and exact binary hash are in VERIFICATION.md.
