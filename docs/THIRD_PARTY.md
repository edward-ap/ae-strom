# Third-party components in Windows releases

The application is MIT licensed; see [LICENSE](../LICENSE). Tæt UI, app-owned extensions and bundled Noto Sans retain their own notices under `web/vendor`, `web/extensions` and `web/vendor/fonts`.

Each Windows release includes a separate `ae-strom-<version>-licenses.zip` asset containing license texts, component/version inventory and source links for the locked Windows Rust graph, including build/test dependencies. It is attribution/source documentation, not an application runtime dependency. Preserve it when redistributing the executable.

Unmodified MPL-2.0 components include [Symphonia 0.5.5](https://crates.io/crates/symphonia/0.5.5) and [selectors 0.38.0](https://crates.io/crates/selectors/0.38.0). The inventory gives exact-version source archive and browser links for every crate; source remains available in the published registry archives. See the [MPL distribution guidance](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).

The inventory is derived from `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc` and Cargo.lock. Packaged license files are retained verbatim; where packaging omits them, immutable upstream revisions from `.cargo_vcs_info.json` are used. `LICENSE_INVENTORY.json` records those public URLs and hashes without local manifest paths. Two crates lack a standalone license file even at the upstream revision: selectors retains its MPL source notice plus the standard MPL text; realfft retains its published MIT declaration, author attribution and an explicitly labelled standard MIT template. This distinction is recorded rather than presenting generated terms as an upstream file.
