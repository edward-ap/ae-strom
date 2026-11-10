# Font audit

## Exact standard face

The app uses the stock **Noto Sans variable**, normal style, CSS weight range **100–900** from pinned **taet-ui@0.6.0-rc.7** (source 53c5b177a5b68f1c1779afe97316480c65101840). Current app text is **12 px**, measured weight 400. Tæt's root tokens are --dui-font-family: "Noto Sans", "Segoe UI", Arial, sans-serif; --dui-font-size: 12px; --dui-line-height: 18px. Per-control line heights remain deliberate layout values rather than separate font sizes.

The shipped file is **web/vendor/fonts/NotoSans-variable.woff2**, **918,168 bytes**, SHA256 **7ed77c38412a34312e975abcd5eb2b2848a596f1921393f925a53a27938eb116**. The adjacent NotoSans-OFL.txt carries Copyright 2022 The Noto Project Authors and **SIL Open Font License 1.1**; its source collection is latin-greek-cyrillic. The original @font-face has only a local asset URL, normal style, 100–900 weights and font-display:swap; no local() font lookup or CDN is used. The existing native reveal handshake waits for the loaded face. No OS font installation is required.

## Coverage and corrections

All 21 diagnostic groups request the same Noto stack and 12 px normal text: body/owner, toolbar, ticker/idle, bank/channel markers, Settings labels/URL inputs/selects/options, EQ preset/custom-name fields/buttons, slider labels/transient values/scale and hidden status. Placeholder styles match. Current native evidence includes 11 select controls and their option DOM. Main, Settings and EQ geometry regressions remain 725×36, 725×258 and 725×257; compact remains 102×36.

Legacy app CSS still contained 9/10/14 px declarations, most already superseded by later overrides, plus a 10 px status declaration. They were normalized to Tæt tokens and the duplicate body shorthand removed. Native option/optgroup DOM now explicitly requests the same family/size. No vendor/core file, Rust backend, dependency version or personal configuration was changed. SVG button icons and the spectrum bars are vector/canvas geometry and do not depend on a text font. Unused framework code-block/color-picker monospace styles are not instantiated by this app and were not modified.

## Executed evidence

- **76/76 Chromium browser checks**, DPR 1/1.5; 28 JS syntax files, strict types, immutable vendor integrity, fmt and locked offline application release build PASS.
- Browser WOFF2 response body has the exact manifest byte count and SHA256 above. Chromium CSS.getPlatformFontsForNode reports **Noto Sans / NotoSans / isCustomFont=true** for the idle label, Latin/Nordic/Cyrillic/Greek sample, Settings header, URL text and closed select text. This distinguishes actual web-font rendering from a mere computed family declaration.
- **36/36 real Windows Tauri fake checks**, plus **7 checks in each of 2 default-live/stopped portable starts**. The latter folder contained only the EXE and synthetic settings, with no adjacent web/font/source directories. FontFaceSet reports loaded Noto Sans, normal, 100–900, and the real embedded resource **http://tauri.localhost/vendor/fonts/NotoSans-variable.woff2** reports **918,168 encoded/decoded bytes**. It is Tauri's virtual local asset route, not a font CDN. All 21 native DOM groups are 12 px. This establishes presence/loading from the built EXE; native resource timing is not described as a separate cryptographic extraction of PE bytes.
- Actual native environment: WebView2 156.0.4314.58, scale 1.5, Win32. Earlier Rust unit/doc/Clippy results remain historical for byte-identical Rust source; this change rebuilt the app and reran font/UI gates. No real audio or personal profile was opened in this audit. The already-running external player was not replaced, stopped or restarted.

## Explicit platform limits

1. SelectInput currently takes its **native select** path, not a DOM custom popup. The closed field and option DOM request Noto Sans, but the opened OS/browser picker font was **not measured** and is not guaranteed identical. HTML title tooltips and the Windows startup MessageBox are native/browser-owned surfaces outside app CSS; no Noto claim is made for those.
2. Missing scripts/emoji legitimately use system fallback. The Windows probe rendered two Han glyphs with **Microsoft YaHei** and an emoji with **Segoe UI Emoji**, while the Latin prefix stayed custom Noto Sans. Those are observed Windows results, not bundled fonts or predictions for Mac. This one font file is not a full Unicode/emoji/CJK font package.
3. **macOS/WKWebView was not executed**, and no Mac binary, actual Mac popup/tooltip, rasterization, fallback or layout parity is certified. Tauri uses [WebView2 on Windows and WKWebView on macOS](https://v2.tauri.app/reference/webview-versions/); WKWebView support tracks the OS. WebKit's newer [customizable select](https://webkit.org/blog/18325/webkit-features-for-safari-27-0/#customizable-select) is opt-in/version-dependent; the current appearance:none square-arrow styling does not opt into it. No control replacement or platform minimum was silently introduced.

Before claiming Mac acceptance, run the same font/load diagnostics on the actual supported WKWebView, inspect Nordic/Cyrillic/Greek plus missing-script fallback, and physically inspect native picker/title tooltips and the fixed panel/compact geometry. Shared font bytes remove the primary-face dependency on an OS-installed font; they do not guarantee pixel-identical native widgets or all-glyph fallback.

## Artifact

The tested full EXE is **ae-strom.exe**, **18593280 bytes**, SHA256 **3a2201d8558b8c2aa152533cf42cae0c4aa4485216ec010c818b19e8abb84807**. Canonical project promotion: **True**. No launcher is needed; the existing portable player profile and Stopped/no-autoplay startup remain. The active external player/profile were not accessed. Binary rollback is retained in ignored local-data/font-audit-rollback-*.

Ignored evidence: font-audit-ui.log, font-audit-build.log, font-audit-native.log, font-audit-single.log, font-audit-rendered-fonts.json and font-audit-verification.json. Native font/resource record: local-data/single exe smoke 20261103-230926-130/native-1.json. Source and exact font hashes are also recorded in web/vendor/manifest.json and SOURCE_PROVENANCE.json.
