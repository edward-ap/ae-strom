#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ae_strom::{
    config,
    controller::{Action, Controller, PlaybackMode, Snapshot, Update},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
};
use tauri::{Emitter, Manager, State, WebviewWindow};

struct Host {
    controller: Controller,
    data: PathBuf,
    smoke: bool,
    layouts: Mutex<Vec<Value>>,
    startup: Arc<(Mutex<StartupState>, Condvar)>,
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
            ae_strom::startup::MonitorArea {x:position.x,y:position.y,width:size.width,height:size.height,scale:m.scale_factor(),primary:primary.as_ref().is_some_and(|p|p.position()==position&&p.size()==size)}
        }).collect();
        let planned = ae_strom::startup::plan_position(state.saved, &monitors, (info.width, info.height)).ok_or("No usable monitor for startup")?;
        let visible_before = window.is_visible().map_err(|e|e.to_string())?;
        window.set_position(tauri::PhysicalPosition::new(planned.x,planned.y)).map_err(|e|e.to_string())?;
        window.set_size(tauri::LogicalSize::new(info.width,info.height)).map_err(|e|e.to_string())?;
        let positioned = window.outer_position().map_err(|e|e.to_string())?;
        let size_before = window.inner_size().map_err(|e|e.to_string())?;
        window.show().map_err(|e|e.to_string())?;
        state.evidence=Some(json!({"visibleBefore":visible_before,"visibleAfter":window.is_visible().map_err(|e|e.to_string())?,"positionBeforeShow":positioned,"positionAfterShow":window.outer_position().map_err(|e|e.to_string())?,"sizeBeforeShow":size_before,"sizeAfterShow":window.inner_size().map_err(|e|e.to_string())?,"planned":planned,"monitors":monitors,"ui":info,"arbitraryDelay":false}));
        Ok(())
    }).await.map_err(|e|e.to_string())?
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
}
#[tauri::command]
async fn set_window_layout(
    window: WebviewWindow,
    layout: WindowLayout,
    state: State<'_, Host>,
) -> Result<(), String> {
    main_window(&window)?;
    let max_width = if layout.compact { 400. } else { 1400. };
    if !layout.width.is_finite()
        || !layout.height.is_finite()
        || !(100.0..=max_width).contains(&layout.width)
        || !(32.0..=1000.0).contains(&layout.height)
    {
        return Err("Window dimensions out of range".into());
    }
    window
        .set_size(tauri::LogicalSize::new(layout.width, layout.height))
        .map_err(|e| e.to_string())?;
    if state.smoke {
        let mut layouts = state.layouts.lock().unwrap();
        if layouts.len() < 64 {
            layouts.push(json!({"compact": layout.compact, "requestedLogical": [layout.width, layout.height], "actualPhysical": window.inner_size().map_err(|e|e.to_string())?, "scaleFactor": window.scale_factor().map_err(|e|e.to_string())?}));
        }
    }
    Ok(())
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
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        {
            clipboard_win::set_clipboard_string(&title).map_err(|e| e.to_string())
        }
        #[cfg(not(windows))]
        {
            let _ = title;
            Err("Native title clipboard is currently implemented on Windows".to_owned())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
async fn finish(app: tauri::AppHandle, controller: Controller) -> Result<(), String> {
    let result = tauri::async_runtime::spawn_blocking(move || controller.shutdown())
        .await
        .map_err(|e| e.to_string())?;
    if let Err(ref e) = result {
        let _ = app.emit_to("main", "config-error", e);
        return result;
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
    let record = json!({"ui":ui,"host":{"pid":std::process::id(),"packageName":app.package_info().name,"productName":app.config().product_name,"identifier":app.config().identifier,"windowTitle":window.title().map_err(|e|e.to_string())?,"version":app.package_info().version.to_string(),"dataIsAbsolute":state.data.is_absolute(),"profileName":state.data.file_name().map(|s|s.to_string_lossy().into_owned()),"webview":tauri::webview_version().unwrap_or_default(),"scaleFactor":window.scale_factor().map_err(|e|e.to_string())?,"size":window.inner_size().map_err(|e|e.to_string())?,"mode":state.controller.snapshot().mode,"audioAllowed":false,"engineDiagnostics":state.controller.diagnostics().ok(),"layouts":*state.layouts.lock().unwrap(),"startup":state.startup.0.lock().unwrap().evidence}});
    let path = state.data.join("native-ui.json");
    std::fs::write(path, serde_json::to_vec_pretty(&record).unwrap()).map_err(|e| e.to_string())?;
    let controller = state.controller.clone();
    finish(app, controller).await
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = ae_strom::launch::parse(std::env::args().skip(1), &std::env::current_exe()?)
        .map_err(std::io::Error::other)?;
    if options.mode == PlaybackMode::Live && !cfg!(feature = "live-audio") {
        return Err("Rebuild with --features live-audio before enabling real audio".into());
    }
    std::fs::create_dir_all(&options.data)?;
    let config_path = options.data.join("config.json");
    if let Some(source) = options.import.as_ref() {
        config::import_explicit(source, &config_path)?;
    }
    let restore = config::load_or_default(&config_path)?;
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
        .setup(move |app| {
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
                .map_err(std::io::Error::other)?;
            app.manage(Host {
                controller,
                data: options.data.clone(),
                smoke: options.smoke || options.startup_probe,
                layouts: Mutex::new(Vec::new()),
                startup: Arc::new((
                    Mutex::new(StartupState {
                        host_ready: false,
                        saved: restore
                            .window_pos_valid
                            .then_some((restore.window_x, restore.window_y)),
                        evidence: None,
                    }),
                    Condvar::new(),
                )),
            });
            let _window = tauri::WebviewWindowBuilder::from_config(
                app.handle(),
                &app.config().app.windows[0],
            )?
            .visible(false)
            .background_color(tauri::window::Color(0, 0, 0, 255))
            .data_directory(options.data.join("webview"))
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
            .build()?;
            // No window is visible until the frontend has settled fonts/layout.
            let host = app.state::<Host>();
            let (lock, wake) = &*host.startup;
            lock.lock().unwrap().host_ready = true;
            wake.notify_all();
            Ok(())
        })
        .on_window_event(|window, event| {
            let Some(host) = window.app_handle().try_state::<Host>() else {
                return;
            };
            match event {
                tauri::WindowEvent::Moved(position) => {
                    host.controller.position(position.x, position.y)
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
        .build(tauri::generate_context!())?;
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            if let Some(host) = handle.try_state::<Host>() {
                let _ = host.controller.shutdown();
            }
        }
    });
    Ok(())
}
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    if let Err(error) = run() {
        eprintln!("Æ Strøm: {error}");
        #[cfg(windows)]
        {
            #[link(name = "user32")]
            extern "system" {
                fn MessageBoxW(
                    window: *mut std::ffi::c_void,
                    text: *const u16,
                    caption: *const u16,
                    flags: u32,
                ) -> i32;
            }
            let text:Vec<u16>=format!("Unable to start Æ Strøm. Keep the portable folder writable; no other profile was substituted.\n{error}").encode_utf16().chain(Some(0)).collect();
            let caption: Vec<u16> = "Æ Strøm".encode_utf16().chain(Some(0)).collect();
            unsafe {
                MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
            }
        }
        std::process::exit(1);
    }
}
