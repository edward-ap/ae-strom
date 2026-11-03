//! This module serves as the entry point for audio-related functionalities.
//! It provides access to various submodules and exposes key components for
//! implementing and managing an audio processing system.
//!
//! # Modules
//! - `builtin_presets`: Contains predefined audio presets that can be used to
//!   initialize or configure the audio engine with common settings or effects.
//! - `dsp`: Contains utilities and algorithms related to Digital Signal Processing
//!   (DSP), including audio processing functions and effects.
//! - `engine`: Contains the core implementation of the audio engine, managing
//!   audio processing, playback, and other core functionalities.
//!
//! # Re-exports
//! - `AudioEngine`: The primary interface to interact with the audio engine,
//!   providing functionality for managing audio streams, applying presets,
//!   and utilizing DSP tools.
//!
//! # Usage
//! This module and its submodules can be used to create powerful audio
//! applications, such as digital audio workstations, sound effect processors,
//! or other custom audio solutions. To get started, use the `AudioEngine`
//! re-export from this module.
//!
//! ```rust
//! use ae_strom::audio::AudioEngine;
//!
//! let engine = AudioEngine::new();
//! // configure presets / start playback as needed
//! ```
//!
//! # Notes
//! Ensure that your environment supports the required audio backend and dependencies
//! to enable seamless integration and execution of audio functionalities.
pub mod builtin_presets;
pub mod dsp;
pub mod engine;

pub use engine::AudioEngine;
