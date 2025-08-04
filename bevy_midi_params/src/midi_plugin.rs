#[cfg(feature = "midi")]
use crate::{MidiController, PersistableParams, PersistenceController};
#[cfg(feature = "midi")]
use bevy::prelude::*;
#[cfg(feature = "midi")]
use log::{debug, error, info, warn};

/// MIDI control plugin for development builds (requires "midi" feature)
#[cfg(feature = "midi")]
pub struct MidiControlPlugin {
    /// Whether to auto-connect to MIDI on startup
    pub auto_connect: bool,
    /// Preferred MIDI controller name (partial match)
    pub preferred_controller: Option<String>,
}

#[cfg(feature = "midi")]
impl Default for MidiControlPlugin {
    fn default() -> Self {
        Self {
            auto_connect: true,
            preferred_controller: None,
        }
    }
}

#[cfg(feature = "midi")]
impl MidiControlPlugin {
    /// Create new plugin with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set preferred MIDI controller name (partial match)
    pub fn with_controller(mut self, controller_name: impl Into<String>) -> Self {
        self.preferred_controller = Some(controller_name.into());
        self
    }

    /// Disable auto MIDI connection (useful for testing)
    pub fn no_auto_connect(mut self) -> Self {
        self.auto_connect = false;
        self
    }
}

#[cfg(feature = "midi")]
impl Plugin for MidiControlPlugin {
    fn build(&self, app: &mut App) {
        // Insert MIDI controller resource
        app.insert_resource(MidiController::new(
            None, // Persistence is handled by PersistenceController
            self.preferred_controller.clone(),
        ));

        // Register MIDI mappings for all registered types
        for registration in inventory::iter::<crate::ParamsRegistration> {
            info!("Registering MIDI mappings for: {}", registration.type_name);
            // This will be handled by the register_midi_mappings system
        }

        if self.auto_connect {
            app.add_systems(Startup, setup_midi_input);
        }

        app.add_systems(PreUpdate, update_midi_controller);
        app.add_systems(Update, register_midi_mappings_system);
    }
}

#[cfg(feature = "midi")]
/// Setup MIDI input connection
fn setup_midi_input(mut midi_controller: ResMut<MidiController>) {
    match midi_controller.connect_midi() {
        Ok(()) => info!("MIDI connection established"),
        Err(e) => warn!("Failed to connect MIDI: {}", e),
    }
}

#[cfg(feature = "midi")]
fn update_midi_controller(mut midi_controller: ResMut<MidiController>) {
    midi_controller.update_values();
}

#[cfg(feature = "midi")]
/// System to register MIDI mappings from all persistable types
fn register_midi_mappings_system(world: &mut World) {
    // This system runs once to register all MIDI mappings
    // We'll implement this as a run-once system
    static mut REGISTERED: bool = false;
    
    unsafe {
        if REGISTERED {
            return;
        }
        REGISTERED = true;
    }

    if let Some(mut midi_controller) = world.get_resource_mut::<MidiController>() {
        for registration in inventory::iter::<crate::ParamsRegistration> {
            info!("Registering MIDI mappings for: {}", registration.type_name);
            // The actual mapping registration will be handled by each type's registration function
        }
    }
}

/// Register MIDI control for a PersistableParams type (only available with "midi" feature)
#[cfg(feature = "midi")]
pub fn register_midi_control<T: Resource + PersistableParams + Default>(app: &mut App) {
    let type_name = T::get_type_name();

    let world = app.world_mut();

    // Register mappings with the MIDI controller
    if let Some(mut midi_controller) = world.get_resource_mut::<MidiController>() {
        for mapping in T::get_param_mappings() {
            // Only register mappings that have MIDI control enabled
            if mapping.has_midi_control() {
                midi_controller.register_mapping(mapping);
            }
        }
        midi_controller.register_type(type_name);
    }

    // Add MIDI update system for this type
    app.add_systems(Update, update_from_midi::<T>);
}

/// Generic system to update parameters from MIDI input
#[cfg(feature = "midi")]
fn update_from_midi<T: Resource + PersistableParams>(
    midi_controller: Res<MidiController>,
    mut params: ResMut<T>,
    persistence_controller: Res<PersistenceController>,
) {
    let mut changed = false;

    // Update from MIDI input
    for mapping in T::get_param_mappings() {
        // Only process mappings that have MIDI control enabled
        if let Some(cc) = mapping.cc {
            if let Some(normalized_value) = midi_controller.values.get(&cc).copied() {
                let scaled_value = mapping.scale_value(normalized_value);

                // For range controls, pass the scaled value directly
                // For buttons, we pass the normalized value (> 0.5 triggers toggle)
                let value_to_pass = match mapping.control_type {
                    crate::ControlType::Range { .. } => scaled_value,
                    crate::ControlType::Button => normalized_value,
                };

                if params.update_from_midi(cc, value_to_pass) {
                    changed = true;
                }
            }
        }
    }

    // Auto-save when values change via MIDI
    if changed {
        if let Err(e) = save_params_to_file(&persistence_controller, &*params) {
            error!("Failed to save MIDI parameter changes: {}", e);
        }
    }
}

/// Helper function to save parameters to file (MIDI version)
#[cfg(feature = "midi")]
fn save_params_to_file<T: PersistableParams>(
    controller: &PersistenceController,
    params: &T,
) -> crate::MidiResult<()> {
    let mut persist_file = controller.load_persist_file()?;
    let new_data = params.to_persist_data();

    persist_file.set_type_data(T::get_type_name().to_string(), new_data);
    controller.save_persist_file(&mut persist_file)?;

    Ok(())
}

// Stub implementations for when MIDI feature is disabled
#[cfg(not(feature = "midi"))]
pub struct MidiControlPlugin;

#[cfg(not(feature = "midi"))]
impl bevy::prelude::Plugin for MidiControlPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {
        // No-op when MIDI feature is disabled
    }
}

#[cfg(not(feature = "midi"))]
impl MidiControlPlugin {
    pub fn new() -> Self { Self }
    pub fn with_controller(self, _controller_name: impl Into<String>) -> Self { self }
    pub fn no_auto_connect(self) -> Self { self }
}

#[cfg(not(feature = "midi"))]
impl Default for MidiControlPlugin {
    fn default() -> Self { Self }
}

#[cfg(not(feature = "midi"))]
pub fn register_midi_control<T: bevy::prelude::Resource + crate::PersistableParams + Default>(_app: &mut bevy::prelude::App) {
    // No-op when MIDI feature is disabled
}
