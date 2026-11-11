//! Small AppKit boundary. Native objects never leave the main thread.
use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertStyle, NSApplication, NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;

/// Called through Tauri's main-thread dispatcher, never its async worker pool.
pub fn copy_text(text: &str) -> Result<(), String> {
    let _main = MainThreadMarker::new().ok_or("Clipboard requires the main thread")?;
    let pasteboard = NSPasteboard::generalPasteboard();
    let text = NSString::from_str(text);
    pasteboard.clearContents();
    // SAFETY: AppKit's immutable NSString pasteboard-type constant is valid.
    if pasteboard.setString_forType(&text, unsafe { NSPasteboardTypeString }) {
        Ok(())
    } else {
        Err("Unable to copy the title to the clipboard".into())
    }
}

/// Startup invokes this from main() or Tauri's main-thread setup callback.
/// Only sanitized Failure text is accepted, never an underlying OS/config error.
pub fn present_startup_failure(error: &crate::startup_failure::Failure) {
    let Some(main) = MainThreadMarker::new() else {
        log::error!("Startup dialog requires the main thread; sanitized error was sent to stderr");
        return;
    };
    let app = NSApplication::sharedApplication(main);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    let alert = NSAlert::new(main);
    alert.setAlertStyle(NSAlertStyle::Critical);
    alert.setMessageText(&NSString::from_str("Æ Strøm — startup error"));
    alert.setInformativeText(&NSString::from_str(&error.to_string()));
    // NSAlert supplies its standard OK button when none is specified.
    alert.runModal();
}

/// Custom menu item: native predefined Quit calls NSApplication.terminate:
/// directly and bypasses Tauri's preventable ExitRequested save path.
pub const QUIT_MENU_ID: &str = "ae-strom-quit";

/// Preserve Tauri's native menus (especially editor cut/copy/paste), replacing
/// only the app-menu Quit command. The host must route this ID through finish().
/// Dock Quit and OS termination still use NSApplication.terminate: directly.
/// Pinned Tao provides no applicationShouldTerminate: interception; its
/// applicationWillTerminate: notification is already too late to veto exit.
/// Those paths retain best-effort save only. Do not replace/swizzle Tao's
/// delegate here: it also owns launch, URL, focus and native event routing.
pub fn configure_menu(app: &tauri::AppHandle) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    let _main = MainThreadMarker::new().ok_or("Menu setup requires the main thread")?;
    let menu = Menu::default(app).map_err(|e| e.to_string())?;
    let menus = menu.items().map_err(|e| e.to_string())?;
    let app_menu = menus
        .first()
        .and_then(|item| item.as_submenu())
        .ok_or("Default application menu unavailable")?;
    let items = app_menu.items().map_err(|e| e.to_string())?;
    // Pinned Tauri 2.12.1 ends its application submenu with predefined Quit.
    // Validate against a fresh native Quit label rather than assuming a locale.
    let native_quit = items
        .last()
        .and_then(|item| item.as_predefined_menuitem())
        .ok_or("Default Quit command unavailable")?;
    let label = native_quit.text().map_err(|e| e.to_string())?;
    let expected = PredefinedMenuItem::quit(app, None)
        .and_then(|item| item.text())
        .map_err(|e| e.to_string())?;
    if label != expected {
        return Err("Unexpected application menu structure".into());
    }
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, label, true, Some("CmdOrCtrl+Q"))
        .map_err(|e| e.to_string())?;
    app_menu.remove(native_quit).map_err(|e| e.to_string())?;
    app_menu.append(&quit).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}

/// Apply both bounds synchronously on AppKit's owning thread. Tauri/Tao's
/// separate setters enqueue further main-queue work even from the main thread.
/// Coordinates use Tao's physical top-left desktop convention; size is logical.
pub fn set_window_bounds(
    window: &tauri::WebviewWindow,
    x: i32,
    y: i32,
    width: f64,
    height: f64,
) -> Result<(), String> {
    use objc2_app_kit::NSWindow;
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    let _main = MainThreadMarker::new().ok_or("Window layout requires the main thread")?;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err("Invalid logical window dimensions".into());
    }
    let raw = window
        .ns_window()
        .map_err(|_| "Native window unavailable")?;
    // SAFETY: Tauri owns this live NSWindow, the WebviewWindow remains borrowed,
    // and the main-thread guard prevents concurrent AppKit access.
    let native = unsafe { raw.cast::<NSWindow>().as_ref() }.ok_or("Native window unavailable")?;
    let scale = native.backingScaleFactor();
    if !scale.is_finite() || scale <= 0.0 {
        return Err("Invalid window backing scale".into());
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGMainDisplayID() -> u32;
        fn CGDisplayPixelsHigh(display: u32) -> usize;
    }
    // Match pinned Tao's util::window_position/bottom_left_to_top_left exactly.
    // The desktop origin is anchored to the primary display, not the window's
    // current screen, so negative/secondary-display coordinates remain valid.
    let primary_height = unsafe { CGDisplayPixelsHigh(CGMainDisplayID()) } as f64;
    if primary_height <= 0.0 {
        return Err("Primary display unavailable".into());
    }
    let content = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height));
    let mut frame = native.frameRectForContentRect(content);
    frame.origin = frame_origin(x, y, scale, primary_height, frame.size.height);
    native.setFrame_display(frame, true);
    if let Some(view) = native.contentView() {
        // AppKit resizes the content view synchronously; Wry's WKWebView uses
        // width/height autoresizing. Flush pending constraints without modifying
        // unrelated native subviews or changing their autoresizing policies.
        view.layoutSubtreeIfNeeded();
    }
    Ok(())
}

fn frame_origin(
    x: i32,
    y: i32,
    scale: f64,
    primary_height: f64,
    frame_height: f64,
) -> objc2_foundation::NSPoint {
    objc2_foundation::NSPoint::new(
        f64::from(x) / scale,
        primary_height - f64::from(y) / scale - frame_height,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn frame_origin_roundtrips_tao_coordinates_on_secondary_displays() {
        for (x, y, scale, primary, height) in [
            (1003, 1081, 2.0, 1117.0, 36.0),
            (-1920, -240, 1.0, 1080.0, 600.0),
            (3600, 2160, 2.0, 1440.0, 420.0),
        ] {
            let origin = super::frame_origin(x, y, scale, primary, height);
            assert_eq!((origin.x * scale).round() as i32, x);
            assert_eq!(((primary - (origin.y + height)) * scale).round() as i32, y);
        }
    }

    #[test]
    fn clipboard_rejects_worker_thread_before_accessing_pasteboard() {
        let result = std::thread::spawn(|| super::copy_text("synthetic test"))
            .join()
            .unwrap();
        assert_eq!(result.unwrap_err(), "Clipboard requires the main thread");
    }

    #[test]
    fn startup_guidance_does_not_require_windows_runtime() {
        use crate::startup_failure::{Failure, Kind};
        for kind in [Kind::AppData, Kind::WebView, Kind::Runtime] {
            let text = Failure(kind).to_string();
            assert!(!text.contains("Windows"));
            assert!(!text.contains("WebView2"));
        }
    }
}
