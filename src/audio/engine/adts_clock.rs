use serde::Serialize;

const RATES: [u32; 13] = [
    96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350,
];

#[derive(Clone, Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdtsStats {
    pub enabled: bool,
    pub frames: u64,
    pub seconds: f64,
    pub sample_rate: u32,
    pub profile: u8,
    pub channel_configuration: u8,
    pub rate_changes: u64,
    pub format_changes: u64,
    pub discarded_bytes: u64,
    pub sync_losses: u64,
}

/// Passive ADTS framing observer. Never changes, buffers or replaces decoder input.
/// ADTS frequency is the AAC core rate; 1024 core samples per raw data block.
#[derive(Default)]
pub(crate) struct AdtsClock {
    pub stats: AdtsStats,
    header: [u8; 7],
    have: usize,
    remaining: usize,
    blocks: u8,
    locked: bool,
}
impl AdtsClock {
    pub fn feed(&mut self, mut bytes: &[u8]) {
        self.stats.enabled = true;
        while !bytes.is_empty() {
            if self.remaining > 0 {
                let count = self.remaining.min(bytes.len());
                self.remaining -= count;
                bytes = &bytes[count..];
                if self.remaining == 0 {
                    self.complete();
                }
                continue;
            }
            self.header[self.have] = bytes[0];
            self.have += 1;
            bytes = &bytes[1..];
            if self.have < 7 {
                continue;
            }
            let h = self.header;
            let rate = RATES.get(((h[2] >> 2) & 15) as usize).copied();
            let length =
                (((h[3] & 3) as usize) << 11) | ((h[4] as usize) << 3) | ((h[5] as usize) >> 5);
            let header_length = if h[1] & 1 != 0 { 7 } else { 9 };
            if h[0] != 0xff || h[1] & 0xf6 != 0xf0 || rate.is_none() || length <= header_length {
                self.stats.discarded_bytes += 1;
                if self.locked {
                    self.stats.sync_losses += 1;
                    self.locked = false;
                }
                self.header.copy_within(1..7, 0);
                self.have = 6;
                continue;
            }
            let rate = rate.unwrap();
            let profile = (h[2] >> 6) + 1;
            let channels = ((h[2] & 1) << 2) | (h[3] >> 6);
            if self.stats.sample_rate != 0 {
                if self.stats.sample_rate != rate {
                    self.stats.rate_changes += 1;
                }
                if (
                    self.stats.sample_rate,
                    self.stats.profile,
                    self.stats.channel_configuration,
                ) != (rate, profile, channels)
                {
                    self.stats.format_changes += 1;
                }
            }
            self.stats.sample_rate = rate;
            self.stats.profile = profile;
            self.stats.channel_configuration = channels;
            self.blocks = (h[6] & 3) + 1;
            self.remaining = length - 7;
            self.have = 0;
            self.locked = true;
        }
    }
    fn complete(&mut self) {
        self.stats.frames += 1;
        self.stats.seconds += 1024.0 * f64::from(self.blocks) / f64::from(self.stats.sample_rate);
    }
}

#[cfg(test)]
pub(super) fn frame(rate_index: u8, blocks: u8, crc: bool) -> Vec<u8> {
    let length: usize = if crc { 17 } else { 15 };
    let mut frame = vec![0; length];
    frame[..7].copy_from_slice(&[
        0xff,
        if crc { 0xf0 } else { 0xf1 },
        0x40 | (rate_index << 2),
        0x80 | ((length >> 11) as u8),
        (length >> 3) as u8,
        ((length & 7) as u8) << 5 | 0x1f,
        0xfc | (blocks - 1),
    ]);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fragmented_crc_frames_and_rate_changes_use_header_time_not_decoder_assumptions() {
        let bytes = [frame(4, 1, false), frame(7, 2, true), frame(7, 1, false)].concat();
        for chunk_size in [1, 2, 7, 13, 4096] {
            let mut clock = AdtsClock::default();
            for part in bytes.chunks(chunk_size) {
                clock.feed(part);
            }
            assert_eq!(clock.stats.frames, 3);
            assert_eq!(clock.stats.rate_changes, 1);
            assert_eq!(clock.stats.sample_rate, 22050);
            assert!((clock.stats.seconds - (1024.0 / 44100.0 + 3072.0 / 22050.0)).abs() < 1e-12);
            assert_eq!(clock.stats.discarded_bytes, 0);
        }
    }
    #[test]
    fn truncated_frames_are_not_counted_and_lost_sync_is_reported() {
        let first = frame(4, 1, false);
        let mut clock = AdtsClock::default();
        clock.feed(&first[..10]);
        assert_eq!(clock.stats.frames, 0);
        clock.feed(&first[10..]);
        assert_eq!(clock.stats.frames, 1);
        clock.feed(b"broken");
        clock.feed(&first);
        assert_eq!(clock.stats.frames, 2);
        assert_eq!(clock.stats.discarded_bytes, 6);
        assert_eq!(clock.stats.sync_losses, 1);
    }
    #[test]
    fn raw_header_clock_exposes_rate_changes_that_demuxer_time_base_does_not_follow() {
        use symphonia::core::{
            formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
        };
        let bytes = [frame(4, 1, false), frame(7, 1, false), frame(7, 1, false)].concat();
        let mut clock = AdtsClock::default();
        clock.feed(&bytes);
        let source =
            MediaSourceStream::new(Box::new(std::io::Cursor::new(bytes)), Default::default());
        let mut hint = Hint::new();
        hint.with_extension("aac");
        let mut format = symphonia::default::get_probe()
            .format(
                &hint,
                source,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .unwrap()
            .format;
        let tb = format
            .default_track()
            .unwrap()
            .codec_params
            .time_base
            .unwrap();
        let mut packet_seconds = 0.0;
        for _ in 0..3 {
            let packet = format.next_packet().unwrap();
            packet_seconds += packet.dur() as f64 * f64::from(tb.numer) / f64::from(tb.denom);
        }
        assert_eq!(tb.denom, 44100);
        assert_eq!(clock.stats.sample_rate, 22050);
        assert_eq!(clock.stats.rate_changes, 1);
        assert!((packet_seconds - 3072.0 / 44100.0).abs() < 1e-12);
        assert!((clock.stats.seconds - 5120.0 / 44100.0).abs() < 1e-12);
    }
}
