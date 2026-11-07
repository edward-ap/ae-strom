//! ICY framing shared by playback and metadata-only readers.
use super::{direct::parse_stream_title, Info};
use std::{
    io::{self, Read},
    sync::Arc,
};

pub(crate) const MAX_INTERVAL: usize = 1_048_576;

/// Passive framing for ingress clocks. Retains no audio or metadata payload.
pub(crate) struct AudioObserver {
    interval: usize,
    audio_left: usize,
    metadata_left: Option<usize>,
}
impl AudioObserver {
    pub(crate) fn new(interval: usize) -> Self {
        Self {
            interval,
            audio_left: interval,
            metadata_left: None,
        }
    }
    pub(crate) fn feed(&mut self, mut bytes: &[u8], mut audio: impl FnMut(&[u8])) {
        while !bytes.is_empty() {
            if let Some(left) = self.metadata_left {
                let n = left.min(bytes.len());
                bytes = &bytes[n..];
                if n == left {
                    self.metadata_left = None;
                    self.audio_left = self.interval;
                } else {
                    self.metadata_left = Some(left - n);
                }
            } else if self.audio_left > 0 {
                let n = self.audio_left.min(bytes.len());
                audio(&bytes[..n]);
                self.audio_left -= n;
                bytes = &bytes[n..];
            } else {
                self.metadata_left = Some(usize::from(bytes[0]) * 16);
                bytes = &bytes[1..];
            }
        }
    }
}

pub(crate) fn interval(value: Option<&str>) -> io::Result<Option<usize>> {
    match value {
        None => Ok(None),
        Some(value) => match value.trim().parse::<usize>() {
            Ok(0) => Ok(None),
            Ok(n) if n <= MAX_INTERVAL => Ok(Some(n)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid icy-metaint",
            )),
        },
    }
}

/// Exposes only audio bytes. Metadata is consumed on the SAME reader, never by
/// a second HTTP request. Each instance belongs to one connection.
pub(crate) struct IcyReader<R> {
    reader: R,
    interval: Option<usize>,
    audio_left: usize,
    failed: bool,
    request_id: u64,
    block: u64,
    last_title: String,
    on_info: Arc<dyn Fn(Info) + Send + Sync>,
}
impl<R: Read> IcyReader<R> {
    pub(crate) fn new(
        reader: R,
        interval: Option<usize>,
        request_id: u64,
        on_info: Arc<dyn Fn(Info) + Send + Sync>,
    ) -> Self {
        Self {
            reader,
            interval,
            audio_left: interval.unwrap_or(0),
            failed: false,
            request_id,
            block: 0,
            last_title: String::new(),
            on_info,
        }
    }
    fn metadata(&mut self) -> io::Result<()> {
        // A truncated block must never let a subsequent read mistake its tail
        // for audio or a new metadata length byte.
        self.failed = true;
        let mut length = [0];
        self.reader.read_exact(&mut length)?;
        let length = usize::from(length[0]) * 16;
        let mut bytes = [0; 255 * 16];
        self.reader.read_exact(&mut bytes[..length])?;
        self.block += 1;
        if length > 0 {
            log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"icy_block","source":"audio","requestId":self.request_id,"block":self.block,"audioBytes":self.block.saturating_mul(self.interval.unwrap() as u64),"raw":String::from_utf8_lossy(&bytes[..length]).trim_end_matches('\0')}));
            if let Some(title) = parse_stream_title(&bytes[..length]) {
                if title != self.last_title {
                    self.last_title = title.clone();
                    (self.on_info)(Info {
                        title,
                        station_name: None,
                    });
                }
            }
        }
        self.audio_left = self.interval.unwrap();
        self.failed = false;
        Ok(())
    }
}
impl<R: Read> Read for IcyReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Truncated ICY metadata",
            ));
        }
        if self.interval.is_none() {
            return self.reader.read(out);
        }
        if self.audio_left == 0 {
            self.metadata()?;
        }
        let length = out.len().min(self.audio_left);
        let count = self.reader.read(&mut out[..length])?;
        self.audio_left -= count;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, sync::Mutex};
    fn block(audio: &[u8], title: Option<&str>) -> Vec<u8> {
        let mut out = audio.to_vec();
        let mut meta = title.map_or(Vec::new(), |t| format!("StreamTitle='{t}';").into_bytes());
        let length = meta.len().div_ceil(16);
        meta.resize(length * 16, 0);
        out.push(length as u8);
        out.extend(meta);
        out
    }
    struct Fragmented(Cursor<Vec<u8>>, usize);
    impl Read for Fragmented {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let n = out.len().min(self.1);
            self.0.read(&mut out[..n])
        }
    }
    #[test]
    fn fragmented_metadata_never_reaches_decoder_and_changed_titles_are_normalized() {
        let bytes = [
            block(b"abcd", Some("ARTIST - ONE")),
            block(b"efgh", None),
            block(b"ijkl", Some("ARTIST - ONE")),
            block(b"mnop", Some("OTHER - TWO")),
            b"qr".to_vec(),
        ]
        .concat();
        for chunk in [1, 2, 7, 64] {
            let mut observer = AudioObserver::new(4);
            let mut observed = Vec::new();
            for bytes in bytes.chunks(chunk) {
                observer.feed(bytes, |audio| observed.extend_from_slice(audio));
            }
            assert_eq!(observed, b"abcdefghijklmnopqr");
            let titles = Arc::new(Mutex::new(Vec::new()));
            let got = titles.clone();
            let mut reader = IcyReader::new(
                Fragmented(Cursor::new(bytes.clone()), chunk),
                Some(4),
                1,
                Arc::new(move |i| got.lock().unwrap().push(i.title)),
            );
            let mut audio = Vec::new();
            reader.read_to_end(&mut audio).unwrap();
            assert_eq!(audio, b"abcdefghijklmnopqr");
            assert_eq!(*titles.lock().unwrap(), vec!["Artist - One", "Other - Two"]);
        }
    }
    #[test]
    fn truncated_metadata_is_terminal_and_a_new_connection_resets_framing() {
        let mut bytes = block(b"abcd", Some("Artist - Track"));
        bytes.truncate(bytes.len() - 3);
        let mut reader = IcyReader::new(Cursor::new(bytes), Some(4), 1, Arc::new(|_| {}));
        assert_eq!(reader.read(&mut [0; 8]).unwrap(), 4);
        assert_eq!(
            reader.read(&mut [0; 8]).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert!(reader.read(&mut [0; 8]).is_err());
        let mut next = IcyReader::new(Cursor::new(b"wxyz"), Some(4), 2, Arc::new(|_| {}));
        let mut audio = [0; 4];
        next.read_exact(&mut audio).unwrap();
        assert_eq!(&audio, b"wxyz");
    }
    #[test]
    fn absent_zero_and_invalid_intervals_and_unframed_audio() {
        assert_eq!(interval(None).unwrap(), None);
        assert_eq!(interval(Some("0")).unwrap(), None);
        assert_eq!(interval(Some(" 16000 ")).unwrap(), Some(16000));
        for value in ["-1", "oops", "1048577"] {
            assert!(interval(Some(value)).is_err());
        }
        let mut r = IcyReader::new(
            Cursor::new(b"plain audio"),
            None,
            1,
            Arc::new(|_| panic!("no ICY")),
        );
        let mut bytes = Vec::new();
        r.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"plain audio");
    }
}
