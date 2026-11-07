//! This module provides the primary public interface for the library.
//!
//! Modules:
//! - `direct`: Handles operations related to direct interactions.
//! - `helpers`: Contains utility functions and helper methods used across the library.
//! - `model`: Defines core data structures and types, such as `Info` and `StrategyHint`.
//! - `provider`: Implements the `Provider` trait for external data or resource fetching.
//! - `sibling`: Contains logic related to sibling or parallel processing.
//! - `stabilizer`: Includes the `TitleStabilizer` for maintaining title stability.
//! - `status_json`: Handles JSON serialization and deserialization for status information.
//!
//! Re-exports:
//! - `Info`: A core model struct, likely representing some primary information structure.
//! - `StrategyHint`: A utility associated with strategies or approach selection.
//! - `Provider`: The main trait or struct for interacting with data providers.
//! - `TitleStabilizer`: A utility class for managing and stabilizing titles or related entities.
//!
//! This module organizes the library into distinct functional components, making it
//! easier for developers to work with each feature set independently while still
//! providing a unified interface at the top level.
pub mod direct;
pub mod helpers;
pub(crate) mod icy;
pub mod model;
pub mod provider;
pub mod sibling;
pub mod stabilizer;
pub mod status_json;

pub use model::{Info, StrategyHint};
pub use provider::Provider;
pub use stabilizer::TitleStabilizer;
