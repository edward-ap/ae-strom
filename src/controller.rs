use crate::{
    config,
    model::{Config, EqPreset, StationPreset},
    presets,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex, RwLock,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackMode {
    Fake,
    Live,
    #[cfg(all(test, feature = "live-audio"))]
    TestSink,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EqChoice {
    pub name: String,
    pub preamp: f32,
    pub bands: [f32; 10],
    pub builtin: bool,
}
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EqView {
    pub name: String,
    pub preamp: f32,
    pub bands: [f32; 10],
    pub dirty: bool,
    pub can_save: bool,
    pub can_delete: bool,
}
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub revision: u64,
    pub generation: u64,
    pub mode: String,
    pub status: String,
    pub requested_playing: bool,
    pub full_title: String,
    pub display_title: String,
    pub active_bank: usize,
    pub active_slot: usize,
    pub volume: u8,
    pub muted: bool,
    pub stations: Vec<StationPreset>,
    pub eq_presets: Vec<EqChoice>,
    pub eq: EqView,
    pub error: Option<String>,
}
#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Spectrum {
    pub generation: u64,
    pub levels: [f32; 5],
    pub powered: bool,
}
#[derive(Clone, Debug)]
pub enum Update {
    State(Box<Snapshot>),
    Spectrum(Spectrum),
}
pub type Sink = Arc<dyn Fn(Update) + Send + Sync>;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action {
    SetPlaying {
        playing: bool,
    },
    SetVolume {
        volume: i32,
    },
    SetMuted {
        muted: bool,
    },
    SelectStation {
        bank: usize,
        slot: usize,
    },
    SetStationUrl {
        bank: usize,
        slot: usize,
        url: String,
    },
    AssignStationEq {
        bank: usize,
        slot: usize,
        name: String,
    },
    SelectEqPreset {
        name: String,
    },
    SetEq {
        preamp: f32,
        bands: [f32; 10],
    },
    SaveEqPreset {
        name: String,
    },
    DeleteEqPreset {
        name: String,
    },
}
type Reply = SyncSender<Result<Snapshot, String>>;
#[cfg_attr(not(feature = "live-audio"), allow(dead_code))]
enum Message {
    Action(Action, Reply),
    Shutdown(SyncSender<Result<(), String>>),
    Position(i32, i32),
    Diagnostics(SyncSender<serde_json::Value>),
    Metadata {
        generation: u64,
        station: usize,
        title: String,
        name: Option<String>,
    },
    Strategy {
        generation: u64,
        station: usize,
        kind: String,
        url: String,
    },
}

#[derive(Clone)]
pub struct Controller {
    tx: SyncSender<Message>,
    current: Arc<RwLock<Snapshot>>,
    closing: Arc<AtomicBool>,
    worker: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}
impl Controller {
    pub fn start(path: PathBuf, mode: PlaybackMode, sink: Sink) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("Configuration path must be absolute".into());
        }
        let mut cfg =
            config::load_or_default(&path).map_err(|e| format!("Isolated configuration: {e}"))?;
        cfg.volume = cfg.volume.min(100);
        cfg.last_bank = cfg.last_bank.min(25);
        cfg.last_preset = cfg.last_preset.min(9);
        cfg.presets.resize_with(260, empty_station);
        cfg.presets.truncate(260);
        let backend = Backend::new(mode)?;
        let (tx, rx) = mpsc::sync_channel(256);
        let selected = resolve(&cfg, &cfg.presets[cfg.last_bank * 10 + cfg.last_preset].eq)
            .unwrap_or_else(|_| presets::find_builtin_eq_preset("Off").unwrap().clone());
        let eq = eq_view(&cfg, &selected);
        let mut core = Core {
            cfg,
            path,
            backend,
            mode,
            generation: 0,
            revision: 0,
            requested: false,
            title: "Stopped".into(),
            status: "stopped".into(),
            error: None,
            storage_error: None,
            eq,
            cancellation: Arc::new(AtomicBool::new(true)),
            tx: tx.clone(),
            started: Instant::now(),
        };
        core.backend.volume(core.cfg.volume);
        core.backend.muted(core.cfg.muted);
        core.apply_eq();
        let current = Arc::new(RwLock::new(core.snapshot()));
        let shared = current.clone();
        let worker = thread::Builder::new()
            .name("radio-controller".into())
            .spawn(move || {
                let publish = |core: &Core| {
                    let snapshot = core.snapshot();
                    *shared.write().unwrap() = snapshot.clone();
                    sink(Update::State(Box::new(snapshot)));
                };
                let mut tick = Instant::now();
                loop {
                    match rx.recv_timeout(Duration::from_millis(if core.requested {
                        50
                    } else {
                        250
                    })) {
                        Ok(Message::Action(action, reply)) => {
                            let result = core.action(action);
                            core.revision += 1;
                            if let Err(ref e) = result {
                                core.error = Some(e.clone());
                            }
                            publish(&core);
                            let _ = reply.send(result.map(|_| core.snapshot()));
                        }
                        Ok(Message::Metadata {
                            generation,
                            station,
                            title,
                            name,
                        }) => {
                            if core.accepts(generation, station) {
                                if let Some(name) = name.filter(|n| !n.trim().is_empty()) {
                                    core.cfg.presets[station].name = name.trim().to_owned();
                                    if title.trim().is_empty() {
                                        core.title = name.trim().to_owned();
                                    }
                                }
                                let clean = clean_title(&title);
                                if !clean.is_empty() {
                                    core.title = clean;
                                }
                                core.persist();
                                core.revision += 1;
                                publish(&core);
                            }
                        }
                        Ok(Message::Strategy {
                            generation,
                            station,
                            kind,
                            url,
                        }) => {
                            if core.accepts(generation, station) {
                                core.cfg.presets[station].metadata_type = kind;
                                core.cfg.presets[station].metadata_url = url;
                                core.persist();
                                core.revision += 1;
                                publish(&core);
                            }
                        }
                        Ok(Message::Diagnostics(reply)) => {
                            let _ = reply.send(core.backend.diagnostics());
                        }
                        Ok(Message::Position(x, y)) => {
                            core.cfg.window_x = x;
                            core.cfg.window_y = y;
                            core.cfg.window_pos_valid = true;
                        }
                        Ok(Message::Shutdown(reply)) => {
                            core.cancel();
                            core.backend.stop();
                            core.requested = false;
                            core.status = "stopped".into();
                            core.title = "Stopped".into();
                            let result =
                                config::save(&core.path, &core.cfg).map_err(|e| e.to_string());
                            core.storage_error = result.as_ref().err().cloned();
                            core.revision += 1;
                            publish(&core);
                            let finished = result.is_ok();
                            if finished {
                                core.backend.shutdown();
                            }
                            let _ = reply.send(result);
                            if finished {
                                break;
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            core.cancel();
                            core.backend.stop();
                            break;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                    if tick.elapsed() >= Duration::from_millis(50) {
                        if core.tick() {
                            core.revision += 1;
                            publish(&core);
                        }
                        if core.requested {
                            sink(Update::Spectrum(core.spectrum()));
                        }
                        tick = Instant::now();
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            tx,
            current,
            closing: Arc::new(AtomicBool::new(false)),
            worker: Arc::new(Mutex::new(Some(worker))),
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        self.current.read().unwrap().clone()
    }
    /// Enqueue only; waiting happens in spawn_blocking, never the native UI loop.
    pub fn submit(&self, action: Action) -> Result<Receiver<Result<Snapshot, String>>, String> {
        if self.closing.load(Ordering::Acquire) {
            return Err("Player is shutting down".into());
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .try_send(Message::Action(action, tx))
            .map_err(|_| "Controller queue unavailable".to_owned())?;
        Ok(rx)
    }
    pub fn diagnostics(&self) -> Result<serde_json::Value, String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .try_send(Message::Diagnostics(tx))
            .map_err(|_| "Controller queue unavailable")?;
        rx.recv_timeout(Duration::from_secs(2))
            .map_err(|_| "Diagnostics timed out".into())
    }
    pub fn position(&self, x: i32, y: i32) {
        let _ = self.tx.try_send(Message::Position(x, y));
    }
    pub fn shutdown(&self) -> Result<(), String> {
        if self.closing.swap(true, Ordering::AcqRel) {
            return Err("Shutdown is already in progress or complete".into());
        }
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .send(Message::Shutdown(tx))
            .map_err(|_| "Controller unavailable")?;
        let result = rx.recv().map_err(|_| "Shutdown interrupted")?;
        if result.is_err() {
            self.closing.store(false, Ordering::Release);
            return result;
        }
        if let Some(worker) = self.worker.lock().unwrap().take() {
            worker.join().map_err(|_| "Controller panicked")?;
        }
        result
    }
}

struct Core {
    cfg: Config,
    path: PathBuf,
    backend: Backend,
    mode: PlaybackMode,
    generation: u64,
    revision: u64,
    requested: bool,
    title: String,
    status: String,
    error: Option<String>,
    storage_error: Option<String>,
    eq: EqView,
    cancellation: Arc<AtomicBool>,
    #[cfg_attr(not(feature = "live-audio"), allow(dead_code))]
    tx: SyncSender<Message>,
    started: Instant,
}
impl Core {
    fn index(&self) -> usize {
        self.cfg.last_bank * 10 + self.cfg.last_preset
    }
    fn accepts(&self, generation: u64, station: usize) -> bool {
        self.requested
            && self.generation == generation
            && self.index() == station
            && !self.cancellation.load(Ordering::Acquire)
    }
    fn snapshot(&self) -> Snapshot {
        let mut display: String = self.title.chars().take(240).collect();
        if self.title.chars().count() > 240 {
            display.push_str("...");
        }
        Snapshot {
            revision: self.revision,
            generation: self.generation,
            mode: if self.mode == PlaybackMode::Fake {
                "fake"
            } else {
                "live"
            }
            .into(),
            status: self.status.clone(),
            requested_playing: self.requested,
            full_title: self.title.clone(),
            display_title: display,
            active_bank: self.cfg.last_bank,
            active_slot: self.cfg.last_preset,
            volume: self.cfg.volume,
            muted: self.cfg.muted,
            stations: self.cfg.presets.clone(),
            eq_presets: choices(&self.cfg),
            eq: self.eq.clone(),
            error: self.storage_error.clone().or_else(|| self.error.clone()),
        }
    }
    fn persist(&mut self) {
        self.storage_error = config::save(&self.path, &self.cfg)
            .err()
            .map(|e| format!("Settings were not saved: {e}"));
    }
    fn cancel(&mut self) {
        self.cancellation.store(true, Ordering::Release);
        self.generation += 1;
    }
    fn start(&mut self) -> Result<(), String> {
        self.cancel();
        self.backend.stop();
        self.cancellation = Arc::new(AtomicBool::new(false));
        self.error = None;
        self.requested = true;
        if self.mode == PlaybackMode::Fake {
            self.backend.play_fake();
            self.status = "playing".into();
            self.title = "Synthetic preview — no audio".into();
            return Ok(());
        }
        let idx = self.index();
        let station = self.cfg.presets[idx].clone();
        if station.url.is_empty() {
            self.requested = false;
            self.status = "stopped".into();
            self.title = "No stream URL".into();
            self.cancellation.store(true, Ordering::Release);
            return Err("No stream URL for the selected slot".into());
        }
        if let Err(error) = validate_url(&station.url) {
            self.requested = false;
            self.status = "error".into();
            self.cancellation.store(true, Ordering::Release);
            return Err(error);
        }
        self.status = "connecting".into();
        self.title = if station.name.trim().is_empty() {
            "Connecting...".into()
        } else {
            station.name.clone()
        };
        #[cfg(feature = "live-audio")]
        {
            let tx = self.tx.clone();
            let strategy_tx = self.tx.clone();
            let generation = self.generation;
            self.backend.play_live(crate::session::SessionSpec {
                url: station.url,
                hint: crate::metadata::StrategyHint {
                    r#type: station.metadata_type,
                    url: station.metadata_url,
                },
                poll: Duration::from_secs(self.cfg.json_poll_interval_sec.unwrap_or(5).max(5)),
                cancel: self.cancellation.clone(),
                on_info: Arc::new(move |info| {
                    let _ = tx.try_send(Message::Metadata {
                        generation,
                        station: idx,
                        title: info.title,
                        name: info.station_name,
                    });
                }),
                on_strategy: Arc::new(move |hint| {
                    let _ = strategy_tx.try_send(Message::Strategy {
                        generation,
                        station: idx,
                        kind: hint.r#type,
                        url: hint.url,
                    });
                }),
            });
        }
        Ok(())
    }
    fn action(&mut self, action: Action) -> Result<(), String> {
        self.error = None;
        match action {
            Action::SetPlaying { playing } => {
                if playing {
                    self.start()?;
                } else {
                    self.cancel();
                    self.requested = false;
                    self.backend.stop();
                    self.status = "stopped".into();
                    self.title = "Stopped".into();
                }
            }
            Action::SetVolume { volume } => {
                self.cfg.volume = volume.clamp(0, 100) as u8;
                self.backend.volume(self.cfg.volume);
            }
            Action::SetMuted { muted } => {
                self.cfg.muted = muted;
                self.backend.muted(muted);
            }
            Action::SelectStation { bank, slot } => {
                let target = index(bank, slot)?;
                let selected = resolve(&self.cfg, &self.cfg.presets[target].eq)?;
                self.cfg.last_bank = bank;
                self.cfg.last_preset = slot;
                self.eq = eq_view(&self.cfg, &selected);
                self.apply_eq();
                if self.requested {
                    self.start()?;
                }
            }
            Action::SetStationUrl { bank, slot, url } => {
                let idx = index(bank, slot)?;
                let url = url.trim().to_owned();
                if url.len() > 16384 {
                    return Err("URL is too long".into());
                }
                if !url.is_empty() {
                    validate_url(&url)?;
                }
                self.cfg.presets[idx].url = url;
                self.cfg.presets[idx].name.clear();
                self.cfg.presets[idx].metadata_type.clear();
                self.cfg.presets[idx].metadata_url.clear();
                if idx == self.index() && self.requested {
                    self.cancel();
                } // Do not restart live audio on text edit; reject old metadata.
            }
            Action::AssignStationEq { bank, slot, name } => {
                let idx = index(bank, slot)?;
                let preset = resolve(&self.cfg, &name)?;
                self.cfg.presets[idx].eq = preset.name.clone();
                if idx == self.index() {
                    self.eq = eq_view(&self.cfg, &preset);
                    self.apply_eq();
                }
            }
            Action::SelectEqPreset { name } => {
                let preset = resolve(&self.cfg, &name)?;
                let idx = self.index();
                self.cfg.presets[idx].eq = preset.name.clone();
                self.eq = eq_view(&self.cfg, &preset);
                self.apply_eq();
            }
            Action::SetEq { preamp, bands } => {
                validate_eq(preamp, &bands)?;
                self.eq.preamp = preamp;
                self.eq.bands = bands;
                self.eq.dirty = true;
                self.eq.can_save = true;
                self.apply_eq();
                return Ok(());
            }
            Action::SaveEqPreset { name } => {
                let name = name.trim();
                config::validate_custom_eq_name(name)?;
                let preset = EqPreset {
                    name: name.into(),
                    preamp: self.eq.preamp,
                    bands: self.eq.bands,
                };
                validate_eq(preset.preamp, &preset.bands)?;
                crate::model::upsert_custom_eq_preset(&mut self.cfg, preset.clone());
                let idx = self.index();
                self.cfg.presets[idx].eq = preset.name.clone();
                self.eq = eq_view(&self.cfg, &preset);
            }
            Action::DeleteEqPreset { name } => {
                if is_builtin(&name)
                    || !self
                        .cfg
                        .custom_eq_presets
                        .iter()
                        .any(|p| p.name.eq_ignore_ascii_case(&name))
                {
                    return Err("Only custom presets can be deleted".into());
                }
                crate::model::delete_custom_eq_preset(&mut self.cfg, &name);
                let active = resolve(&self.cfg, &self.cfg.presets[self.index()].eq)?;
                self.eq = eq_view(&self.cfg, &active);
                self.apply_eq();
            }
        }
        self.persist();
        Ok(())
    }
    fn apply_eq(&mut self) {
        self.backend.eq(self.eq.preamp, self.eq.bands);
    }
    fn tick(&mut self) -> bool {
        if !self.requested {
            return false;
        }
        let (status, error) = self.backend.status();
        let changed = self.status != status || self.error != error;
        self.status = status;
        self.error = error;
        if self.status == "error" {
            self.requested = false;
            self.cancel();
        }
        changed
    }
    fn spectrum(&self) -> Spectrum {
        let levels = if self.requested && self.mode == PlaybackMode::Fake {
            let t = self.started.elapsed().as_secs_f32();
            std::array::from_fn(|i| ((t * 2.1 + i as f32 * 0.7).sin() * 0.35 + 0.45).clamp(0., 1.))
        } else {
            self.backend.spectrum()
        };
        Spectrum {
            generation: self.generation,
            levels,
            powered: self.requested && self.status == "playing",
        }
    }
}

enum Backend {
    Fake {
        playing: bool,
    },
    #[cfg(feature = "live-audio")]
    Real(crate::session::LivePlayback),
}
impl Backend {
    fn new(mode: PlaybackMode) -> Result<Self, String> {
        match mode {
            PlaybackMode::Fake => Ok(Self::Fake { playing: false }),
            #[cfg(all(test, feature = "live-audio"))]
            PlaybackMode::TestSink => Ok(Self::Real(crate::session::LivePlayback::new_sink())),
            PlaybackMode::Live => {
                #[cfg(feature = "live-audio")]
                {
                    Ok(Self::Real(crate::session::LivePlayback::new()))
                }
                #[cfg(not(feature = "live-audio"))]
                {
                    Err("This UI-shell build has no live-audio feature".into())
                }
            }
        }
    }
    fn play_fake(&mut self) {
        if let Self::Fake { playing } = self {
            *playing = true;
        }
    }
    #[cfg(feature = "live-audio")]
    fn play_live(&self, spec: crate::session::SessionSpec) {
        if let Self::Real(service) = self {
            service.play(spec);
        }
    }
    fn shutdown(&mut self) {
        #[cfg(feature = "live-audio")]
        if let Self::Real(service) = self {
            service.shutdown();
        }
    }
    fn diagnostics(&self) -> serde_json::Value {
        match self {
            Self::Fake { playing } => serde_json::json!({"fake":true,"playing":playing}),
            #[cfg(feature = "live-audio")]
            Self::Real(service) => serde_json::to_value(service.diagnostics()).unwrap(),
        }
    }
    fn stop(&mut self) {
        match self {
            Self::Fake { playing } => *playing = false,
            #[cfg(feature = "live-audio")]
            Self::Real(e) => e.stop(),
        }
    }
    fn volume(&self, _value: u8) {
        #[cfg(feature = "live-audio")]
        if let Self::Real(e) = self {
            e.volume(_value)
        }
    }
    fn muted(&self, _value: bool) {
        #[cfg(feature = "live-audio")]
        if let Self::Real(e) = self {
            e.muted(_value)
        }
    }
    fn eq(&self, _preamp: f32, _bands: [f32; 10]) {
        #[cfg(feature = "live-audio")]
        if let Self::Real(e) = self {
            e.eq(crate::audio::dsp::EqSettings {
                preamp: _preamp,
                bands: _bands,
            })
        }
    }
    fn spectrum(&self) -> [f32; 5] {
        match self {
            Self::Fake { .. } => [0.; 5],
            #[cfg(feature = "live-audio")]
            Self::Real(e) => e.spectrum(),
        }
    }
    fn status(&self) -> (String, Option<String>) {
        match self {
            Self::Fake { playing } => ((if *playing { "playing" } else { "stopped" }).into(), None),
            #[cfg(feature = "live-audio")]
            Self::Real(e) => e.status(),
        }
    }
}
fn empty_station() -> StationPreset {
    StationPreset {
        name: String::new(),
        url: String::new(),
        eq: "Off".into(),
        metadata_type: String::new(),
        metadata_url: String::new(),
    }
}
fn index(bank: usize, slot: usize) -> Result<usize, String> {
    if bank < 26 && slot < 10 {
        Ok(bank * 10 + slot)
    } else {
        Err("Bank/slot is out of range".into())
    }
}
fn validate_url(url: &str) -> Result<(), String> {
    config::validate_stream_url(url)
}

fn validate_eq(preamp: f32, bands: &[f32; 10]) -> Result<(), String> {
    if std::iter::once(&preamp)
        .chain(bands.iter())
        .all(|v| v.is_finite() && (-15.0..=15.0).contains(v))
    {
        Ok(())
    } else {
        Err("EQ values must be finite and between -15 and +15 dB".into())
    }
}
fn resolve(cfg: &Config, name: &str) -> Result<EqPreset, String> {
    presets::BUILTIN_EQ_PRESETS
        .iter()
        .chain(cfg.custom_eq_presets.iter())
        .find(|p| p.name.eq_ignore_ascii_case(name))
        .cloned()
        .ok_or_else(|| format!("Unknown EQ preset: {name}"))
}
fn eq_view(cfg: &Config, preset: &EqPreset) -> EqView {
    EqView {
        name: preset.name.clone(),
        preamp: preset.preamp,
        bands: preset.bands,
        dirty: false,
        can_save: false,
        can_delete: cfg
            .custom_eq_presets
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(&preset.name))
            && !is_builtin(&preset.name),
    }
}
fn choices(cfg: &Config) -> Vec<EqChoice> {
    presets::BUILTIN_EQ_PRESETS
        .iter()
        .map(|p| (p, true))
        .chain(cfg.custom_eq_presets.iter().map(|p| (p, false)))
        .map(|(p, builtin)| EqChoice {
            name: p.name.clone(),
            preamp: p.preamp,
            bands: p.bands,
            builtin,
        })
        .collect()
}
fn clean_title(raw: &str) -> String {
    let t = raw.trim();
    if t == "-" {
        String::new()
    } else {
        t.replace("&amp;", "&")
            .replace("&apos;", "'")
            .replace("&quot;", "\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Controller, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "mini-radio-controller-{}-{}",
            std::process::id(),
            SystemTimeTag::next()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let c = Controller::start(
            dir.join("config.json"),
            PlaybackMode::Fake,
            Arc::new(|_| {}),
        )
        .unwrap();
        (c, dir)
    }
    struct SystemTimeTag;
    impl SystemTimeTag {
        fn next() -> u128 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        }
    }
    fn act(c: &Controller, a: Action) -> Snapshot {
        c.submit(a)
            .unwrap()
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap()
    }
    #[test]
    fn fake_actions_preserve_stopped_selection_and_persist_mute() {
        let (c, dir) = fixture();
        let s = act(&c, Action::SelectStation { bank: 25, slot: 9 });
        assert!(!s.requested_playing);
        assert_eq!((s.active_bank, s.active_slot), (25, 9));
        act(&c, Action::SetMuted { muted: true });
        act(&c, Action::SetVolume { volume: 999 });
        c.shutdown().unwrap();
        let cfg = config::load_or_default(&dir.join("config.json")).unwrap();
        assert!(cfg.muted);
        assert_eq!(cfg.volume, 100);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn stale_metadata_cannot_change_new_generation_or_stopped_state() {
        let (c, dir) = fixture();
        let first = act(&c, Action::SetPlaying { playing: true });
        act(&c, Action::SelectStation { bank: 1, slot: 0 });
        let current = act(&c, Action::SelectStation { bank: 0, slot: 0 });
        assert!(current.generation > first.generation);
        c.tx.send(Message::Metadata {
            generation: first.generation,
            station: 0,
            title: "stale".into(),
            name: Some("wrong".into()),
        })
        .unwrap();
        let s = act(&c, Action::SetVolume { volume: 45 });
        assert_ne!(s.full_title, "stale");
        assert!(s.stations[0].name.is_empty());
        act(&c, Action::SetPlaying { playing: false });
        c.tx.send(Message::Strategy {
            generation: s.generation,
            station: 0,
            kind: "ICY".into(),
            url: "https://example.invalid/stale".into(),
        })
        .unwrap();
        let s = act(&c, Action::SetMuted { muted: true });
        assert!(s.stations[0].metadata_url.is_empty());
        c.shutdown().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn custom_eq_delete_resets_all_bank_references() {
        let (c, dir) = fixture();
        act(
            &c,
            Action::SetEq {
                preamp: -2.,
                bands: [1.; 10],
            },
        );
        act(
            &c,
            Action::SaveEqPreset {
                name: "Desk".into(),
            },
        );
        act(
            &c,
            Action::AssignStationEq {
                bank: 25,
                slot: 9,
                name: "Desk".into(),
            },
        );
        let s = act(
            &c,
            Action::DeleteEqPreset {
                name: "Desk".into(),
            },
        );
        assert_eq!(s.stations[0].eq, "Off");
        assert_eq!(s.stations[259].eq, "Off");
        assert!(!s.eq_presets.iter().any(|p| p.name == "Desk"));
        assert!(c
            .submit(Action::SelectStation { bank: 26, slot: 0 })
            .unwrap()
            .recv()
            .unwrap()
            .is_err());
        c.shutdown().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn queue_returns_without_waiting_and_shutdown_rejects_new_actions() {
        let (c, dir) = fixture();
        let response = c.submit(Action::SetPlaying { playing: true }).unwrap();
        assert!(response
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .is_ok());
        c.shutdown().unwrap();
        assert!(c.submit(Action::SetMuted { muted: true }).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod continuity_tests {
    use super::*;
    fn fixture() -> (Controller, PathBuf) {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("radio-continuity-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        (
            Controller::start(
                dir.join("config.json"),
                PlaybackMode::Fake,
                Arc::new(|_| {}),
            )
            .unwrap(),
            dir,
        )
    }
    fn act(controller: &Controller, action: Action) -> Snapshot {
        controller
            .submit(action)
            .unwrap()
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap()
    }
    #[test]
    fn same_slot_restart_rejects_old_generation_and_retains_full_copy_title() {
        let (controller, dir) = fixture();
        let old = act(&controller, Action::SetPlaying { playing: true });
        act(&controller, Action::SetPlaying { playing: false });
        let new = act(&controller, Action::SetPlaying { playing: true });
        controller
            .tx
            .send(Message::Metadata {
                generation: old.generation,
                station: 0,
                title: "old session".into(),
                name: None,
            })
            .unwrap();
        let title = "long title ".repeat(40);
        controller
            .tx
            .send(Message::Metadata {
                generation: new.generation,
                station: 0,
                title: title.clone(),
                name: None,
            })
            .unwrap();
        let current = act(&controller, Action::SetVolume { volume: 50 });
        assert_eq!(current.full_title, title.trim());
        assert_eq!(current.display_title.chars().count(), 243);
        assert!(current.requested_playing);
        controller.shutdown().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn url_commit_cancels_old_metadata_without_restarting_playback() {
        let (controller, dir) = fixture();
        let old = act(&controller, Action::SetPlaying { playing: true });
        let changed = act(
            &controller,
            Action::SetStationUrl {
                bank: 0,
                slot: 0,
                url: "https://example.invalid/new".into(),
            },
        );
        assert!(changed.requested_playing);
        assert!(changed.generation > old.generation);
        controller
            .tx
            .send(Message::Strategy {
                generation: old.generation,
                station: 0,
                kind: "ICY".into(),
                url: "https://example.invalid/old".into(),
            })
            .unwrap();
        let current = act(&controller, Action::SetMuted { muted: true });
        assert!(current.stations[0].metadata_url.is_empty());
        assert_eq!(current.stations[0].url, "https://example.invalid/new");
        assert!(current.requested_playing && current.muted);
        controller.shutdown().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn malformed_action_and_nonfinite_eq_are_rejected() {
        assert!(serde_json::from_value::<Action>(
            serde_json::json!({"type":"setPlaying","playing":true,"extra":1})
        )
        .is_err());
        assert!(serde_json::from_value::<Action>(
            serde_json::json!({"type":"setEq","preamp":0,"bands":[0,0]})
        )
        .is_err());
        let (controller, dir) = fixture();
        assert!(controller
            .submit(Action::SetEq {
                preamp: f32::NAN,
                bands: [0.; 10]
            })
            .unwrap()
            .recv()
            .unwrap()
            .is_err());
        assert_eq!(controller.snapshot().eq.preamp, 0.);
        controller.shutdown().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
}

fn is_builtin(name: &str) -> bool {
    presets::BUILTIN_EQ_PRESETS
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod shutdown_failure_test {
    use super::*;
    #[test]
    fn failed_flush_keeps_controller_available_and_can_be_retried() {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("radio-shutdown-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.json");
        let c = Controller::start(path.clone(), PlaybackMode::Fake, Arc::new(|_| {})).unwrap();
        let act = |a| {
            c.submit(a)
                .unwrap()
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .unwrap()
        };
        act(Action::SetMuted { muted: true });
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        let failed = act(Action::SetPlaying { playing: true });
        assert!(failed.error.is_some());
        assert!(c.shutdown().is_err());
        assert!(c.snapshot().error.is_some());
        std::fs::remove_dir(&path).unwrap();
        let recovered = act(Action::SetVolume { volume: 42 });
        assert!(recovered.error.is_none());
        c.shutdown().unwrap();
        assert_eq!(config::load_or_default(&path).unwrap().volume, 42);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
