# Manual native acceptance

Use only an isolated profile and do not run a second real audio session beside an existing player. Browser automation and native fake DOM/IPC are separate from physical Windows input.

1. Start `./target/release/ae-strom.exe --fake --data-dir <absolute-new-test-directory>`. Confirm Æ Strøm title/app identity, stopped synthetic state and isolated data/WebView directory.
2. Physically drag background, ticker and channel frame in normal/compact modes. Buttons, volume and Settings controls retain their gestures. Check actual monitor/DPI transitions.
3. On 100%/150% Windows scale, select long synthetic URLs in all right-side slots 6–0, scroll both ways and use keyboard selection/clipboard/IME. Selection must stay inside the native input.
4. Check 110 px Settings preset fields and 24×24 arrow areas, bank/slot distinction and drafts. EQ has bottom labels, dark ribbed grips, gray tracks, a seven-line scale and transient values above hover/focus/dragged sliders. Compare thumb centers at +15/0/−15.
5. Compact stays 102×36 with three visible controls; expansion restores the panel/drafts and player state. Normal 725×36, Settings 725×258 and EQ 725×257 stay fixed.
6. Ctrl+C outside editors copies the full Rust title. Settings/EQ/mute states and physical-key shortcuts must stay independent of text editing.
7. Only after other radio players are closed, intentionally launch `--enable-live-audio` with a fresh absolute --data-dir. Enter the user-approved stream, set app volume 10% and Play briefly. Confirm audible output yourself, then Stop→Play and orderly Close. Do not change system volume/device. PCM/callback diagnostics alone do not establish perceived quality.

No profile import is part of this migration. Choose and authorize that separately. Long listening soak, accessibility/IME, installers/code signing and non-Windows native hosts remain unverified until explicitly exercised.

Check startup with missing, valid and offscreen synthetic geometry: there must be no visible white flash or intermediate window. Confirm Noto Sans 12 px throughout and legibility of Æ/ø and Cyrillic. Selecting an empty slot must not add a row; real errors remain visible in the ticker. Native programmatic checks do not replace these first-frame and physical-input observations.

Check startup with missing, valid and offscreen synthetic geometry: there must be no visible white flash or intermediate window. Confirm Noto Sans 12 px throughout and legibility of Æ/ø and Cyrillic. Selecting an empty slot must not add a row; real errors remain visible in the ticker. Native programmatic checks do not replace these first-frame and physical-input observations.
