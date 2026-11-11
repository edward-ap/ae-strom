# Third-party components

The application is MIT licensed; see [LICENSE](../LICENSE). Tæt UI, app-owned extensions and bundled Noto Sans retain their own notices under `web/vendor`, `web/extensions` and `web/vendor/fonts`.

Each Windows release includes a separate `ae-strom-<version>-licenses.zip` asset containing license texts, component/version inventory and source links for the locked Windows Rust graph, including build/test dependencies. It is attribution/source documentation, not an application runtime dependency. Preserve it when redistributing the executable.

Unmodified MPL-2.0 components include [Symphonia 0.5.5](https://crates.io/crates/symphonia/0.5.5) and [selectors 0.38.0](https://crates.io/crates/selectors/0.38.0). The inventory gives exact-version source archive and browser links for every crate; source remains available in the published registry archives. See the [MPL distribution guidance](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).

The inventory is derived from `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` and Cargo.lock. Packaged license files are retained verbatim; where packaging omits them, immutable upstream revisions from `.cargo_vcs_info.json` are used. `LICENSE_INVENTORY.json` records those public URLs and hashes without local manifest paths. Two crates lack a standalone license file even at the upstream revision: selectors retains its MPL source notice plus the standard MPL text; realfft retains its published MIT declaration, author attribution and an explicitly labelled standard MIT template. This distinction is recorded rather than presenting generated terms as an upstream file.


## macOS local release candidate

The macOS ARM64 graph is resolved separately from the Windows graph. Run
`python3 scripts/macos-licenses.py` from a checkout with the locked dependencies
available to `cargo metadata --locked --offline`. The script uses default features
and `--filter-platform aarch64-apple-darwin`, includes build/dev dependencies, and
writes only to ignored `local-data/publication-0.1.4-macos-licenses` by default.
It does not publish or modify Cargo manifests. Python 3.9+, Cargo, curl and network
access for missing public source archives are required.

For the 0.1.4 candidate this inventory contains 358 third-party Rust crates, plus
application, Tæt UI, app-owned extension and Noto Sans notices. The generated ZIP
contains `LICENSE_INVENTORY.json`, original notice files, source evidence for
unresolved notices, `README.md` and `SHA256SUMS`. Registry archives are verified
against Cargo.lock SHA-256 checksums. When a published crate omits standalone
notices, the script reads the exact upstream revision from `.cargo_vcs_info.json`
and records the public source URL and archive/file hashes. Metadata contains no
local Cargo cache paths. UI manifest provenance and runtime file hashes are retained.

The macOS candidate supplements two crates whose published archives and relevant
pinned upstream directories omit standalone license files. For realfft 3.5.0,
it preserves the published MIT declaration and author attribution alongside a
clearly labelled standard MIT template. No copyright owner or year is invented;
the template retains its placeholders and is not represented as an upstream
copyright notice. For selectors 0.38.0, it preserves the source MPL notice and
includes the full standard MPL 2.0 text with the exact-version registry source URL.

These supplements reuse the public 0.1.3 license asset, pinned by its SHA-256,
after verifying identical crate versions, license declarations and byte-for-byte
Cargo.toml.orig/README evidence (plus selectors lib.rs). The inventory records
standard-text origins, hashes and the prior-release retrieval source separately
from verbatim upstream notices. This closes the missing standard-text packaging,
but is not legal assurance: realfft still supplies no standalone copyright notice.
The generated README and inventory explicitly retain that caveat.

The count describes the resolved graph, not only code linked into the binary.
Apple system frameworks and unshipped npm test/build tools are outside this
Rust/UI inventory.

Preserve the appropriate platform license archive alongside any eventual app
distribution. Regenerate it after Cargo.lock, selected features or bundled UI
assets change, and verify that its application version matches the candidate.
