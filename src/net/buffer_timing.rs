use super::BodyMessage;
use crate::audio::engine::AdtsStats;
use serde::Serialize;
use std::{
    sync::{
        mpsc::{SyncSender, TrySendError},
        Arc, Mutex,
    },
    time::Instant,
};

#[derive(Clone, Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpBufferStats {
    pub enabled: bool,
    pub request_id: u64,
    pub wall_seconds: f64,
    pub received_bytes: u64,
    pub delivered_bytes: u64,
    pub discarded_bytes: u64,
    pub received_chunks: u64,
    pub worker_pending_bytes: u64,
    pub channel_queued_bytes: u64,
    pub channel_queued_chunks: u64,
    pub reader_pending_bytes: u64,
    pub retained_bytes: u64,
    pub peak_retained_bytes: u64,
    pub peak_channel_bytes: u64,
    pub producer_wait_seconds: f64,
    pub producer_wait_events: u64,
    pub producer_waiting: bool,
    pub producer_ended: bool,
    pub reader_closed: bool,
    pub received_adts: AdtsStats,
}
struct State {
    data: HttpBufferStats,
    started: Option<Instant>,
    waiting_since: Option<Instant>,
}
#[derive(Clone)]
pub(crate) struct HttpBufferTiming(Arc<Mutex<State>>);
impl State {
    fn retained(&mut self) {
        self.data.retained_bytes = self.data.worker_pending_bytes
            + self.data.channel_queued_bytes
            + self.data.reader_pending_bytes;
        self.data.peak_retained_bytes = self.data.peak_retained_bytes.max(self.data.retained_bytes);
        debug_assert_eq!(
            self.data.received_bytes,
            self.data.delivered_bytes + self.data.discarded_bytes + self.data.retained_bytes
        );
    }
    fn end_wait(&mut self) {
        if let Some(started) = self.waiting_since.take() {
            self.data.producer_wait_seconds += started.elapsed().as_secs_f64();
        }
    }
}
impl HttpBufferTiming {
    pub fn new(request_id: u64) -> Self {
        Self(Arc::new(Mutex::new(State {
            data: HttpBufferStats {
                enabled: true,
                request_id,
                ..Default::default()
            },
            started: None,
            waiting_since: None,
        })))
    }
    pub fn start(&self) {
        self.0.lock().unwrap().started = Some(Instant::now());
    }
    pub fn received(&self, length: usize, adts: Option<&AdtsStats>) {
        let mut state = self.0.lock().unwrap();
        state.data.received_bytes += length as u64;
        state.data.received_chunks += 1;
        state.data.worker_pending_bytes += length as u64;
        if let Some(adts) = adts {
            state.data.received_adts = adts.clone();
        }
        state.retained();
    }
    /// Account for enqueue while holding the same lock the receiver uses. This
    /// avoids a receiver observing an unaccounted message between send/update.
    pub fn try_send(
        &self,
        tx: &SyncSender<BodyMessage>,
        message: BodyMessage,
    ) -> Result<(), TrySendError<BodyMessage>> {
        let length = message.as_ref().map_or(0, |bytes| bytes.len()) as u64;
        let mut state = self.0.lock().unwrap();
        state.data.worker_pending_bytes -= length;
        state.data.channel_queued_bytes += length;
        if length > 0 {
            state.data.channel_queued_chunks += 1;
        }
        let result = tx.try_send(message);
        match &result {
            Ok(()) => {
                state.data.peak_channel_bytes = state
                    .data
                    .peak_channel_bytes
                    .max(state.data.channel_queued_bytes);
                state.end_wait();
            }
            Err(_) => {
                state.data.channel_queued_bytes -= length;
                if length > 0 {
                    state.data.channel_queued_chunks -= 1;
                }
                state.data.worker_pending_bytes += length;
                if matches!(&result, Err(TrySendError::Full(_))) && state.waiting_since.is_none() {
                    state.waiting_since = Some(Instant::now());
                    state.data.producer_wait_events += 1;
                }
            }
        }
        state.retained();
        result
    }
    pub fn take_from_channel(&self, length: usize) {
        let mut state = self.0.lock().unwrap();
        state.data.channel_queued_bytes -= length as u64;
        if length > 0 {
            state.data.channel_queued_chunks -= 1;
        }
        state.data.reader_pending_bytes += length as u64;
        state.retained();
    }
    pub fn delivered(&self, length: usize) {
        let mut state = self.0.lock().unwrap();
        state.data.reader_pending_bytes -= length as u64;
        state.data.delivered_bytes += length as u64;
        state.retained();
    }
    pub fn producer_ended(&self) {
        let mut state = self.0.lock().unwrap();
        state.end_wait();
        state.data.discarded_bytes += state.data.worker_pending_bytes;
        state.data.worker_pending_bytes = 0;
        state.data.producer_ended = true;
        state.retained();
    }
    pub fn reader_closed(&self) {
        let mut state = self.0.lock().unwrap();
        // The producer has been joined before this method is called.
        state.data.discarded_bytes +=
            state.data.channel_queued_bytes + state.data.reader_pending_bytes;
        state.data.channel_queued_bytes = 0;
        state.data.channel_queued_chunks = 0;
        state.data.reader_pending_bytes = 0;
        state.data.reader_closed = true;
        state.retained();
    }
    pub fn snapshot(&self) -> HttpBufferStats {
        let state = self.0.lock().unwrap();
        let mut data = state.data.clone();
        data.wall_seconds = state
            .started
            .map_or(0.0, |start| start.elapsed().as_secs_f64());
        data.producer_waiting = state.waiting_since.is_some();
        if let Some(started) = state.waiting_since {
            data.producer_wait_seconds += started.elapsed().as_secs_f64();
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::engine::AdtsClock;
    #[test]
    fn seconds_are_counted_before_a_full_queue_and_bytes_follow_partial_reads_and_cancel() {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let timing = HttpBufferTiming::new(42);
        timing.start();
        let frame = [
            0xff, 0xf1, 0x50, 0x80, 0x01, 0xff, 0xfc, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let body = frame.repeat(5000);
        let mut clock = AdtsClock::default();
        clock.feed(&body);
        timing.received(body.len(), Some(&clock.stats));
        timing.try_send(&tx, Ok(body.clone())).unwrap();
        clock.feed(&body);
        timing.received(body.len(), Some(&clock.stats));
        let pending = timing.try_send(&tx, Ok(body.clone())).unwrap_err();
        let stats = timing.snapshot();
        assert_eq!(stats.received_adts.frames, 10000);
        assert!((stats.received_adts.seconds - 10240000.0 / 44100.0).abs() < 1e-8);
        assert!(
            stats.received_adts.seconds > 180.0,
            "minutes are visible before any decoder read"
        );
        assert_eq!(stats.channel_queued_bytes, 75000);
        assert_eq!(stats.worker_pending_bytes, 75000);
        assert_eq!(stats.retained_bytes, 150000);
        assert!(stats.producer_waiting);
        let first = rx.recv().unwrap().unwrap();
        timing.take_from_channel(first.len());
        timing.delivered(6);
        let stats = timing.snapshot();
        assert_eq!(stats.reader_pending_bytes, 74994);
        assert_eq!(stats.retained_bytes, 149994);
        let TrySendError::Full(second) = pending else {
            panic!("expected full queue");
        };
        timing.try_send(&tx, second).unwrap();
        timing.producer_ended();
        timing.reader_closed();
        let stats = timing.snapshot();
        assert_eq!(stats.delivered_bytes, 6);
        assert_eq!(stats.discarded_bytes, 149994);
        assert_eq!(stats.retained_bytes, 0);
        assert_eq!(stats.producer_wait_events, 1);
        assert!(!stats.producer_waiting);
    }
}
