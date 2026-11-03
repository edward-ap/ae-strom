# Migration UI follow-up

The accepted compact UI is retained. Normal 725×36, compact 102×36, Settings 725×258, EQ 725×257 logical px.

The protruding status row is replaced by ticker feedback and an off-layout accessible live region. Both empty-slot and real decoder-error regressions verify 36 px chrome and closed panels. The host now creates hidden/black and reveals after the frontend fonts/layout and host monitor geometry are ready, without a delay. Standard bundled Noto Sans at 12 px is uniform throughout; font file integrity and successful local loading are tested. EQ scale labels use 44 px boxes to fit that text.

Settings preset width 110 px, native square arrow 24×24 and channel frame 40×26 are unchanged. Frame/button colors, dark ribbed grips, EQ grid, lower frequency labels and transient values remain as accepted. Browser screenshots at DPR 1/1.5 were inspected, with Unicode fixture captures. Full final evidence, binary hash and honest native-input/first-frame limitations are in VERIFICATION.md.

## Idle branding

Stopped state without a pending error displays -= Æ Strøm =- centered in the ticker. The standard ScrollingLabel is retained: public setPaused controls motion, while three scoped idle CSS rules center its track and hide the decorative repeated copy under narrow clipping. Connecting/reconnecting, errors and playing titles retain their existing text and marquee behavior. Compact remains 102×36 and hides the ticker; expanding restores the prior panel/drafts and correct ticker state. No framework/vendor, Rust backend, profile or audio-session changes were made.

A temporary gradient was compared in isolated browser previews at DPR 1/1.5. Uniform text was kept: solid text/background contrast is 9.13:1 versus 4.82:1 at the trial gradient edges, and the 12 px punctuation reads more clearly. No effect, timer or animated gradient was shipped. Reference normal/Settings/EQ screenshots were refreshed only when their previous hashes matched the committed synthetic baseline.

Final checks: 72 browser regressions, 26 JS syntax files, strict types/vendor integrity, fmt and locked offline release build; 35 native fake checks plus 7 checks in each of two default-live/stopped synthetic save/restart runs. No physical OS-input or listening pass is claimed. The personal profile was neither opened nor written, and no audio was started during this UI update. Binary/hash and artifact paths are in VERIFICATION.md and ignored local-data/idle-brand-verification.json.
