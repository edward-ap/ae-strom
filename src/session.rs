use std::sync::atomic::AtomicUsize;
pub static ACTIVE_SESSION_OWNERS: AtomicUsize = AtomicUsize::new(0);
// Serializes blocking retirement outside the domain actor. Every generation
// owns its audio engine, metadata supervisor and cancellable HTTP readers.
use crate::{
    audio::{dsp::EqSettings, engine::Diagnostics, AudioEngine},
    metadata::{provider::MetadataWatch, Info, Provider, StrategyHint},
};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::Duration,
};
#[derive(Clone)]
pub struct SessionSpec {
    pub url: String,
    pub hint: StrategyHint,
    pub poll: Duration,
    pub cancel: Arc<AtomicBool>,
    pub on_info: Arc<dyn Fn(Info) + Send + Sync>,
    pub on_strategy: Arc<dyn Fn(StrategyHint) + Send + Sync>,
}
struct State {
    revision: u64,
    request: Option<SessionSpec>,
    shutdown: bool,
    active: Option<AudioEngine>,
    active_revision: Option<u64>,
    status: String,
    error: Option<String>,
    volume: u8,
    muted: bool,
    eq: EqSettings,
    diagnostics: Diagnostics,
    retiring: bool,
    finished: bool,
}
pub struct LivePlayback {
    shared: Arc<(Mutex<State>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
}
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionDiagnostics {
    pub engine: Diagnostics,
    pub active: bool,
    pub requested_revision: u64,
    pub active_revision: Option<u64>,
    pub retiring: bool,
    pub service_finished: bool,
    pub active_http_requests: usize,
}
impl LivePlayback {
    pub fn new() -> Self {
        Self::create(false)
    }
    #[cfg(test)]
    pub fn new_sink() -> Self {
        Self::create(true)
    }
    fn create(headless: bool) -> Self {
        let shared = Arc::new((
            Mutex::new(State {
                revision: 0,
                request: None,
                shutdown: false,
                active: None,
                active_revision: None,
                status: "stopped".into(),
                error: None,
                volume: 50,
                muted: false,
                eq: EqSettings::default(),
                diagnostics: Diagnostics::default(),
                retiring: false,
                finished: false,
            }),
            Condvar::new(),
        ));
        let thread_shared = shared.clone();
        let worker = thread::Builder::new()
            .name("radio-session-owner".into())
            .spawn(move || {
                ACTIVE_SESSION_OWNERS.fetch_add(1, Ordering::AcqRel);
                struct Count;
                impl Drop for Count {
                    fn drop(&mut self) {
                        ACTIVE_SESSION_OWNERS.fetch_sub(1, Ordering::AcqRel);
                    }
                }
                let _count = Count;
                let (lock, wake) = &*thread_shared;
                let mut current: Option<(u64, AudioEngine, MetadataWatch)> = None;
                let mut installed = 0;
                loop {
                    let desired = {
                        let state = lock.lock().unwrap();
                        (state.revision, state.shutdown, state.request.clone())
                    };
                    if desired.1 || desired.0 != installed {
                        if let Some((_, engine, mut metadata)) = current.take() {
                            engine.request_stop();
                            metadata.request_stop();
                            {
                                lock.lock().unwrap().retiring = true;
                            }
                            engine.stop();
                            metadata.stop();
                            let mut state = lock.lock().unwrap();
                            state.diagnostics = engine.diagnostics();
                            state.active = None;
                            state.active_revision = None;
                            state.retiring = false;
                        }
                        let mut state = lock.lock().unwrap();
                        installed = state.revision;
                        if state.shutdown {
                            break;
                        }
                        if let Some(spec) = state.request.clone() {
                            #[cfg(test)]
                            let engine = if headless {
                                AudioEngine::new_sink()
                            } else {
                                AudioEngine::new()
                            };
                            #[cfg(not(test))]
                            let engine = {
                                let _ = headless;
                                AudioEngine::new()
                            };
                            engine.set_volume(state.volume);
                            engine.set_muted(state.muted);
                            engine.set_eq_settings(state.eq.clone());
                            state.active = Some(engine.clone());
                            state.active_revision = Some(installed);
                            state.status = "connecting".into();
                            state.error = None;
                            engine.play_url(spec.url.clone());
                            let revision = installed;
                            drop(state);
                            let info = spec.on_info.clone();
                            let strategy = spec.on_strategy.clone();
                            let metadata = Provider::new(spec.poll).watch(
                                spec.url,
                                spec.hint,
                                move |value| info(value),
                                move |value| strategy(value),
                                spec.cancel,
                            );
                            current = Some((revision, engine, metadata));
                        } else {
                            state.status = "stopped".into();
                            state.error = None;
                        }
                    }
                    let mut state = lock.lock().unwrap();
                    if let Some((revision, engine, _)) = &current {
                        if *revision == state.revision {
                            let (status, error) = engine.playback_status();
                            state.status = status;
                            state.error = error;
                            state.diagnostics = engine.diagnostics();
                        }
                    }
                    let terminal = current.as_ref().is_some_and(|(_, engine, _)| {
                        engine.is_finished() && engine.playback_status().0 == "error"
                    });
                    if terminal {
                        drop(state);
                        if let Some((_, engine, mut metadata)) = current.take() {
                            metadata.stop();
                            engine.stop();
                            let mut state = lock.lock().unwrap();
                            state.active = None;
                            state.active_revision = None;
                            state.diagnostics = engine.diagnostics();
                        }
                        state = lock.lock().unwrap();
                    }
                    if state.shutdown {
                        continue;
                    }
                    let _guard = wake.wait_timeout(state, Duration::from_millis(25)).unwrap();
                }
                let mut state = lock.lock().unwrap();
                state.active = None;
                state.active_revision = None;
                state.status = "stopped".into();
                state.retiring = false;
                state.finished = true;
                wake.notify_all();
            })
            .expect("start session owner");
        Self {
            shared,
            worker: Some(worker),
        }
    }
    pub fn play(&self, spec: SessionSpec) {
        let (mut state, wake) = (self.shared.0.lock().unwrap(), &self.shared.1);
        if let Some(old) = state.request.as_ref() {
            old.cancel.store(true, Ordering::Release);
        }
        if let Some(engine) = state.active.as_ref() {
            engine.request_stop();
        }
        state.revision += 1;
        state.request = Some(spec);
        state.diagnostics = Diagnostics::default();
        state.status = "connecting".into();
        state.error = None;
        wake.notify_all();
    }
    pub fn stop(&self) {
        let mut state = self.shared.0.lock().unwrap();
        if let Some(spec) = state.request.take() {
            spec.cancel.store(true, Ordering::Release);
        }
        if let Some(engine) = state.active.as_ref() {
            engine.request_stop();
        }
        state.revision += 1;
        state.status = "stopped".into();
        state.error = None;
        self.shared.1.notify_all();
    }
    pub fn volume(&self, value: u8) {
        let mut state = self.shared.0.lock().unwrap();
        state.volume = value;
        if let Some(engine) = &state.active {
            engine.set_volume(value);
        }
    }
    pub fn muted(&self, value: bool) {
        let mut state = self.shared.0.lock().unwrap();
        state.muted = value;
        if let Some(engine) = &state.active {
            engine.set_muted(value);
        }
    }
    pub fn eq(&self, value: EqSettings) {
        let mut state = self.shared.0.lock().unwrap();
        state.eq = value.clone();
        if let Some(engine) = &state.active {
            engine.set_eq_settings(value);
        }
    }
    pub fn status(&self) -> (String, Option<String>) {
        let state = self.shared.0.lock().unwrap();
        (state.status.clone(), state.error.clone())
    }
    pub fn spectrum(&self) -> [f32; 5] {
        let engine = self.shared.0.lock().unwrap().active.clone();
        engine.map_or([0.; 5], |e| e.get_spectrum_levels())
    }
    pub fn diagnostics(&self) -> SessionDiagnostics {
        let state = self.shared.0.lock().unwrap();
        SessionDiagnostics {
            engine: state
                .active
                .as_ref()
                .map_or_else(|| state.diagnostics.clone(), |e| e.diagnostics()),
            active: state.active.is_some(),
            requested_revision: state.revision,
            active_revision: state.active_revision,
            retiring: state.retiring,
            service_finished: state.finished,
            active_http_requests: crate::net::ACTIVE_REQUESTS.load(Ordering::Acquire),
        }
    }
    pub fn shutdown(&mut self) {
        self.stop();
        {
            let mut state = self.shared.0.lock().unwrap();
            state.shutdown = true;
            self.shared.1.notify_all();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
impl Default for LivePlayback {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for LivePlayback {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn worker_counts() -> serde_json::Value {
    serde_json::json!({"http":crate::net::ACTIVE_REQUESTS.load(Ordering::Acquire),"metadata":crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),"audio":crate::audio::engine::ACTIVE_AUDIO_WORKERS.load(Ordering::Acquire),"sessionOwners":ACTIVE_SESSION_OWNERS.load(Ordering::Acquire)})
}
