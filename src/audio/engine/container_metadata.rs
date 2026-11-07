//! Consume Symphonia metadata revisions from the played connection, including
//! updates on chained Ogg streams. Artwork-only revisions must not hide tags.
use super::{StreamMetadataEvent, StreamMetadataSink};
use crate::metadata::{helpers::normalize_stream_title_case, Info};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use symphonia::core::meta::{Metadata, MetadataRevision, StandardTagKey, Value};

fn empty() -> Info {
    Info {
        title: String::new(),
        station_name: None,
    }
}
fn text(value: &Value) -> Option<&str> {
    match value {
        Value::String(s) => {
            let s = s.trim_matches(|c: char| c.is_whitespace() || c == '\0');
            (!s.is_empty()).then_some(s)
        }
        _ => None,
    }
}
fn extract(revision: &MetadataRevision) -> Info {
    let tags = revision.tags();
    let field = |standard, raw: &str| {
        tags.iter()
            .filter(|t| t.std_key == Some(standard))
            .find_map(|t| text(&t.value))
            .or_else(|| {
                tags.iter()
                    .filter(|t| t.key.eq_ignore_ascii_case(raw))
                    .find_map(|t| text(&t.value))
            })
    };
    let title = field(StandardTagKey::TrackTitle, "TITLE");
    let artist = field(StandardTagKey::Artist, "ARTIST");
    let title = title
        .map(|title| match artist {
            Some(artist)
                if !title
                    .to_lowercase()
                    .starts_with(&format!("{} - ", artist.to_lowercase())) =>
            {
                format!("{artist} - {title}")
            }
            _ => title.to_owned(),
        })
        .map(|t| normalize_stream_title_case(&html_escape::decode_html_entities(&t)))
        .unwrap_or_default();
    // Explicit station fields only. ORGANIZATION/PUBLISHER may be a record label;
    // ARTIST, TITLE and ALBUM describe the song, not the radio station.
    let station_name = tags.iter().find_map(|tag| {
        let key: String = tag
            .key
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .flat_map(char::to_uppercase)
            .collect();
        matches!(
            key.as_str(),
            "STATION"
                | "STATIONNAME"
                | "RADIOSTATION"
                | "RADIOSTATIONNAME"
                | "RADIONAME"
                | "ICYNAME"
                | "TRSN"
        )
        .then(|| text(&tag.value))
        .flatten()
        .and_then(crate::metadata::direct::normalize_station_name)
    });
    Info {
        title,
        station_name,
    }
}
fn merge(into: &mut Info, next: Info) {
    if !next.title.is_empty() {
        into.title = next.title;
    }
    if next.station_name.is_some() {
        into.station_name = next.station_name;
    }
}
#[derive(Default)]
struct Cursor {
    seen: bool,
}
impl Cursor {
    fn read(&mut self, mut metadata: Metadata<'_>) -> Option<Info> {
        // MetadataLog retains its newest revision. Skip that already-consumed
        // revision, then visit ALL new revisions, not just the final picture.
        if self.seen && metadata.pop().is_none() {
            return None;
        }
        let mut result = None;
        while let Some(revision) = metadata.current() {
            self.seen = true;
            merge(result.get_or_insert_with(empty), extract(revision));
            if metadata.pop().is_none() {
                break;
            }
        }
        result
    }
}
pub(super) struct ContainerMetadata {
    cursor: Cursor,
    enabled: bool,
    request_id: u64,
    sink: Option<StreamMetadataSink>,
    cancel: Arc<AtomicBool>,
    last_title: Option<String>,
    last_station: Option<String>,
}
impl ContainerMetadata {
    pub fn new(
        request_id: u64,
        enabled: bool,
        sink: Option<StreamMetadataSink>,
        cancel: Arc<AtomicBool>,
        prefix: Option<Metadata<'_>>,
        format: Metadata<'_>,
    ) -> Self {
        let mut info = prefix
            .and_then(|m| Cursor::default().read(m))
            .unwrap_or_else(empty);
        let mut cursor = Cursor::default();
        if let Some(next) = cursor.read(format) {
            merge(&mut info, next);
        }
        let mut result = Self {
            cursor,
            enabled,
            request_id,
            sink,
            cancel,
            last_title: None,
            last_station: None,
        };
        result.publish(info, true);
        result
    }
    pub fn update(&mut self, metadata: Metadata<'_>, reset: bool) {
        let next = self.cursor.read(metadata);
        if reset {
            self.last_title = None;
            self.last_station = None;
            self.publish(next.unwrap_or_else(empty), true);
        } else if let Some(info) = next {
            self.publish(info, false);
        }
    }
    fn publish(&mut self, mut info: Info, force: bool) {
        let title_changed =
            !info.title.is_empty() && self.last_title.as_deref() != Some(&info.title);
        let station_changed = info.station_name.is_some() && self.last_station != info.station_name;
        if !info.title.is_empty() {
            self.last_title = Some(info.title.clone());
        }
        if info.station_name.is_some() {
            self.last_station = info.station_name.clone();
        }
        if (station_changed || (self.enabled && (force || title_changed)))
            && !self.cancel.load(Ordering::Acquire)
        {
            let has_title = self.enabled && self.last_title.is_some();
            if !self.enabled {
                info.title.clear();
            }
            log::debug!(target:"ae_strom::http_trace","{}",serde_json::json!({"event":"audio_metadata_source","requestId":self.request_id,"source":if !self.enabled{"container_station"}else if has_title{"container"}else{"fallback"},"title":info.title,"stationName":info.station_name}));
            if let Some(sink) = &self.sink {
                sink(StreamMetadataEvent::Container {
                    request_id: self.request_id,
                    info,
                    has_title,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use symphonia::core::meta::{MetadataBuilder, MetadataLog, Tag};
    fn revision(fields: &[(&str, &str)]) -> MetadataRevision {
        let mut builder = MetadataBuilder::new();
        for (key, value) in fields {
            builder.add_tag(Tag::new(None, key, Value::String((*value).into())));
        }
        builder.metadata()
    }
    #[test]
    fn titles_and_explicit_station_names_do_not_confuse_song_and_station() {
        let i = extract(&revision(&[
            ("ARTIST", "Anita Ward"),
            ("TITLE", "Ring My Bell"),
            ("STATION_NAME", "Fixture Radio"),
        ]));
        assert_eq!(i.title, "Anita Ward - Ring My Bell");
        assert_eq!(i.station_name.as_deref(), Some("Fixture Radio"));
        let i = extract(&revision(&[
            ("TITLE", "Anita Ward - Ring My Bell"),
            ("ARTIST", "Anita Ward"),
            ("ORGANIZATION", "Record Label"),
        ]));
        assert_eq!(i.title, "Anita Ward - Ring My Bell");
        assert!(i.station_name.is_none());
        assert!(
            extract(&revision(&[("ARTIST", "Someone"), ("TITLE", "  ")]))
                .title
                .is_empty()
        );
    }
    #[test]
    fn revisions_survive_picture_only_metadata_and_reset_without_reusing_old_artist() {
        let mut log = MetadataLog::default();
        log.push(revision(&[
            ("ARTIST", "First Artist"),
            ("TITLE", "First Track"),
        ]));
        log.push(revision(&[]));
        let events = Arc::new(Mutex::new(Vec::new()));
        let received = events.clone();
        let mut state = ContainerMetadata::new(
            1,
            true,
            Some(Arc::new(move |e| received.lock().unwrap().push(e))),
            Arc::new(AtomicBool::new(false)),
            None,
            log.metadata(),
        );
        state.update(log.metadata(), false);
        assert_eq!(events.lock().unwrap().len(), 1);
        log.push(revision(&[("TITLE", "Second Track")]));
        state.update(log.metadata(), true);
        log.push(revision(&[]));
        state.update(log.metadata(), true);
        let events = events.lock().unwrap();
        assert!(
            matches!(&events[0],StreamMetadataEvent::Container{info,has_title:true,..} if info.title=="First Artist - First Track")
        );
        assert!(
            matches!(&events[1],StreamMetadataEvent::Container{info,has_title:true,..} if info.title=="Second Track")
        );
        assert!(matches!(
            &events[2],
            StreamMetadataEvent::Container {
                has_title: false,
                ..
            }
        ));
    }
}
