#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ae_strom::controller::{Action, Controller, PlaybackMode, Snapshot, Update};
use ae_strom::startup_failure::{self, Failure, Kind};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
};
use tauri::{Emitter, Manager, State, WebviewWindow};

struct Host {
    controller: Controller,
    closing: Arc<CloseState>,
    profile: ae_strom::profile::Prepared,
    data: PathBuf,
    smoke: bool,
    layouts: Mutex<Vec<Value>>,
    layout_lock: Mutex<()>,
    panels_above: AtomicBool,
    startup: Arc<(Mutex<StartupState>, Condvar)>,
}
/// Coalesce all UI/native close sources and allow retry after a save failure.
#[derive(Default)]
struct CloseState {
    pending: AtomicBool,
    complete: AtomicBool,
}
impl CloseState {
    fn begin(&self) -> bool {
        self.pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    fn failed(&self) {
        self.pending.store(false, Ordering::Release);
    }
    fn succeeded(&self) {
        self.complete.store(true, Ordering::Release);
    }
}

/// Exactly one side can claim queued native work: the callback or cancellation.
#[cfg(any(target_os = "macos", test))]
#[derive(Default)]
struct NativeClaim(AtomicBool);
#[cfg(any(target_os = "macos", test))]
impl NativeClaim {
    fn take(&self) -> bool {
        !self.0.swap(true, Ordering::AcqRel)
    }
}

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UiReadyInfo {
    width: f64,
    height: f64,
    font_loaded: bool,
    background: String,
}
struct StartupState {
    host_ready: bool,
    saved: Option<(i32, i32)>,
    evidence: Option<Value>,
    default_context_menus_enabled: Option<bool>,
}
#[tauri::command]
async fn ui_ready(
    window: WebviewWindow,
    state: State<'_, Host>,
    info: UiReadyInfo,
) -> Result<(), String> {
    main_window(&window)?;
    if !info.width.is_finite()
        || !info.height.is_finite()
        || !(100.0..=1400.0).contains(&info.width)
        || !(32.0..=1000.0).contains(&info.height)
        || !info.font_loaded
        || info.background != "rgb(0, 0, 0)"
    {
        return Err("UI geometry, bundled font or background is not ready".into());
    }
    let startup = state.startup.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (lock, wake) = &*startup;
        let mut state = lock.lock().map_err(|e| e.to_string())?;
        while !state.host_ready { state = wake.wait(state).map_err(|e|e.to_string())?; }
        if state.evidence.is_some() { return Ok(()); }
        let primary = window.primary_monitor().map_err(|e|e.to_string())?;
        let monitors: Vec<ae_strom::startup::MonitorArea> = window.available_monitors().map_err(|e|e.to_string())?.iter().map(|m| {
            let position=m.position();let size=m.size();
            #[cfg(target_os = "macos")]
            let (usable_position, usable_size) = (m.work_area().position, m.work_area().size);
            #[cfg(not(target_os = "macos"))]
            let (usable_position, usable_size) = (position, size);
            ae_strom::startup::MonitorArea {x:usable_position.x,y:usable_position.y,width:usable_size.width,height:usable_size.height,scale:m.scale_factor(),primary:primary.as_ref().is_some_and(|p|p.position()==position&&p.size()==size)}
        }).collect();
        let planned = ae_strom::startup::plan_position(state.saved, &monitors, (info.width, info.height)).ok_or("No usable monitor for startup")?;
        let visible_before = window.is_visible().map_err(|e|e.to_string())?;
        #[cfg(target_os = "macos")]
        set_macos_bounds(&window, planned.x, planned.y, info.width, info.height)?;
        #[cfg(not(target_os = "macos"))]
        {
            window.set_position(tauri::PhysicalPosition::new(planned.x,planned.y)).map_err(|e|e.to_string())?;
            window.set_size(tauri::LogicalSize::new(info.width,info.height)).map_err(|e|e.to_string())?;
        }
        let positioned = window.outer_position().map_err(|e|e.to_string())?;
        let size_before = window.inner_size().map_err(|e|e.to_string())?;
        window.show().map_err(|e|e.to_string())?;
        state.evidence=Some(json!({"visibleBefore":visible_before,"visibleAfter":window.is_visible().map_err(|e|e.to_string())?,"positionBeforeShow":positioned,"positionAfterShow":window.outer_position().map_err(|e|e.to_string())?,"sizeBeforeShow":size_before,"sizeAfterShow":window.inner_size().map_err(|e|e.to_string())?,"planned":planned,"monitors":monitors,"ui":info,"arbitraryDelay":false,"defaultContextMenusEnabled":state.default_context_menus_enabled}));
        Ok(())
    }).await.map_err(|e|e.to_string())?
}
/// Tao's macOS setters defer AppKit work even when called on the UI thread.
/// Complete one native frame transaction before acknowledging UI layout.
#[cfg(target_os = "macos")]
fn set_macos_bounds(
    window: &WebviewWindow,
    x: i32,
    y: i32,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let target = window.clone();
    let claim = Arc::new(NativeClaim::default());
    let callback_claim = claim.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    window
        .run_on_main_thread(move || {
            if !callback_claim.take() {
                return;
            }
            let _ = tx.send(ae_strom::macos::set_window_bounds(
                &target, x, y, width, height,
            ));
        })
        .map_err(|e| e.to_string())?;
    match rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) if !claim.take() => {
            // AppKit has started the operation. Observe its real result rather
            // than reporting failure while it can still change window bounds.
            rx.recv()
                .map_err(|_| "Native window layout was interrupted".to_owned())?
        }
        Err(_) => Err("Native window layout did not complete".to_owned()),
    }
}

fn main_window(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "main" {
        Ok(())
    } else {
        Err("Main window only".into())
    }
}
#[tauri::command]
fn get_snapshot(window: WebviewWindow, state: State<'_, Host>) -> Result<Snapshot, String> {
    main_window(&window)?;
    Ok(state.controller.snapshot())
}
#[tauri::command]
async fn dispatch_action(
    window: WebviewWindow,
    state: State<'_, Host>,
    action: Action,
) -> Result<Snapshot, String> {
    main_window(&window)?;
    let rx = state.controller.submit(action)?;
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv()
            .map_err(|_| "Controller response interrupted".to_owned())?
    })
    .await
    .map_err(|e| e.to_string())?
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WindowLayout {
    compact: bool,
    width: f64,
    height: f64,
    #[serde(default)]
    prepare: bool,
    #[serde(default, rename = "panelsAbove")]
    planned_above: Option<bool>,
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutResult {
    panels_above: bool,
}

#[tauri::command]
async fn set_window_layout(
    window: WebviewWindow,
    layout: WindowLayout,
    state: State<'_, Host>,
) -> Result<LayoutResult, String> {
    main_window(&window)?;
    let max_width = if layout.compact { 400. } else { 1400. };
    if !layout.width.is_finite()
        || !layout.height.is_finite()
        || !(100.0..=max_width).contains(&layout.width)
        || !(32.0..=1000.0).contains(&layout.height)
    {
        return Err("Window dimensions out of range".into());
    }
    let _layout_guard = state.layout_lock.lock().map_err(|e| e.to_string())?;
    let before_position = window.outer_position().map_err(|e| e.to_string())?;
    let before_size = window.outer_size().map_err(|e| e.to_string())?;
    let before_inner = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let strip_height = (36.0 * scale).round().max(1.0) as u32;
    // The panel lives in the client area. Non-client frame pixels belong to
    // the collapsed window and must never be counted as panel height.
    let strip_y = ae_strom::startup::player_strip_y(
        before_position.y,
        before_inner.height,
        strip_height,
        state.panels_above.load(Ordering::Acquire),
    );
    let center_x = i64::from(before_position.x) + i64::from(before_size.width) / 2;
    let center_y = i64::from(strip_y) + i64::from(strip_height) / 2;
    let monitor = window
        .available_monitors()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|monitor| {
            let p = monitor.position();
            let size = monitor.size();
            center_x >= i64::from(p.x)
                && center_x < i64::from(p.x) + i64::from(size.width)
                && center_y >= i64::from(p.y)
                && center_y < i64::from(p.y) + i64::from(size.height)
        });
    let monitor = match monitor {
        Some(m) => m,
        None => window
            .current_monitor()
            .map_err(|e| e.to_string())?
            .ok_or("No monitor for player layout")?,
    };
    let work = monitor.work_area();
    let target_inner =
        tauri::LogicalSize::new(layout.width, layout.height).to_physical::<u32>(scale);
    let frame_width = before_size.width.saturating_sub(before_inner.width);
    let frame_height = before_size.height.saturating_sub(before_inner.height);
    let target_outer = tauri::PhysicalSize::new(
        target_inner.width.saturating_add(frame_width),
        target_inner.height.saturating_add(frame_height),
    );
    let x = ae_strom::startup::fit_resize_x(
        before_position.x,
        before_size.width,
        target_outer.width,
        work.position.x,
        work.size.width,
    );
    let horizontal_anchor = if i64::from(x) + i64::from(target_outer.width)
        == i64::from(before_position.x) + i64::from(before_size.width)
    {
        "right"
    } else if x == before_position.x {
        "left"
    } else {
        "clamped"
    };
    let mut placement = ae_strom::startup::place_panel(
        strip_y,
        strip_height.saturating_add(frame_height),
        target_outer.height,
        work.position.y,
        work.size.height,
    );
    let preferred = layout.planned_above.or_else(|| {
        (state.panels_above.load(Ordering::Acquire)
            && before_inner.height > strip_height
            && target_inner.height > strip_height)
            .then_some(true)
    });
    if target_inner.height > strip_height {
        if let Some(above) = preferred {
            let y = i64::from(strip_y)
                - if above {
                    i64::from(target_inner.height - strip_height)
                } else {
                    0
                };
            if y >= i64::from(work.position.y)
                && y + i64::from(target_outer.height)
                    <= i64::from(work.position.y) + i64::from(work.size.height)
            {
                placement.y = i32::try_from(y).map_err(|_| "Panel position overflow")?;
                placement.panels_above = above;
            }
        }
    }
    if layout.prepare {
        return Ok(LayoutResult {
            panels_above: placement.panels_above,
        });
    }
    let visible_layout = state
        .startup
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .evidence
        .is_some();
    state
        .panels_above
        .store(placement.panels_above, Ordering::Release);
    #[cfg(windows)]
    if visible_layout {
        let raw_hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        let width = i32::try_from(target_outer.width).map_err(|_| "Window width overflow")?;
        let height = i32::try_from(target_outer.height).map_err(|_| "Window height overflow")?;
        let y = placement.y;
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        window
            .run_on_main_thread(move || {
                use windows::Win32::{
                    Foundation::HWND,
                    UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER},
                };
                // SAFETY: this live window handle is used on its owning UI thread,
                // with the same DPI awareness as Tauri and its WebView child.
                let result = unsafe {
                    SetWindowPos(
                        HWND(raw_hwnd as *mut std::ffi::c_void),
                        None,
                        x,
                        y,
                        width,
                        height,
                        SWP_NOACTIVATE | SWP_NOZORDER,
                    )
                }
                .map_err(|e| e.to_string());
                let _ = tx.send(result);
            })
            .map_err(|e| e.to_string())?;
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "Native window layout did not complete")??;
    } else {
        window
            .set_size(tauri::LogicalSize::new(layout.width, layout.height))
            .map_err(|e| e.to_string())?;
        window
            .set_position(tauri::PhysicalPosition::new(x, placement.y))
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    set_macos_bounds(&window, x, placement.y, layout.width, layout.height)?;
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        window
            .set_size(tauri::LogicalSize::new(layout.width, layout.height))
            .map_err(|e| e.to_string())?;
        window
            .set_position(tauri::PhysicalPosition::new(x, placement.y))
            .map_err(|e| e.to_string())?;
    }
    if state.smoke {
        let mut layouts = state.layouts.lock().unwrap();
        if layouts.len() < 64 {
            layouts.push(json!({"compact":layout.compact,"requestedLogical":[layout.width,layout.height],"actualPhysical":window.inner_size().map_err(|e|e.to_string())?,"scaleFactor":window.scale_factor().map_err(|e|e.to_string())?,"atomicBounds":cfg!(target_os = "macos")||(cfg!(windows)&&visible_layout),"horizontalAnchor":horizontal_anchor,"panelsAbove":placement.panels_above,"stripYBefore":strip_y,"stripYAfter":ae_strom::startup::player_strip_y(window.outer_position().map_err(|e|e.to_string())?.y,window.inner_size().map_err(|e|e.to_string())?.height,strip_height,placement.panels_above),"workArea":work,"outerBefore":{"position":before_position,"size":before_size},"outerAfter":{"position":window.outer_position().map_err(|e|e.to_string())?,"size":window.outer_size().map_err(|e|e.to_string())?}}));
        }
    }
    Ok(LayoutResult {
        panels_above: placement.panels_above,
    })
}
#[tauri::command]
fn start_drag(window: WebviewWindow) -> Result<(), String> {
    main_window(&window)?;
    window.start_dragging().map_err(|e| e.to_string())
}
#[tauri::command]
async fn copy_title(window: WebviewWindow, state: State<'_, Host>) -> Result<(), String> {
    main_window(&window)?;
    let title = state.controller.snapshot().full_title;
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        window
            .run_on_main_thread(move || {
                let _ = tx.send(ae_strom::macos::copy_text(&title));
            })
            .map_err(|e| e.to_string())?;
        tauri::async_runtime::spawn_blocking(move || {
            rx.recv()
                .map_err(|_| "Clipboard operation interrupted".to_owned())?
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(target_os = "macos"))]
    {
        tauri::async_runtime::spawn_blocking(move || {
            #[cfg(windows)]
            {
                clipboard_win::set_clipboard_string(&title).map_err(|e| e.to_string())
            }
            #[cfg(not(windows))]
            {
                let _ = title;
                Err("Native title clipboard is not implemented on this platform".to_owned())
            }
        })
        .await
        .map_err(|e| e.to_string())?
    }
}

async fn finish(app: tauri::AppHandle, controller: Controller) -> Result<(), String> {
    let closing = app.try_state::<Host>().map(|host| host.closing.clone());
    if closing.as_ref().is_some_and(|state| !state.begin()) {
        return Ok(());
    }
    let result = tauri::async_runtime::spawn_blocking(move || controller.shutdown())
        .await
        .map_err(|_| "Shutdown worker failed".to_owned())
        .and_then(|result| result);
    if let Err(ref e) = result {
        if let Some(state) = closing {
            state.failed();
        }
        let _ = app.emit_to("main", "config-error", e);
        return result;
    }
    if let Some(state) = closing {
        state.succeeded();
    }
    // Return the UI result before closing; audio/control work has already completed.
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        app.exit(0);
    });
    result
}
#[tauri::command]
async fn shutdown(
    window: WebviewWindow,
    state: State<'_, Host>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    main_window(&window)?;
    let controller = state.controller.clone();
    finish(app, controller).await
}
#[tauri::command]
async fn save_ui_evidence(
    window: WebviewWindow,
    state: State<'_, Host>,
    body: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    main_window(&window)?;
    if !state.smoke {
        return Err("Evidence capture requires --smoke".into());
    }
    if body.len() > 131072 {
        return Err("Evidence body too large".into());
    }
    let ui: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let record = json!({"ui":ui,"host":{"pid":std::process::id(),"packageName":app.package_info().name,"productName":app.config().product_name,"identifier":app.config().identifier,"windowTitle":window.title().map_err(|e|e.to_string())?,"version":app.package_info().version.to_string(),"dataIsAbsolute":state.data.is_absolute(),"configFile":state.data.join("config.json"),"webviewDirectory":if cfg!(target_os = "macos") { None } else { Some(&state.profile.webview) },"webviewStorage":if cfg!(target_os = "macos") { "nonpersistent" } else { "profile-directory" },"profileOrigin":state.profile.origin,"profileName":state.data.file_name().map(|s|s.to_string_lossy().into_owned()),"webview":tauri::webview_version().unwrap_or_default(),"scaleFactor":window.scale_factor().map_err(|e|e.to_string())?,"size":window.inner_size().map_err(|e|e.to_string())?,"mode":state.controller.snapshot().mode,"audioAllowed":false,"engineDiagnostics":state.controller.diagnostics().ok(),"layouts":*state.layouts.lock().unwrap(),"startup":state.startup.0.lock().unwrap().evidence}});
    let path = state.data.join("native-ui.json");
    std::fs::write(path, serde_json::to_vec_pretty(&record).unwrap()).map_err(|e| e.to_string())?;
    let controller = state.controller.clone();
    finish(app, controller).await
}
fn mark_host_ready(app: &tauri::AppHandle, default_context_menus_enabled: Option<bool>) {
    let host = app.state::<Host>();
    let (lock, wake) = &*host.startup;
    let mut state = lock.lock().unwrap();
    state.default_context_menus_enabled = default_context_menus_enabled;
    state.host_ready = true;
    wake.notify_all();
}
fn initialize_player(
    app: &mut tauri::App,
    options: ae_strom::launch::Options,
) -> Result<(), Failure> {
    #[cfg(target_os = "macos")]
    ae_strom::macos::configure_menu(app.handle()).map_err(|_| Failure(Kind::Runtime))?;
    let app_local = app
        .path()
        .app_local_data_dir()
        .map_err(|_| Failure(Kind::AppData))?;
    #[cfg(target_os = "macos")]
    let options = {
        let mut options = options;
        options
            .use_app_data_profile(&app_local)
            .map_err(|_| Failure(Kind::AppData))?;
        options
    };
    let config_path = options.data.join("config.json");
    let profile = ae_strom::profile::prepare(
        &config_path,
        options.legacy.as_deref(),
        options.import.as_deref(),
        &app_local,
    )
    .map_err(Failure::profile)?;
    let restore = profile.config.clone();
    let webview_directory = profile.webview.clone();
    let handle = app.handle().clone();
    let sink = Arc::new(move |update| match update {
        Update::State(s) => {
            let _ = handle.emit_to("main", "radio-state", s);
        }
        Update::Spectrum(s) => {
            let _ = handle.emit_to("main", "spectrum", s);
        }
    });
    let controller = Controller::start(config_path.clone(), options.mode, sink)
        .map_err(|_| Failure(Kind::Controller))?;
    app.manage(Host {
        controller,
        closing: Arc::new(CloseState::default()),
        profile,
        data: options.data.clone(),
        smoke: options.smoke || options.startup_probe,
        layouts: Mutex::new(Vec::new()),
        layout_lock: Mutex::new(()),
        panels_above: AtomicBool::new(false),
        startup: Arc::new((
            Mutex::new(StartupState {
                host_ready: false,
                saved: restore
                    .window_pos_valid
                    .then_some((restore.window_x, restore.window_y)),
                evidence: None,
                default_context_menus_enabled: None,
            }),
            Condvar::new(),
        )),
    });
    let builder = tauri::WebviewWindowBuilder::from_config(
        app.handle(),
        app.config()
            .app
            .windows
            .first()
            .ok_or(Failure(Kind::WebView))?,
    )
    .map_err(|_| Failure(Kind::WebView))?
    .visible(false)
    .background_color(tauri::window::Color(0, 0, 0, 255))
    .data_directory(webview_directory);
    // WKWebView ignores data_directory. The UI keeps no browser state, so a
    // nonpersistent store isolates every profile on all supported macOS versions.
    #[cfg(target_os = "macos")]
    let builder = builder.incognito(true);
    let _window = builder
        .initialization_script(if options.startup_probe {
            "window.__RADIO_STARTUP_PROBE__=true;window.__RADIO_SMOKE_AUTORUN__=false;"
        } else if options.smoke {
            "window.__RADIO_SMOKE_AUTORUN__=true;"
        } else {
            "window.__RADIO_SMOKE_AUTORUN__=false;"
        })
        .on_navigation(|url| {
            matches!(
                (url.scheme(), url.host_str()),
                ("tauri", Some("localhost")) | ("http", Some("tauri.localhost"))
            ) && url.port().is_none()
        })
        .build()
        .map_err(|_| Failure(Kind::WebView))?;
    // Configure WebView2 on its owning UI thread before releasing ui_ready.
    // Only the native menu flag changes; keyboard editing/clipboard stays native.
    #[cfg(windows)]
    {
        let handle = app.handle().clone();
        _window
            .with_webview(move |webview| {
                // SAFETY: Tauri invokes this closure on the WebView's UI thread;
                // all COM handles remain within the callback.
                let result = unsafe {
                    webview
                        .controller()
                        .CoreWebView2()
                        .and_then(|view| view.Settings())
                        .and_then(|settings| {
                            settings.SetAreDefaultContextMenusEnabled(false)?;
                            let mut enabled = Default::default();
                            settings.AreDefaultContextMenusEnabled(&mut enabled)?;
                            Ok(enabled.as_bool())
                        })
                }
                .map_err(|_| Failure(Kind::WebView))
                .and_then(|enabled| {
                    if enabled {
                        return Err(Failure(Kind::WebView));
                    }
                    mark_host_ready(&handle, Some(enabled));
                    Ok(())
                });
                startup_failure::finish_setup(result, startup_failure::present, |code| {
                    handle.exit(code)
                });
            })
            .map_err(|_| Failure(Kind::WebView))?;
    }
    #[cfg(not(windows))]
    mark_host_ready(app.handle(), None);
    Ok(())
}
fn run() -> Result<(), Failure> {
    let executable = std::env::current_exe().map_err(|_| Failure(Kind::Paths))?;
    let options = ae_strom::launch::parse(std::env::args().skip(1), &executable)
        .map_err(|_| Failure(Kind::Arguments))?;
    if options.mode == PlaybackMode::Live && !cfg!(feature = "live-audio") {
        return Err(Failure(Kind::AudioBuild));
    }
    ae_strom::webview_prerequisite::ensure_available()?;
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            ui_ready,
            dispatch_action,
            set_window_layout,
            start_drag,
            copy_title,
            shutdown,
            save_ui_evidence
        ])
        .on_menu_event(|_app, _event| {
            #[cfg(target_os = "macos")]
            if _event.id().as_ref() == ae_strom::macos::QUIT_MENU_ID {
                if let Some(host) = _app.try_state::<Host>() {
                    let controller = host.controller.clone();
                    let app = _app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = finish(app, controller).await;
                    });
                }
            }
        })
        .setup(move |app| {
            startup_failure::finish_setup(
                initialize_player(app, options),
                startup_failure::present,
                |code| app.handle().exit(code),
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            let Some(host) = window.app_handle().try_state::<Host>() else {
                return;
            };
            match event {
                tauri::WindowEvent::Moved(position) => {
                    let y = if host.panels_above.load(Ordering::Acquire) {
                        match (window.inner_size(), window.scale_factor()) {
                            (Ok(size), Ok(scale)) => ae_strom::startup::player_strip_y(
                                position.y,
                                size.height,
                                (36.0 * scale).round().max(1.0) as u32,
                                true,
                            ),
                            _ => position.y,
                        }
                    } else {
                        position.y
                    };
                    host.controller.position(position.x, y)
                }
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let controller = host.controller.clone();
                    let app = window.app_handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = finish(app, controller).await;
                    });
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .map_err(|_| Failure(Kind::Runtime))?;
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested {
            code: None,
            ref api,
            ..
        } = event
        {
            if let Some(host) = handle.try_state::<Host>() {
                api.prevent_exit();
                let controller = host.controller.clone();
                let app = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = finish(app, controller).await;
                });
            }
        }
        if matches!(event, tauri::RunEvent::Exit) {
            if let Some(host) = handle.try_state::<Host>() {
                if !host.closing.complete.load(Ordering::Acquire)
                    && host.controller.shutdown().is_err()
                {
                    // Last-chance cleanup cannot veto native Dock/OS termination.
                    // Never log configuration contents or raw storage errors here.
                    log::error!("Player exited before settings could be confirmed saved");
                }
            }
        }
    });
    Ok(())
}
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    if let Err(error) = run() {
        startup_failure::present(&error);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::{CloseState, NativeClaim};
    use std::sync::atomic::Ordering;

    #[test]
    fn duplicate_close_requests_coalesce_and_failed_save_can_retry() {
        let state = CloseState::default();
        assert!(state.begin());
        assert!(!state.begin());
        state.failed();
        assert!(state.begin());
        state.succeeded();
        assert!(state.complete.load(Ordering::Acquire));
        assert!(!state.begin());
    }

    #[test]
    fn canceled_native_callback_cannot_apply_late_bounds() {
        let claim = NativeClaim::default();
        assert!(claim.take()); // timeout cancels before main-thread execution
        assert!(!claim.take()); // callback must not execute
    }

    #[test]
    fn started_native_callback_cannot_be_reported_as_canceled() {
        let claim = NativeClaim::default();
        assert!(claim.take()); // main-thread callback starts
        assert!(!claim.take()); // timeout must wait for the actual result
    }
}
