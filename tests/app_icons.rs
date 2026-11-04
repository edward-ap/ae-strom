//! Compile-time Tauri window icon identity; creates no window/runtime/profile.
#[test]
fn tauri_embedded_window_png_matches_transparent_radio_asset() {
    let icon = tauri::include_image!("icons/32x32.png");
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../icons/manifest.json")).unwrap();
    assert_eq!((icon.width(), icon.height()), (32, 32));
    let mut fingerprint = 0xcbf29ce484222325u64;
    for byte in icon.rgba() {
        fingerprint = (fingerprint ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
    }
    assert_eq!(
        format!("{fingerprint:016x}"),
        manifest["windowIcon"]["rgbaFnv1a64"].as_str().unwrap()
    );
    for (x, y) in [(0, 0), (31, 0), (0, 31), (31, 31)] {
        assert_eq!(icon.rgba()[(y * 32 + x) * 4 + 3], 0);
    }
    assert!(
        icon.rgba()[(16 * 32 + 16) * 4 + 3] >= 240,
        "navy tile interior must be retained"
    );
}
