#[cfg(feature = "live-audio")]
pub mod audio;
pub mod config;
pub mod controller;
#[cfg(feature = "live-audio")]
pub mod metadata;
pub mod model;
#[cfg(not(feature = "live-audio"))]
#[path = "audio/builtin_presets.rs"]
pub mod presets;
#[cfg(feature = "live-audio")]
pub use audio::builtin_presets as presets;

#[cfg(feature = "live-audio")]
pub(crate) mod net;

#[cfg(feature = "live-audio")]
pub mod session;

#[cfg(all(test, feature = "live-audio"))]
mod backend_tests;

pub mod startup;

pub mod launch;

mod config_document;
pub mod profile;

#[cfg(test)]
mod profile_tests;

pub mod startup_failure;

pub mod webview_prerequisite;

pub mod config_issue;
