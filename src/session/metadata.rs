//! Select metadata only after the audio response establishes ICY capability.
use super::{SessionSpec, StationNamePolicy};
use crate::{
    audio::engine::{StreamMetadataEvent, StreamMetadataSink},
    metadata::{provider::MetadataWatch, Provider},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
    Arc, Mutex,
};

pub(super) struct SessionMetadata {
    spec: SessionSpec,
    events: Receiver<StreamMetadataEvent>,
    request_id: Option<u64>,
    icy: bool,
    epoch: Arc<Mutex<u64>>,
    fallback: Option<MetadataWatch>,
}
impl SessionMetadata {
    pub fn new(spec: SessionSpec) -> (Self, StreamMetadataSink) {
        let (send, events) = mpsc::channel();
        (
            Self {
                spec,
                events,
                request_id: None,
                icy: false,
                epoch: Arc::new(Mutex::new(0)),
                fallback: None,
            },
            Arc::new(move |event| {
                let _ = send.send(event);
            }),
        )
    }
    pub fn request_stop(&self) {
        *self.epoch.lock().unwrap() += 1;
        if let Some(watch) = &self.fallback {
            watch.request_stop();
        }
    }
    pub fn stop(&mut self) {
        self.request_stop();
        if let Some(mut watch) = self.fallback.take() {
            watch.stop();
        }
    }
    pub fn poll(&mut self) {
        if self.spec.cancel.load(Ordering::Acquire) {
            return;
        }
        while let Ok(event) = self.events.try_recv() {
            if self.spec.cancel.load(Ordering::Acquire) {
                return;
            }
            match event {
                StreamMetadataEvent::Connected { request_id, icy } => {
                    self.stop();
                    self.request_id = Some(request_id);
                    self.icy = icy;
                }
                StreamMetadataEvent::Container {
                    request_id,
                    info,
                    has_title,
                } => {
                    if self.request_id != Some(request_id) {
                        continue;
                    }
                    if self.icy {
                        if info.station_name.is_some() {
                            (self.spec.on_info)(
                                crate::metadata::Info {
                                    title: String::new(),
                                    station_name: info.station_name,
                                },
                                StationNamePolicy::FillEmpty,
                            );
                        }
                        continue;
                    }
                    if has_title && self.fallback.is_some() {
                        self.stop();
                    }
                    (self.spec.on_info)(info, StationNamePolicy::FillEmpty);
                    if !has_title && self.fallback.is_none() {
                        let epoch = *self.epoch.lock().unwrap();
                        let info_epoch = self.epoch.clone();
                        let strategy_epoch = self.epoch.clone();
                        let info_cancel = self.spec.cancel.clone();
                        let strategy_cancel = self.spec.cancel.clone();
                        let info = self.spec.on_info.clone();
                        let strategy = self.spec.on_strategy.clone();
                        self.fallback = Some(Provider::new(self.spec.poll).fallback_only().watch(
                            self.spec.url.clone(),
                            self.spec.hint.clone(),
                            move |value| {
                                let active = info_epoch.lock().unwrap();
                                if !info_cancel.load(Ordering::Acquire) && *active == epoch {
                                    info(value, StationNamePolicy::FillEmpty);
                                }
                            },
                            move |value| {
                                let active = strategy_epoch.lock().unwrap();
                                if !strategy_cancel.load(Ordering::Acquire) && *active == epoch {
                                    strategy(value);
                                }
                            },
                            Arc::new(AtomicBool::new(false)),
                        ));
                    }
                }
                StreamMetadataEvent::Info { request_id, info } => {
                    if self.request_id == Some(request_id) {
                        (self.spec.on_info)(info, StationNamePolicy::Replace);
                    }
                }
            }
        }
    }
}
impl Drop for SessionMetadata {
    fn drop(&mut self) {
        self.stop();
    }
}
