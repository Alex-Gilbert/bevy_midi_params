//! # bevy_midi_params
//!
//! Hardware MIDI controller integration for live parameter tweaking in Bevy games.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_midi_params::prelude::*;
//!
//! #[derive(Resource, MidiParams)]
//! struct GameSettings {
//!     #[midi(1, 0.0..1.0)]
//!     pub player_speed: f32,
//!     
//!     #[midi(2, button)]
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
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(MidiParamsPlugin::default())
//!         .run();
//! }
//! ```

mod controller;
mod mapping;
mod persistence;
mod plugin;
mod error;

#[cfg(feature = "ui")]
mod ui;

// Re-export everything users need
pub use bevy_midi_params_derive::MidiParams;
pub use controller::*;
pub use mapping::*;
pub use persistence::*;
pub use plugin::*;
pub use error::*;

#[cfg(feature = "ui")]
pub use ui::*;

// For auto-registration
pub use inventory;

/// Prelude module for easy imports
pub mod prelude {
    pub use crate::{
        MidiParams,
        MidiParamsPlugin,
        MidiMapping,
        MidiController,
        MidiError,
    };
    
    #[cfg(feature = "ui")]
    pub use crate::ui::*;
}
