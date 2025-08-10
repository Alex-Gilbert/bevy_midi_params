use crate::MidiController;
use bevy::prelude::*;
use log::{info, warn};

#[cfg(feature = "ui")]
use bevy_egui::EguiPrimaryContextPass;

/// Main plugin for MIDI parameter integration
pub struct MidiParamsPlugin {
    /// Whether to auto-connect to MIDI on startup
    pub auto_connect: bool,
    /// Preferred MIDI controller name (partial match)
    pub preferred_controller: Option<String>,
    /// Whether to enable the built-in UI (requires bevy_egui)
    #[cfg(feature = "ui")]
    pub enable_ui: bool,
}

impl Default for MidiParamsPlugin {
    fn default() -> Self {
        Self {
            auto_connect: true,
            preferred_controller: None,
            #[cfg(feature = "ui")]
            enable_ui: false,  // Default to false, user must opt-in
        }
    }
}

impl MidiParamsPlugin {
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
    
    /// Enable the built-in UI (requires bevy_egui plugin to be added)
    #[cfg(feature = "ui")]
    pub fn with_ui(mut self) -> Self {
        self.enable_ui = true;
        self
    }
}

impl Plugin for MidiParamsPlugin {
    fn build(&self, app: &mut App) {
        // Insert MIDI controller resource
        app.insert_resource(MidiController::new(self.preferred_controller.clone()));

        // Auto-register all MidiParams types that have been defined
        for registration in inventory::iter::<MidiParamsRegistration> {
            info!("Auto-registering MIDI type: {}", registration.type_name);
            (registration.register_fn)(app);
        }

        if self.auto_connect {
            app.add_systems(Startup, setup_midi_input);
        }

        #[cfg(feature = "ui")]
        if self.enable_ui {
            app.add_systems(EguiPrimaryContextPass, crate::midi_control_ui);
        }

        app.add_systems(PreUpdate, update_midi_controller);
    }
}

/// Registration data for auto-discovered MidiParams types
#[derive(Debug)]
pub struct MidiParamsRegistration {
    pub type_name: &'static str,
    pub register_fn: fn(&mut App),
}

inventory::collect!(MidiParamsRegistration);

/// Trait for types that can be controlled via MIDI
pub trait MidiControllable {
    /// Update fields from MIDI input, returns true if any field changed
    fn update_from_midi(&mut self, cc: u8, value: f32) -> bool;

    /// Get all MIDI mappings for this type
    fn get_midi_mappings() -> Vec<crate::MidiMapping>;

    /// Render UI controls (egui or unit type if no UI)
    #[cfg(feature = "ui")]
    fn render_ui(&mut self, ui: &mut egui::Ui) -> bool;

    #[cfg(not(feature = "ui"))]
    fn render_ui(&mut self, ui: &mut ()) -> bool;

    /// Get type name
    fn get_type_name() -> &'static str;
}

/// Event emitted when MIDI parameters change
#[derive(Event, Debug, Clone)]
pub struct MidiParamChanged {
    pub type_name: String,
    pub cc: u8,
    pub value: f32,
}

/// Register a MidiParams type with the controller
pub fn register_midi_type<T: Resource + MidiControllable + Default>(app: &mut App) {
    let type_name = T::get_type_name();

    let world = app.world_mut();

    // Ensure resource exists
    if !world.contains_resource::<T>() {
        world.init_resource::<T>();
    }

    // Register mappings with the controller
    if let Some(mut midi_controller) = world.get_resource_mut::<MidiController>() {
        for mapping in T::get_midi_mappings() {
            midi_controller.register_mapping(mapping);
        }
        midi_controller.register_type(type_name);
    }

    // Add event type if not already added
    if !app.world().contains_resource::<Events<MidiParamChanged>>() {
        app.add_event::<MidiParamChanged>();
    }

    // Add systems for this type
    app.add_systems(Update, update_params_from_midi::<T>);
}

// ===== SYSTEM IMPLEMENTATIONS =====

/// Setup MIDI input connection
fn setup_midi_input(mut midi_controller: ResMut<MidiController>) {
    match midi_controller.connect_midi() {
        Ok(()) => info!("MIDI connection established"),
        Err(e) => warn!("Failed to connect MIDI: {}", e),
    }
}

fn update_midi_controller(mut midi_controller: ResMut<MidiController>) {
    midi_controller.update_values();
}

/// Generic system to update parameters from MIDI
fn update_params_from_midi<T: Resource + MidiControllable>(
    midi_controller: Res<MidiController>,
    mut params: ResMut<T>,
    mut events: EventWriter<MidiParamChanged>,
) {
    // Update from MIDI input
    for mapping in T::get_midi_mappings() {
        if let Some(normalized_value) = midi_controller.values.get(&mapping.cc).copied() {
            let scaled_value = mapping.scale_value(normalized_value);

            // For range controls, pass the scaled value directly
            // For buttons, we pass the normalized value (> 0.5 triggers toggle)
            let value_to_pass = match mapping.control_type {
                crate::ControlType::Range { .. } => scaled_value,
                crate::ControlType::Button => normalized_value,
            };

            if params.update_from_midi(mapping.cc, value_to_pass) {
                // Emit event for external systems to react to
                events.write(MidiParamChanged {
                    type_name: T::get_type_name().to_string(),
                    cc: mapping.cc,
                    value: value_to_pass,
                });
            }
        }
    }
}
