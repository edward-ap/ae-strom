# Migration UI follow-up

The accepted compact UI is retained. Normal 725×36, compact 102×36, Settings 725×258, EQ 725×257 logical px.

The protruding status row is replaced by ticker feedback and an off-layout accessible live region. Both empty-slot and real decoder-error regressions verify 36 px chrome and closed panels. The host now creates hidden/black and reveals after the frontend fonts/layout and host monitor geometry are ready, without a delay. Standard bundled Noto Sans at 12 px is uniform throughout; font file integrity and successful local loading are tested. EQ scale labels use 44 px boxes to fit that text.

Settings preset width 110 px, native square arrow 24×24 and channel frame 40×26 are unchanged. Frame/button colors, dark ribbed grips, EQ grid, lower frequency labels and transient values remain as accepted. Browser screenshots at DPR 1/1.5 were inspected, with Unicode fixture captures. Full final evidence, binary hash and honest native-input/first-frame limitations are in VERIFICATION.md.
