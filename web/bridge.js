/** The only UI module that knows the native IPC transport. */
export function createTauriBridge(tauri = window.__TAURI__) {
  const invoke = tauri.core.invoke;
  const subscribe = (name, fn) => tauri.event.listen(name, e => fn(e.payload));
  return {
    getSnapshot: () => invoke('get_snapshot'),
    uiReady: info => invoke('ui_ready', { info }),
    dispatch: action => invoke('dispatch_action', { action }),
    subscribeState: fn => subscribe('radio-state', fn),
    subscribeSpectrum: fn => subscribe('spectrum', fn),
    setWindowLayout: layout => invoke('set_window_layout', { layout }),
    startDrag: () => invoke('start_drag'),
    copyTitle: () => invoke('copy_title'),
    shutdown: () => invoke('shutdown'),
  };
}
