fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_snapshot",
            "ui_ready",
            "dispatch_action",
            "set_window_layout",
            "start_drag",
            "copy_title",
            "shutdown",
            "save_ui_evidence",
        ]),
    ))
    .expect("build isolated mini-radio host");
}
