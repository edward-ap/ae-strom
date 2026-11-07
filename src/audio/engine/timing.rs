use super::{AdtsStats, HttpBufferStats};
use serde::Serialize;
use symphonia::core::units::TimeBase;

/// Clocks are scoped to one HTTP/audio connection. Seconds are accumulated at the
/// actual rate of each block, never by dividing old totals by the latest rate.
#[derive(Clone, Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackTiming {
    pub connection: u64,
    pub request_id: u64,
    pub active: bool,
    pub wall_seconds: f64,
    pub codec: String,
    pub declared_sample_rate: u32,
    pub decoded_sample_rate: u32,
    pub decoded_channels: usize,
    pub output_sample_rate: u32,
    pub output_channels: usize,
    pub time_base_numer: Option<u32>,
    pub time_base_denom: Option<u32>,
    pub packets: u64,
    pub packet_duration_unknown: u64,
    pub packet_discontinuities: u64,
    pub source_packet_seconds: f64,
    pub decode_error_packets: u64,
    pub decoded_frames: u64,
    pub decoded_audio_seconds: f64,
    pub resampler_input_seconds: f64,
    pub generated_output_seconds: f64,
    pub device_seconds: f64,
    pub played_audio_seconds: f64,
    pub underrun_samples: u64,
    pub underrun_callbacks: u64,
    pub underrun_seconds: f64,
    pub pcm_buffer_seconds: f64,
    pub input_buffer_seconds: f64,
    pub adts: AdtsStats,
    pub http: HttpBufferStats,
    #[serde(skip)]
    pub(super) expected_ts: Option<u64>,
}
impl PlaybackTiming {
    pub(super) fn packet(&mut self, ts: u64, dur: u64, time_base: Option<TimeBase>) {
        self.packets += 1;
        if self.expected_ts.is_some_and(|expected| expected != ts) {
            self.packet_discontinuities += 1;
        }
        self.expected_ts = ts.checked_add(dur);
        if let Some(tb) = time_base.filter(|tb| tb.numer > 0 && tb.denom > 0 && dur > 0) {
            self.source_packet_seconds += dur as f64 * f64::from(tb.numer) / f64::from(tb.denom);
        } else {
            self.packet_duration_unknown += 1;
        }
    }
    pub(super) fn decoded(&mut self, frames: usize, rate: u32, channels: usize) {
        self.decoded_sample_rate = rate;
        self.decoded_channels = channels;
        self.decoded_frames += frames as u64;
        if rate > 0 {
            self.decoded_audio_seconds += frames as f64 / f64::from(rate);
        }
    }
    pub(super) fn output(&mut self, samples: usize, missing: usize, buffered: usize) {
        let per_second = f64::from(self.output_sample_rate) * self.output_channels as f64;
        if per_second == 0.0 {
            return;
        }
        self.device_seconds += samples as f64 / per_second;
        self.played_audio_seconds += (samples - missing) as f64 / per_second;
        self.pcm_buffer_seconds = buffered as f64 / per_second;
        self.underrun_samples += missing as u64;
        self.underrun_seconds += missing as f64 / per_second;
        if missing > 0 {
            self.underrun_callbacks += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_seconds_and_decoded_seconds_agree_across_sample_rate_changes() {
        let mut t = PlaybackTiming::default();
        t.packet(0, 1024, Some(TimeBase::new(1, 22050)));
        t.decoded(2048, 44100, 2); // HE-AAC-style core/output clock relationship.
        t.packet(1024, 1024, Some(TimeBase::new(1, 44100)));
        t.decoded(1024, 44100, 1);
        assert!((t.source_packet_seconds - t.decoded_audio_seconds).abs() < 1e-12);
        assert_eq!(t.packet_discontinuities, 0);
        t.packet(9999, 0, None);
        assert_eq!(t.packet_duration_unknown, 1);
        assert_eq!(t.packet_discontinuities, 1);
    }
}
