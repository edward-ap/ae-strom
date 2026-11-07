# Tæt UI consumer notes

This migration follows an earlier source-aware audit; it is not a blind docs-only benchmark. Public rc.4 declarations and recipes were inspected before implementation. No framework core/vendor behavior is patched.

- Source package root, components and experimental exports have distinct maturity. TrackBar and ScrollingLabel remain experimental. The built profile combines foundation and components once, and its facade mirrors those public declarations, including icons. The older app-local facade's icons omission is removed.
- App layout deliberately uses dui classes, data-dui-control-id and CSS variables; those prefixes remain valid in Tæt. Do not globally rename them. EQ grid reads the pinned grip height and its own explicit track border. Browser pixel regressions guard actual native thumb-center alignment.
- Settings SelectInput remains a genuine native select with app-scoped appearance:none and a pointer-transparent24×24 decorative arrow area. This is application styling, not a hidden core fix.
- SpectrumAnalyzer uses public BaseControl lifecycle methods and an app-owned mutation guard with the public LifecycleError type. Its RAF/timer/observer lifecycle, veto/disable behavior and pending-frame destruction are regression-tested. Keep it app-owned and on the same BaseControl identity; core rc.4 intentionally does not provide the specialized spectrum API.
- Native fake checks are programmatic DOM/IPC. They do not certify physical OS pointer/clipboard/IME or accessibility. Dev tools are not packaged runtime dependencies.

Tauri PackageInfo.name follows config.productName when provided; it is the display name, not Cargo's crate_name. The native identity gate therefore checks Æ Strøm there, while Cargo metadata and the ae-strom.exe path identify the ASCII technical package. This was verified against the pinned tauri-codegen source before native testing.

The full main executable now defaults to live/stopped with a portable profile. This is native Rust startup policy, independent of the UI framework. Fake is explicit, and no launcher/script host is needed.

ScrollingLabelOptions has no public alignment option in the pinned declarations. Idle branding therefore uses public setPaused/setText and scoped app styling for centering/repeat visibility; its core lifecycle, observers and playing marquee remain unchanged. No new core patch or runtime dependency was introduced.

The standard face is the bundled Noto Sans variable, with app text routed through Tæt font tokens. Native select popup/title tooltip typography is platform-owned and cannot be certified by computed DOM styles; FONT_AUDIT.md separates those limits from actual custom-face evidence.
