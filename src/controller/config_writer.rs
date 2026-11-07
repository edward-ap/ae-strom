use crate::{config, model::Config};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

const DEBOUNCE: Duration = Duration::from_millis(300);
const MAX_WAIT: Duration = Duration::from_secs(1);

struct Pending {
    version: u64,
    config: Config,
    first: Instant,
    latest: Instant,
}

#[derive(Default)]
struct State {
    version: u64,
    pending: Option<Pending>,
    completed: Option<(u64, Result<(), String>)>,
    force: bool,
    stopped: bool,
}

/// One in-flight write and one replaceable pending config, never a queue of copies.
pub(super) struct ConfigWriter {
    shared: Arc<(Mutex<State>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
    reported: u64,
}

impl ConfigWriter {
    pub(super) fn start(path: PathBuf) -> Result<Self, String> {
        Self::start_with(path, |path, cfg| {
            config::save(path, cfg).map_err(|e| format!("Settings were not saved: {e}"))
        })
    }

    fn start_with(
        path: PathBuf,
        mut save: impl FnMut(&Path, &Config) -> Result<(), String> + Send + 'static,
    ) -> Result<Self, String> {
        let shared = Arc::new((Mutex::new(State::default()), Condvar::new()));
        let worker_shared = shared.clone();
        let worker = thread::Builder::new()
            .name("radio-config-writer".into())
            .spawn(move || {
                let (lock, wake) = &*worker_shared;
                let mut state = lock.lock().unwrap();
                loop {
                    if state.stopped {
                        break;
                    }
                    let Some(pending) = &state.pending else {
                        state = wake.wait(state).unwrap();
                        continue;
                    };
                    let due = (pending.latest + DEBOUNCE).min(pending.first + MAX_WAIT);
                    if !state.force && Instant::now() < due {
                        state = wake
                            .wait_timeout(state, due.saturating_duration_since(Instant::now()))
                            .unwrap()
                            .0;
                        continue;
                    }
                    let pending = state.pending.take().unwrap();
                    state.force = false;
                    drop(state);
                    let result = save(&path, &pending.config);
                    state = lock.lock().unwrap();
                    state.completed = Some((pending.version, result));
                    wake.notify_all();
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            shared,
            worker: Some(worker),
            reported: 0,
        })
    }

    fn enqueue(state: &mut State, cfg: &Config) -> u64 {
        state.version += 1;
        let now = Instant::now();
        let first = state.pending.as_ref().map_or(now, |p| p.first);
        state.pending = Some(Pending {
            version: state.version,
            config: cfg.clone(),
            first,
            latest: now,
        });
        state.version
    }

    pub(super) fn schedule(&self, cfg: &Config) {
        let (lock, wake) = &*self.shared;
        Self::enqueue(&mut lock.lock().unwrap(), cfg);
        wake.notify_all();
    }

    pub(super) fn poll(&mut self) -> Option<Result<(), String>> {
        let state = self.shared.0.lock().unwrap();
        let (version, result) = state.completed.as_ref()?;
        if *version <= self.reported {
            return None;
        }
        self.reported = *version;
        Some(result.clone())
    }

    /// Shutdown bypasses debounce and waits behind any older in-flight write.
    /// A failed flush leaves the worker available for subsequent edits/retries.
    pub(super) fn flush(&mut self, cfg: &Config) -> Result<(), String> {
        let (lock, wake) = &*self.shared;
        let mut state = lock.lock().unwrap();
        let version = Self::enqueue(&mut state, cfg);
        state.force = true;
        wake.notify_all();
        loop {
            if let Some((saved, result)) = &state.completed {
                if *saved >= version {
                    self.reported = *saved;
                    return result.clone();
                }
            }
            state = wake.wait(state).unwrap();
        }
    }
}

impl Drop for ConfigWriter {
    fn drop(&mut self) {
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap().stopped = true;
        wake.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn blocked_write_accepts_edits_and_flush_serializes_only_the_latest_pending_config() {
        let (started, writes) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let mut writer = ConfigWriter::start_with(PathBuf::new(), move |_, cfg| {
            started.send(cfg.volume).unwrap();
            gate.recv_timeout(Duration::from_secs(3))
                .map_err(|e| e.to_string())
        })
        .unwrap();
        let mut cfg = config::default_config();
        cfg.volume = 10;
        writer.schedule(&cfg);
        assert_eq!(writes.recv_timeout(Duration::from_secs(2)).unwrap(), 10);
        // The I/O callback is blocked. These writes must still return immediately.
        for volume in 11..=99 {
            cfg.volume = volume;
            writer.schedule(&cfg);
        }
        let flush = thread::spawn(move || writer.flush(&cfg));
        assert!(
            writes.try_recv().is_err(),
            "only one write may be in flight"
        );
        release.send(()).unwrap();
        assert_eq!(writes.recv_timeout(Duration::from_secs(2)).unwrap(), 99);
        release.send(()).unwrap();
        flush.join().unwrap().unwrap();
        assert!(
            writes.try_recv().is_err(),
            "intermediate configs must be coalesced"
        );
    }

    #[test]
    fn burst_flush_bypasses_debounce_and_failed_flush_can_be_retried() {
        let (sent, saved) = mpsc::channel();
        let mut attempt = 0;
        let mut writer = ConfigWriter::start_with(PathBuf::new(), move |_, cfg| {
            attempt += 1;
            sent.send(cfg.volume).unwrap();
            if attempt == 1 {
                Err("disk unavailable".into())
            } else {
                Ok(())
            }
        })
        .unwrap();
        let mut cfg = config::default_config();
        for volume in 0..=100 {
            cfg.volume = volume;
            writer.schedule(&cfg);
        }
        assert!(writer.flush(&cfg).is_err());
        assert_eq!(saved.recv().unwrap(), 100);
        assert!(saved.try_recv().is_err());
        cfg.volume = 42;
        writer.flush(&cfg).unwrap();
        assert_eq!(saved.recv().unwrap(), 42);
        assert!(writer.poll().is_none(), "flush consumes its result");
    }

    #[test]
    fn background_failures_and_recovery_are_reported_without_another_action() {
        let (sent, saved) = mpsc::channel();
        let mut writer = ConfigWriter::start_with(PathBuf::new(), move |_, cfg| {
            sent.send(()).unwrap();
            if cfg.muted {
                Err("disk unavailable".into())
            } else {
                Ok(())
            }
        })
        .unwrap();
        let mut cfg = config::default_config();
        cfg.muted = true;
        writer.schedule(&cfg);
        saved.recv_timeout(Duration::from_secs(2)).unwrap();
        // Synchronize with completion rather than relying on a scheduling delay.
        let (lock, wake) = &*writer.shared;
        let state = lock.lock().unwrap();
        drop(
            wake.wait_timeout_while(state, Duration::from_secs(2), |s| s.completed.is_none())
                .unwrap(),
        );
        assert!(writer.poll().unwrap().is_err());
        cfg.muted = false;
        writer.schedule(&cfg);
        saved.recv_timeout(Duration::from_secs(2)).unwrap();
        let (lock, wake) = &*writer.shared;
        let state = lock.lock().unwrap();
        drop(
            wake.wait_timeout_while(state, Duration::from_secs(2), |s| {
                s.completed.as_ref().unwrap().0 < 2
            })
            .unwrap(),
        );
        assert!(writer.poll().unwrap().is_ok());
    }
}
