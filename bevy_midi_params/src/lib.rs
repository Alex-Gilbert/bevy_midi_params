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
mod persistence_plugin;
mod midi_plugin;

#[cfg(feature = "ui")]
mod ui;

// Re-export everything users need
pub use bevy_midi_params_derive::MidiParams;
#[cfg(feature = "midi")]
pub use controller::*;
pub use mapping::*;
pub use persistence::*;
pub use plugin::*;
pub use error::*;
pub use persistence_plugin::*;
pub use midi_plugin::*;

#[cfg(feature = "ui")]
pub use ui::*;

// For auto-registration
pub use inventory;

/// Prelude module for easy imports
pub mod prelude {
    pub use crate::{
        MidiParams,
        MidiParamsPlugin, // Legacy plugin (deprecated)
        ParamsPersistencePlugin,
        MidiControlPlugin,
        MidiMapping,
        MidiError,
        PersistableParams,
    };
    
    #[cfg(feature = "midi")]
    pub use crate::MidiController;
    
    #[cfg(feature = "ui")]
    pub use crate::ui::*;
}

/// Convenience function to add all plugins for development builds
/// Includes both persistence and MIDI control
#[cfg(feature = "midi")]
pub fn dev_plugins() -> (ParamsPersistencePlugin, MidiControlPlugin) {
    (
        ParamsPersistencePlugin::default(),
        MidiControlPlugin::default(),
    )
}

/// Convenience function to add all plugins for development builds (no MIDI feature)
#[cfg(not(feature = "midi"))]
pub fn dev_plugins() -> ParamsPersistencePlugin {
    ParamsPersistencePlugin::default()
}

/// Convenience function to add plugins for production builds
/// Only includes persistence (no MIDI dependencies)
pub fn prod_plugins() -> ParamsPersistencePlugin {
    ParamsPersistencePlugin::default()
}

/// Convenience function with custom persistence file for development
#[cfg(feature = "midi")]
pub fn dev_plugins_with_file(persist_file: impl Into<String>) -> (ParamsPersistencePlugin, MidiControlPlugin) {
    (
        ParamsPersistencePlugin::default().with_persist(persist_file),
        MidiControlPlugin::default(),
    )
}

/// Convenience function with custom persistence file for development (no MIDI feature)
#[cfg(not(feature = "midi"))]
pub fn dev_plugins_with_file(persist_file: impl Into<String>) -> ParamsPersistencePlugin {
    ParamsPersistencePlugin::default().with_persist(persist_file)
}

/// Convenience function with custom persistence file for production
pub fn prod_plugins_with_file(persist_file: impl Into<String>) -> ParamsPersistencePlugin {
    ParamsPersistencePlugin::default().with_persist(persist_file)
}
