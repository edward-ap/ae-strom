use super::dsp::{apply_volume, EqRuntimeState, EqSettings, SpectrumAnalyzer};
use cpal::traits::StreamTrait;
use ringbuf::{HeapConsumer, HeapProducer, HeapRb};
use rubato::{FftFixedInOut, Resampler};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::TimeBase;
use thiserror::Error;

mod adts_clock;
mod container_metadata;
use container_metadata::ContainerMetadata;
mod timing;
pub use crate::net::HttpBufferStats;
pub(crate) use adts_clock::AdtsClock;
pub use adts_clock::AdtsStats;
pub use timing::PlaybackTiming;

/// Header capability and metadata events from the actual audio HTTP connection.
#[derive(Clone, Debug)]
pub enum StreamMetadataEvent {
    Connected {
        request_id: u64,
        icy: bool,
    },
    Info {
        request_id: u64,
        info: crate::metadata::Info,
    },
    /// Initial result after probing (including no title), or a later revision.
    /// has_title describes availability, not whether this revision changes it.
    Container {
        request_id: u64,
        info: crate::metadata::Info,
        has_title: bool,
    },
}
pub type StreamMetadataSink = Arc<dyn Fn(StreamMetadataEvent) + Send + Sync>;

// Global master attenuation to reduce overall loudness independently of UI slider.
// 0.7 means ~30% quieter output compared to previous builds.
const MASTER_ATTENUATION: f32 = 0.5;
pub static ACTIVE_AUDIO_WORKERS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
const BASE_FRAME_SIZE: usize = 512;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Thin wrapper to adapt a streaming reader into Symphonia's MediaSource.
struct HttpSource<R: Read + Send> {
    reader: R,
    adts: Option<AdtsClock>,
    inner: Arc<Mutex<InnerState>>,
    request_id: u64,
}

impl<R: Read + Send> Read for HttpSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.reader.read(buf)?;
        if let Some(clock) = &mut self.adts {
            let before = (clock.stats.sample_rate, clock.stats.format_changes);
            clock.feed(&buf[..read]);
            {
                let mut state = self.inner.lock().unwrap();
                if state.diagnostics.timing.request_id == self.request_id {
                    state.diagnostics.timing.adts = clock.stats.clone();
                }
            }
            if before != (clock.stats.sample_rate, clock.stats.format_changes) {
                log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"adts_format","requestId":self.request_id,"adts":clock.stats}));
            }
        }
        Ok(read)
    }
}

impl<R: Read + Send> Seek for HttpSource<R> {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "seeking not supported for HTTP stream",
        ))
    }
}

unsafe impl<R: Read + Send> Sync for HttpSource<R> {}
impl<R: Read + Send> symphonia::core::io::MediaSource for HttpSource<R> {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

#[derive(Error, Debug)]
/// Errors that can arise during playback setup/streaming.
enum EngineError {
    #[error("[audio] http: {0}")]
    Http(String),
    #[error("[audio] probe: {0}")]
    Probe(String),
    #[error("[audio] no default track")]
    NoTrack,
    #[error("[audio] decoder: {0}")]
    Decoder(String),
    #[error("[audio] no output device")]
    OutputDevice,
    #[error("[audio] output config: {0}")]
    OutputConfig(String),
    #[error("[audio] resampler: {0}")]
    Resampler(String),
    #[error("[audio] stream build: {0}")]
    StreamBuild(String),
    #[error("[audio] permanent HTTP error: {0}")]
    PermanentHttp(String),
}

/// Mutable playback state shared between UI callbacks and the playback thread.
struct InnerState {
    stop_flag: bool,
    handle: Option<thread::JoinHandle<()>>,
    volume: u8,
    muted: bool,
    target_eq: EqSettings,
    spectrum_analyzer: Option<SpectrumAnalyzer>,
    spectrum_levels: [f32; 5],
    is_playing: bool,
    status: String,
    last_error: Option<String>,
    output_failed: bool,
    cancel: Arc<AtomicBool>,
    headless: bool,
    diagnostics: Diagnostics,
    timing_started: Option<Instant>,
    timing_enabled: bool,
    http_timing: Option<crate::net::HttpBufferTiming>,
}

struct TimingClock(Arc<Mutex<InnerState>>);
impl Drop for TimingClock {
    fn drop(&mut self) {
        let (mut timing, http) = {
            let mut state = self.0.lock().unwrap();
            if let Some(started) = state.timing_started.take() {
                state.diagnostics.timing.wall_seconds = started.elapsed().as_secs_f64();
            }
            state.diagnostics.timing.active = false;
            (state.diagnostics.timing.clone(), state.http_timing.clone())
        };
        if let Some(http) = http {
            timing.http = http.snapshot();
        }
        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"audio_timing_end","timing":timing}));
    }
}

#[derive(Clone, Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub connections: u64,
    pub decoded_samples: u64,
    pub output_callbacks: u64,
    pub output_samples: u64,
    pub non_silent_samples: u64,
    pub peak: f32,
    pub headless: bool,
    pub timing: PlaybackTiming,
}

/// High-level audio controller that manages playback and shared state.
#[derive(Clone)]
pub struct AudioEngine {
    inner: Arc<Mutex<InnerState>>,
}

/// Per-playback resources created when starting a stream.
struct PlaybackContext {
    format: Box<dyn FormatReader>,
    container_metadata: ContainerMetadata,
    decoder: Option<Box<dyn Decoder>>,
    track_id: u32,
    in_channels: usize,
    out_sample_rate: usize,
    out_channels: usize,
    stream_config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
    resampler: Option<FftFixedInOut<f32>>,
    ring_producer: HeapProducer<f32>,
    ring_consumer: Option<HeapConsumer<f32>>,
    eq_state: EqRuntimeState,
    spectrum: Option<SpectrumAnalyzer>,
    needs_channel_map: bool,
    current_in_rate: usize,
    in_channel_buffers: Vec<Vec<f32>>,
    output_device: Option<cpal::Device>,
    headless: bool,
    packet_time_base: Option<TimeBase>,
    collect_timing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaybackLoopExit {
    Stopped,
    Reconnect,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerState {
                stop_flag: false,
                handle: None,
                volume: 50,
                muted: false,
                target_eq: EqSettings::default(),
                spectrum_analyzer: None,
                spectrum_levels: [0.0; 5],
                is_playing: false,
                status: "stopped".into(),
                last_error: None,
                output_failed: false,
                cancel: Arc::new(AtomicBool::new(false)),
                headless: false,
                diagnostics: Diagnostics::default(),
                timing_started: None,
                timing_enabled: false,
                http_timing: None,
            })),
        }
    }
}

impl AudioEngine {
    fn rebuild_decoder_for_current_stream(
        inner: &Arc<Mutex<InnerState>>,
        ctx: &mut PlaybackContext,
    ) -> Result<(), EngineError> {
        let codecs = symphonia::default::get_codecs();
        let previous_track_id = ctx.track_id;

        let track = ctx
            .format
            .default_track()
            .filter(|t| codecs.get_codec(t.codec_params.codec).is_some())
            .or_else(|| {
                ctx.format.tracks().iter().find(|t| {
                    t.id == previous_track_id && codecs.get_codec(t.codec_params.codec).is_some()
                })
            })
            .or_else(|| {
                ctx.format
                    .tracks()
                    .iter()
                    .find(|t| codecs.get_codec(t.codec_params.codec).is_some())
            })
            .ok_or(EngineError::NoTrack)?;

        let decoder = codecs
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| EngineError::Decoder(e.to_string()))?;

        let in_sample_rate = track.codec_params.sample_rate.unwrap_or(48_000) as usize;
        let in_channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2) as usize;
        ctx.packet_time_base = track.codec_params.time_base;
        {
            let mut state = inner.lock().unwrap();
            let timing = &mut state.diagnostics.timing;
            timing.codec = codecs
                .get_codec(track.codec_params.codec)
                .map_or("unknown", |c| c.short_name)
                .into();
            timing.declared_sample_rate = in_sample_rate as u32;
            timing.time_base_numer = ctx.packet_time_base.map(|tb| tb.numer);
            timing.time_base_denom = ctx.packet_time_base.map(|tb| tb.denom);
            timing.expected_ts = None;
        }

        ctx.decoder = Some(decoder);
        ctx.track_id = track.id;
        ctx.in_channels = in_channels;
        ctx.current_in_rate = in_sample_rate;
        ctx.needs_channel_map = in_channels != ctx.out_channels;
        ctx.in_channel_buffers = vec![Vec::new(); in_channels];
        ctx.resampler = if in_sample_rate != ctx.out_sample_rate {
            Some(
                FftFixedInOut::<f32>::new(
                    in_sample_rate,
                    ctx.out_sample_rate,
                    BASE_FRAME_SIZE,
                    in_channels,
                )
                .map_err(|e| EngineError::Resampler(e.to_string()))?,
            )
        } else {
            None
        };

        let eq_target = {
            let st = inner.lock().unwrap();
            st.target_eq.clone()
        };
        ctx.eq_state = EqRuntimeState::new(eq_target, ctx.out_channels, ctx.out_sample_rate as f32);

        Ok(())
    }

    /// Create a new audio engine instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opt-in passive clock diagnostics for the console capture client.
    pub fn new_with_timing() -> Self {
        let engine = Self::new();
        engine.inner.lock().unwrap().timing_enabled = true;
        engine
    }

    /// Test-only PCM sink: never enumerates or opens an output device.
    #[cfg(test)]
    pub fn new_sink() -> Self {
        let engine = Self::new_with_timing();
        {
            let mut state = engine.inner.lock().unwrap();
            state.headless = true;
            state.diagnostics.headless = true;
        }
        engine
    }
    pub fn diagnostics(&self) -> Diagnostics {
        let state = self.inner.lock().unwrap();
        let mut diagnostics = state.diagnostics.clone();
        if let Some(started) = state.timing_started {
            diagnostics.timing.wall_seconds = started.elapsed().as_secs_f64();
        }
        let http = state.http_timing.clone();
        drop(state);
        // Capture ingress after downstream counters: received prefixes cannot
        // precede data already handed to the decoder. No nested engine/HTTP lock.
        if let Some(http) = http {
            diagnostics.timing.http = http.snapshot();
        }
        diagnostics
    }
    pub fn request_stop(&self) {
        let mut state = self.inner.lock().unwrap();
        state.stop_flag = true;
        state.cancel.store(true, Ordering::Release);
    }
    pub fn is_finished(&self) -> bool {
        self.inner
            .lock()
            .unwrap()
            .handle
            .as_ref()
            .is_none_or(|h| h.is_finished())
    }

    /// Set volume (0-100) to be applied on the playback thread.
    pub fn set_volume(&self, volume: u8) {
        let mut state = self.inner.lock().unwrap();
        state.volume = volume;
    }

    /// Mute/unmute playback.
    pub fn set_muted(&self, muted: bool) {
        let mut state = self.inner.lock().unwrap();
        state.muted = muted;
    }

    /// Latest spectrum levels used by the UI.
    pub fn get_spectrum_levels(&self) -> [f32; 5] {
        let state = self.inner.lock().unwrap();
        state.spectrum_levels
    }

    /// Whether playback thread is currently active.
    pub fn is_playing(&self) -> bool {
        let state = self.inner.lock().unwrap();
        state.is_playing
    }

    /// Observable connection state for the actor; does not touch the output device.
    pub fn playback_status(&self) -> (String, Option<String>) {
        let state = self.inner.lock().unwrap();
        (state.status.clone(), state.last_error.clone())
    }

    /// Apply EQ settings from UI/state.
    pub fn set_eq_settings(&self, settings: EqSettings) {
        let mut state = self.inner.lock().unwrap();
        state.target_eq = settings.clone();
    }

    /// Start playback for a given URL (spawns worker thread).
    pub fn play_url(&self, url: String) {
        self.start_playback_thread(url, None);
    }

    pub fn play_url_with_metadata(&self, url: String, on_metadata: StreamMetadataSink) {
        self.start_playback_thread(url, Some(on_metadata));
    }

    /// Stop playback and join worker thread.
    pub fn stop(&self) {
        self.stop_playback();
    }

    fn stop_playback(&self) {
        let mut state = self.inner.lock().unwrap();
        state.stop_flag = true;
        state.cancel.store(true, Ordering::Release);
        if let Some(handle) = state.handle.take() {
            drop(state);
            let _ = handle.join();
            let mut state = self.inner.lock().unwrap();
            state.stop_flag = false;
            state.status = "stopped".into();
            state.last_error = None;
            state.is_playing = false;
            state.spectrum_analyzer = None;
            state.spectrum_levels = [0.0; 5];
        } else {
            state.status = "stopped".into();
            state.last_error = None;
            state.is_playing = false;
            state.spectrum_analyzer = None;
            state.spectrum_levels = [0.0; 5];
        }
    }

    fn start_playback_thread(&self, url: String, on_metadata: Option<StreamMetadataSink>) {
        self.stop_playback();
        {
            let mut state = self.inner.lock().unwrap();
            state.stop_flag = false;
            state.cancel = Arc::new(AtomicBool::new(false));
            state.status = "connecting".into();
            state.last_error = None;
            state.output_failed = false;
        }

        let inner_arc = self.inner.clone();

        let handle = thread::spawn(move || {
            ACTIVE_AUDIO_WORKERS.fetch_add(1, Ordering::AcqRel);
            struct Count;
            impl Drop for Count {
                fn drop(&mut self) {
                    ACTIVE_AUDIO_WORKERS.fetch_sub(1, Ordering::AcqRel);
                }
            }
            let _count = Count;
            loop {
                {
                    let state = inner_arc.lock().unwrap();
                    if state.stop_flag {
                        break;
                    }
                }

                let mut ctx =
                    match Self::build_playback_context(&inner_arc, &url, on_metadata.clone()) {
                        Ok(c) => c,
                        Err(e) => {
                            log::warn!("{e}");
                            if matches!(
                                e,
                                EngineError::PermanentHttp(_)
                                    | EngineError::NoTrack
                                    | EngineError::Decoder(_)
                            ) {
                                let mut state = inner_arc.lock().unwrap();
                                state.status = "error".into();
                                state.last_error =
                                    Some("Cannot open the stream (HTTP or codec error)".into());
                                break;
                            }
                            if !Self::wait_for_reconnect_or_stop(&inner_arc) {
                                break;
                            }
                            continue;
                        }
                    };

                if inner_arc.lock().unwrap().stop_flag {
                    break;
                }
                // Reset before any output callback can run, never on loop entry.
                inner_arc.lock().unwrap().output_failed = false;
                let stream = if ctx.headless {
                    None
                } else {
                    let stream = match Self::build_output_stream(&inner_arc, &mut ctx) {
                        Ok(stream) => stream,
                        Err(e) => {
                            log::warn!("{e}");
                            if !Self::wait_for_reconnect_or_stop(&inner_arc) {
                                break;
                            }
                            continue;
                        }
                    };
                    if stream.play().is_err() {
                        if !Self::wait_for_reconnect_or_stop(&inner_arc) {
                            break;
                        }
                        continue;
                    }
                    Some(stream)
                };

                match Self::playback_loop(inner_arc.clone(), ctx, stream) {
                    PlaybackLoopExit::Stopped => break,
                    PlaybackLoopExit::Reconnect => {
                        log::warn!("[audio] stream interrupted, reconnecting");
                        if !Self::wait_for_reconnect_or_stop(&inner_arc) {
                            break;
                        }
                    }
                }
            }

            let mut st = inner_arc.lock().unwrap();
            st.is_playing = false;
            if st.status != "error" {
                st.status = "stopped".into();
            }
            st.spectrum_analyzer = None;
            st.spectrum_levels = [0.0; 5];
            st.stop_flag = false;
        });

        let mut state = self.inner.lock().unwrap();
        state.handle = Some(handle);
    }

    fn begin_loop(
        inner: &Arc<Mutex<InnerState>>,
        spectrum: Option<SpectrumAnalyzer>,
    ) -> Option<PlaybackLoopExit> {
        let mut state = inner.lock().unwrap();
        if state.stop_flag {
            return Some(PlaybackLoopExit::Stopped);
        }
        if state.output_failed {
            return Some(PlaybackLoopExit::Reconnect);
        }
        state.is_playing = true;
        state.status = "playing".into();
        state.last_error = None;
        state.spectrum_levels = [0.; 5];
        state.spectrum_analyzer = spectrum;
        if state.timing_enabled {
            state.timing_started = Some(Instant::now());
            state.diagnostics.timing.active = true;
        }
        None
    }
    fn interruption(inner: &Arc<Mutex<InnerState>>) -> Option<PlaybackLoopExit> {
        let state = inner.lock().unwrap();
        if state.stop_flag {
            Some(PlaybackLoopExit::Stopped)
        } else if state.output_failed {
            Some(PlaybackLoopExit::Reconnect)
        } else {
            None
        }
    }

    fn wait_for_reconnect_or_stop(inner: &Arc<Mutex<InnerState>>) -> bool {
        {
            let mut state = inner.lock().unwrap();
            state.status = "reconnecting".into();
            state.last_error = Some("Stream or output interrupted; retrying".into());
            state.is_playing = false;
            state.spectrum_levels = [0.0; 5];
        }
        let started = std::time::Instant::now();
        while started.elapsed() < RECONNECT_DELAY {
            {
                let state = inner.lock().unwrap();
                if state.stop_flag {
                    return false;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        true
    }
}

fn fill_output_from_ring_buffer(
    output: &mut [f32],
    inner: &Arc<Mutex<InnerState>>,
    consumer: &mut HeapConsumer<f32>,
) {
    let gain = {
        let state = inner.lock().unwrap();
        if state.muted {
            0.0
        } else {
            ((state.volume as f32) / 100.0) * MASTER_ATTENUATION
        }
    };

    let mut missing = 0;
    for sample in output.iter_mut() {
        if let Some(s) = consumer.pop() {
            *sample = s;
        } else {
            *sample = 0.0;
            missing += 1;
        }
    }

    apply_volume(output, gain);
    let mut state = inner.lock().unwrap();
    state.diagnostics.output_callbacks += 1;
    state.diagnostics.output_samples += output.len() as u64;
    if state.timing_enabled {
        state
            .diagnostics
            .timing
            .output(output.len(), missing, consumer.len());
    }
    state.diagnostics.non_silent_samples += output.iter().filter(|s| s.abs() > 1e-7).count() as u64;
    state.diagnostics.peak = output
        .iter()
        .fold(state.diagnostics.peak, |peak, s| peak.max(s.abs()));
}

impl AudioEngine {
    /// Build playback context by opening the stream URL and initializing audio components.
    ///
    /// Creates the decoder, resampler (if needed), ring buffer, EQ state, and spectrum analyzer.
    /// This method handles HTTP streaming, codec probing, and device configuration.
    ///
    /// # Errors
    /// Returns `EngineError` if HTTP request fails, codec probing fails, no audio track is found,
    /// decoder creation fails, no output device is available, or resampler initialization fails.
    fn build_playback_context(
        inner: &Arc<Mutex<InnerState>>,
        url: &str,
        on_metadata: Option<StreamMetadataSink>,
    ) -> Result<PlaybackContext, EngineError> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let cancel = {
            let mut state = inner.lock().unwrap();
            state.diagnostics.connections += 1;
            state.cancel.clone()
        };
        let collect_timing = inner.lock().unwrap().timing_enabled;
        let request = crate::net::get(url, cancel.clone()).set("Icy-MetaData", "1");
        let request = if collect_timing {
            request.with_buffer_timing()
        } else {
            request
        };
        let response = request.call().map_err(|e| {
            if e.kind() == std::io::ErrorKind::InvalidInput {
                EngineError::PermanentHttp(e.to_string())
            } else {
                EngineError::Http(e.to_string())
            }
        })?;
        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"audio_connection","requestId":response.request_id()}));
        let request_id = response.request_id();
        let metaint = crate::metadata::icy::interval(response.header("icy-metaint"))
            .map_err(|e| EngineError::Http(e.to_string()))?;
        let station_name = response
            .header("icy-name")
            .and_then(crate::metadata::direct::normalize_station_name);
        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"audio_metadata_source","requestId":request_id,"source":if metaint.is_some(){"in_band"}else{"container_probe"},"metaint":metaint}));
        if let Some(callback) = &on_metadata {
            if !cancel.load(Ordering::Acquire) {
                callback(StreamMetadataEvent::Connected {
                    request_id,
                    icy: metaint.is_some(),
                });
                if let Some(station_name) = station_name {
                    callback(StreamMetadataEvent::Info {
                        request_id,
                        info: crate::metadata::Info {
                            title: String::new(),
                            station_name: Some(station_name),
                        },
                    });
                }
            }
        }
        let observe_adts = collect_timing
            && response
                .header("content-type")
                .is_some_and(|ct| ct.to_ascii_lowercase().contains("aac"));
        {
            let mut state = inner.lock().unwrap();
            state.diagnostics.timing = PlaybackTiming {
                connection: state.diagnostics.connections,
                request_id,
                ..PlaybackTiming::default()
            };
            state.timing_started = None;
            state.http_timing = response.buffer_timing();
        }
        let icy_cancel = cancel.clone();
        let icy_callback = on_metadata.clone();
        let reader = crate::metadata::icy::IcyReader::new(
            response.into_reader(),
            metaint,
            request_id,
            Arc::new(move |info| {
                if !icy_cancel.load(Ordering::Acquire) {
                    if let Some(callback) = &icy_callback {
                        callback(StreamMetadataEvent::Info { request_id, info });
                    }
                }
            }),
        );
        let source = HttpSource {
            reader,
            adts: observe_adts.then(AdtsClock::default),
            inner: inner.clone(),
            request_id,
        };
        let mss = MediaSourceStream::new(Box::new(source), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("aac");
        hint.with_extension("adts");

        let mut probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| EngineError::Probe(e.to_string()))?;

        let mut format = probed.format;
        let container_metadata = ContainerMetadata::new(
            request_id,
            metaint.is_none(),
            on_metadata,
            cancel,
            probed.metadata.get(),
            format.metadata(),
        );

        let headless = inner.lock().unwrap().headless;
        let (output_device, stream_config, sample_format) = if headless {
            (
                None,
                cpal::StreamConfig {
                    channels: 2,
                    sample_rate: cpal::SampleRate(48_000),
                    buffer_size: cpal::BufferSize::Default,
                },
                cpal::SampleFormat::F32,
            )
        } else {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .ok_or(EngineError::OutputDevice)?;
            let config = device
                .default_output_config()
                .map_err(|e| EngineError::OutputConfig(e.to_string()))?;
            let format = config.sample_format();
            (Some(device), config.into(), format)
        };
        let out_sample_rate = stream_config.sample_rate.0 as usize;
        let out_channels = stream_config.channels as usize;
        {
            let mut state = inner.lock().unwrap();
            state.diagnostics.timing.output_sample_rate = out_sample_rate as u32;
            state.diagnostics.timing.output_channels = out_channels;
        }

        let rb = HeapRb::<f32>::new(out_sample_rate * out_channels as usize / 2);
        let (ring_producer, ring_consumer) = rb.split();

        let eq_target = {
            let st = inner.lock().unwrap();
            st.target_eq.clone()
        };

        let mut ctx = PlaybackContext {
            format,
            container_metadata,
            decoder: None,
            track_id: 0,
            in_channels: 0,
            out_sample_rate,
            out_channels,
            stream_config,
            sample_format,
            resampler: None,
            ring_producer,
            ring_consumer: Some(ring_consumer),
            eq_state: EqRuntimeState::new(eq_target, out_channels, out_sample_rate as f32),
            spectrum: Some(SpectrumAnalyzer::new(out_sample_rate as u32)),
            needs_channel_map: false,
            current_in_rate: out_sample_rate,
            in_channel_buffers: Vec::new(),
            output_device,
            headless,
            packet_time_base: None,
            collect_timing,
        };

        Self::rebuild_decoder_for_current_stream(inner, &mut ctx)?;
        let timing = inner.lock().unwrap().diagnostics.timing.clone();
        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"audio_format","timing":timing}));

        Ok(ctx)
    }

    /// Build the CPAL output stream for audio playback.
    ///
    /// Creates a hardware audio stream that pulls samples from the ring buffer
    /// and applies volume/mute settings. Currently only supports F32 sample format.
    ///
    /// # Errors
    /// Returns `EngineError` if ring consumer is missing, stream building fails,
    /// or sample format is unsupported.
    fn build_output_stream(
        inner: &Arc<Mutex<InnerState>>,
        ctx: &mut PlaybackContext,
    ) -> Result<cpal::Stream, EngineError> {
        use cpal::traits::DeviceTrait;

        let mut consumer = ctx
            .ring_consumer
            .take()
            .ok_or_else(|| EngineError::StreamBuild("ring consumer missing".into()))?;
        let inner_for_stream = inner.clone();
        let inner_for_error = inner.clone();
        let stream = match ctx.sample_format {
            cpal::SampleFormat::F32 => ctx
                .output_device
                .as_ref()
                .ok_or(EngineError::OutputDevice)?
                .build_output_stream(
                    &ctx.stream_config,
                    move |data: &mut [f32], _| {
                        fill_output_from_ring_buffer(data, &inner_for_stream, &mut consumer);
                    },
                    move |_| {
                        if let Ok(mut state) = inner_for_error.lock() {
                            state.output_failed = true;
                            state.last_error = Some("Audio output interrupted".into());
                        }
                    },
                    None,
                )
                .map_err(|e| EngineError::StreamBuild(e.to_string()))?,
            _ => {
                return Err(EngineError::StreamBuild("unsupported sample format".into()));
            }
        };

        Ok(stream)
    }

    /// Main playback loop that decodes audio packets and feeds the ring buffer.
    ///
    /// Continuously decodes audio packets from the stream, applies resampling if needed,
    /// handles channel mapping, applies EQ and spectrum analysis, and pushes samples
    /// to the ring buffer for output. Runs until stop flag is set or stream ends.
    ///
    /// The loop also smoothly transitions EQ settings when the target changes.
    #[allow(clippy::needless_range_loop)]
    fn playback_loop(
        inner: Arc<Mutex<InnerState>>,
        mut ctx: PlaybackContext,
        _stream: Option<cpal::Stream>,
    ) -> PlaybackLoopExit {
        use symphonia::core::audio::SampleBuffer;
        use symphonia::core::errors::Error as SymphError;

        let mut sample_buf: Option<SampleBuffer<f32>> = None;

        if let Some(exit) = Self::begin_loop(&inner, ctx.spectrum.take()) {
            return exit;
        }
        let _timing_clock = ctx.collect_timing.then(|| TimingClock(inner.clone()));

        loop {
            {
                let state = inner.lock().unwrap();
                if state.stop_flag {
                    return PlaybackLoopExit::Stopped;
                }
                if state.output_failed {
                    return PlaybackLoopExit::Reconnect;
                }
            }

            let next = ctx.format.next_packet();
            let reset = matches!(&next, Err(SymphError::ResetRequired));
            ctx.container_metadata.update(ctx.format.metadata(), reset);
            let packet = match next {
                Ok(p) => p,
                Err(SymphError::IoError(err)) => {
                    log::warn!("[audio] packet read failed: {err}");
                    return PlaybackLoopExit::Reconnect;
                }
                Err(SymphError::ResetRequired) => {
                    log::info!("[audio] stream reset requested, rebuilding decoder");
                    if let Err(err) = Self::rebuild_decoder_for_current_stream(&inner, &mut ctx) {
                        log::warn!("{err}");
                        return PlaybackLoopExit::Reconnect;
                    }
                    sample_buf = None;
                    continue;
                }
                Err(err) => {
                    log::warn!("[audio] packet read ended: {err}");
                    return PlaybackLoopExit::Reconnect;
                }
            };

            if packet.track_id() != ctx.track_id {
                continue;
            }
            if ctx.collect_timing {
                inner.lock().unwrap().diagnostics.timing.packet(
                    packet.ts(),
                    packet.dur(),
                    ctx.packet_time_base,
                );
            }

            let decoded = match ctx.decoder.as_mut().unwrap().decode(&packet) {
                Ok(a) => a,
                Err(SymphError::DecodeError(_)) => {
                    if ctx.collect_timing {
                        inner
                            .lock()
                            .unwrap()
                            .diagnostics
                            .timing
                            .decode_error_packets += 1;
                    }
                    continue;
                }
                Err(SymphError::IoError(err)) => {
                    log::warn!("[audio] decoder I/O failed: {err}");
                    return PlaybackLoopExit::Reconnect;
                }
                Err(SymphError::ResetRequired) => {
                    log::warn!("[audio] decoder requested reinitialization");
                    if let Err(err) = Self::rebuild_decoder_for_current_stream(&inner, &mut ctx) {
                        log::warn!("{err}");
                        return PlaybackLoopExit::Reconnect;
                    }
                    sample_buf = None;
                    continue;
                }
                Err(err) => {
                    log::warn!("[audio] decoder failed: {err}");
                    return PlaybackLoopExit::Reconnect;
                }
            };
            let decoded_format_changed = if ctx.collect_timing {
                let mut state = inner.lock().unwrap();
                let changed = state.diagnostics.timing.decoded_sample_rate != decoded.spec().rate
                    || state.diagnostics.timing.decoded_channels != decoded.spec().channels.count();
                state.diagnostics.timing.decoded(
                    decoded.frames(),
                    decoded.spec().rate,
                    decoded.spec().channels.count(),
                );
                changed
            } else {
                false
            };
            if decoded_format_changed {
                log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"decoded_format","sampleRate":decoded.spec().rate,"channels":decoded.spec().channels.count(),"packetTimeBaseNumer":ctx.packet_time_base.map(|tb| tb.numer),"packetTimeBaseDenom":ctx.packet_time_base.map(|tb| tb.denom)}));
            }

            if sample_buf
                .as_ref()
                .map(|b| b.capacity() < decoded.capacity())
                .unwrap_or(true)
            {
                sample_buf = Some(SampleBuffer::<f32>::new(
                    decoded.capacity() as u64,
                    *decoded.spec(),
                ));
            }

            let sb = sample_buf.as_mut().unwrap();
            let decoded_rate = decoded.spec().rate as usize;
            sb.copy_interleaved_ref(decoded);
            let samples = sb.samples();
            let n_frames = samples.len() / ctx.in_channels;

            if decoded_rate != ctx.current_in_rate {
                ctx.current_in_rate = decoded_rate;
                ctx.resampler = if ctx.current_in_rate != ctx.out_sample_rate {
                    Some(
                        FftFixedInOut::<f32>::new(
                            ctx.current_in_rate,
                            ctx.out_sample_rate,
                            BASE_FRAME_SIZE,
                            ctx.in_channels,
                        )
                        .expect("Failed to recreate resampler"),
                    )
                } else {
                    None
                };
                for buf in &mut ctx.in_channel_buffers {
                    buf.clear();
                }
            }

            for ch in 0..ctx.in_channels {
                ctx.in_channel_buffers[ch].reserve(n_frames);
            }

            for frame_idx in 0..n_frames {
                for ch in 0..ctx.in_channels {
                    let s = samples[frame_idx * ctx.in_channels + ch];
                    ctx.in_channel_buffers[ch].push(s);
                }
            }

            loop {
                let needed_in = ctx
                    .resampler
                    .as_ref()
                    .map(|r| r.input_frames_next())
                    .unwrap_or(BASE_FRAME_SIZE);

                if ctx
                    .in_channel_buffers
                    .iter()
                    .any(|buf| buf.len() < needed_in)
                {
                    break;
                }

                let mut input_slices: Vec<&[f32]> = Vec::with_capacity(ctx.in_channels);
                for ch in 0..ctx.in_channels {
                    input_slices.push(&ctx.in_channel_buffers[ch][..needed_in]);
                }

                let out_slices: Vec<Vec<f32>> = if let Some(resampler) = ctx.resampler.as_mut() {
                    resampler
                        .process(&input_slices, None)
                        .expect("Resample failed")
                } else {
                    input_slices.iter().map(|s| s.to_vec()).collect()
                };

                for ch in 0..ctx.in_channels {
                    let buf = &mut ctx.in_channel_buffers[ch];
                    buf.drain(..needed_in);
                }
                if ctx.collect_timing {
                    let mut state = inner.lock().unwrap();
                    state.diagnostics.timing.resampler_input_seconds +=
                        needed_in as f64 / ctx.current_in_rate as f64;
                    state.diagnostics.timing.generated_output_seconds +=
                        out_slices[0].len() as f64 / ctx.out_sample_rate as f64;
                    state.diagnostics.timing.input_buffer_seconds =
                        ctx.in_channel_buffers[0].len() as f64 / ctx.current_in_rate as f64;
                }

                let mut mapped: Vec<&[f32]> = Vec::with_capacity(ctx.out_channels);
                if !ctx.needs_channel_map {
                    for ch in 0..ctx.out_channels {
                        mapped.push(&out_slices[ch]);
                    }
                } else if ctx.in_channels == 1 && ctx.out_channels == 2 {
                    mapped.push(&out_slices[0]);
                    mapped.push(&out_slices[0]);
                } else {
                    for ch in 0..ctx.out_channels {
                        let src = if ch < ctx.in_channels {
                            ch
                        } else {
                            ctx.in_channels - 1
                        };
                        mapped.push(&out_slices[src]);
                    }
                }

                if let Ok(st) = inner.lock() {
                    ctx.eq_state.update_target(&st.target_eq);
                }

                ctx.eq_state.smooth(ctx.out_sample_rate as f32);

                let out_frames = mapped[0].len();
                let mut interleaved: Vec<f32> = Vec::with_capacity(out_frames * ctx.out_channels);
                let mut mono_chunk: Vec<f32> = Vec::with_capacity(out_frames);

                for i in 0..out_frames {
                    let mut mono_acc = 0.0f32;
                    for ch in 0..ctx.out_channels {
                        let s = mapped[ch][i];
                        interleaved.push(s);
                        mono_acc += s;
                    }
                    mono_chunk.push(mono_acc / ctx.out_channels as f32);
                }

                ctx.eq_state.apply_eq(&mut interleaved, ctx.out_channels);

                inner.lock().unwrap().diagnostics.decoded_samples += interleaved.len() as u64;
                for part in interleaved.chunks(2048) {
                    for sample in part {
                        loop {
                            if ctx.ring_producer.push(*sample).is_ok() {
                                break;
                            }
                            if let Some(exit) = Self::interruption(&inner) {
                                return exit;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                    if ctx.headless {
                        let mut output = vec![0.0; part.len()];
                        fill_output_from_ring_buffer(
                            &mut output,
                            &inner,
                            ctx.ring_consumer.as_mut().unwrap(),
                        );
                    }
                }

                if let Ok(mut inner) = inner.lock() {
                    if let Some(analyzer) = inner.spectrum_analyzer.as_mut() {
                        analyzer.push_block(&mono_chunk);
                        inner.spectrum_levels = analyzer.levels();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod backpressure_regression {
    use super::*;
    use ringbuf::Rb;
    #[test]
    fn output_clock_distinguishes_real_silence_mute_and_empty_buffer() {
        let engine = AudioEngine::new_sink();
        {
            let mut state = engine.inner.lock().unwrap();
            state.diagnostics.timing.output_sample_rate = 48000;
            state.diagnostics.timing.output_channels = 2;
            state.muted = true;
        }
        let (mut producer, mut consumer) = HeapRb::<f32>::new(8).split();
        for sample in [0.0, 0.0, 1.0, 1.0] {
            producer.push(sample).unwrap();
        }
        let mut output = [42.0; 8];
        fill_output_from_ring_buffer(&mut output, &engine.inner, &mut consumer);
        assert_eq!(output, [0.0; 8]);
        let t = engine.diagnostics().timing;
        assert_eq!(t.underrun_samples, 4);
        assert_eq!(t.underrun_callbacks, 1);
        assert!((t.device_seconds - 8.0 / 96000.0).abs() < 1e-12);
        assert!((t.played_audio_seconds - 4.0 / 96000.0).abs() < 1e-12);
        assert!((t.underrun_seconds - 4.0 / 96000.0).abs() < 1e-12);
    }
    #[test]
    fn output_failure_interrupts_a_full_ring_and_stop_wins() {
        let engine = AudioEngine::new_sink();
        let mut ring = HeapRb::<f32>::new(1);
        assert!(ring.push(1.).is_ok());
        assert!(ring.push(2.).is_err());
        engine.inner.lock().unwrap().output_failed = true;
        assert_eq!(
            AudioEngine::begin_loop(&engine.inner, None),
            Some(PlaybackLoopExit::Reconnect)
        );
        assert!(!engine.is_playing());
        assert_eq!(
            AudioEngine::interruption(&engine.inner),
            Some(PlaybackLoopExit::Reconnect)
        );
        engine.request_stop();
        assert_eq!(
            AudioEngine::interruption(&engine.inner),
            Some(PlaybackLoopExit::Stopped)
        );
    }
}
