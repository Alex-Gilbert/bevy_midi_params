//! # bevy_midi_params
//!
//! Hardware MIDI controller integration for live parameter tweaking in Bevy games.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # use bevy::prelude::*;
//! # use bevy_midi_params::prelude::*;
//! #
//! #[derive(Resource, MidiParams)]
//! struct GameSettings {
//!     #[midi(1, 0.0..1.0)]
//!     pub player_speed: f32,
//!     
//!     #[midi(33)]
//!     pub debug_mode: bool,
//! }
//!
//! impl Default for GameSettings {
//!     fn default() -> Self {
//!         Self {
//!             player_speed: 5.0,
//!             debug_mode: false,
//!         }
//!     }
//! }
//!
//! # fn main() {
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(MidiParamsPlugin::default())
//!     .run();
//! # }
//! ```

mod controller;
mod error;
mod mapping;
mod plugin;

// Re-export everything users need
pub use bevy_midi_params_derive::MidiParams;
pub use controller::*;
pub use error::*;
pub use mapping::*;
pub use plugin::*;

// For auto-registration
pub use inventory;

/// Prelude module for easy imports
pub mod prelude {
    pub use crate::{MidiController, MidiError, MidiMapping, MidiParams, MidiParamsPlugin};
}
