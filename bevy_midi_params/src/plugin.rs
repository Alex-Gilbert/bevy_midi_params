use crate::{midi_control_ui, MidiController, MidiPersistFile, MidiResult, PersistData};
use bevy::prelude::*;
use log::{debug, error, info, warn};

/// Main plugin for MIDI parameter integration
pub struct MidiParamsPlugin {
    /// Path to persistence file
    pub persist_file: Option<String>,
    /// Whether to auto-connect to MIDI on startup
    pub auto_connect: bool,
    /// Preferred MIDI controller name (partial match)
    pub preferred_controller: Option<String>,
}

impl Default for MidiParamsPlugin {
    fn default() -> Self {
        Self {
            persist_file: None,
            auto_connect: true,
            preferred_controller: None,
        }
    }
}

impl MidiParamsPlugin {
    /// Create new plugin with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom persistence file path
    pub fn with_persist(mut self, persist_file: impl Into<String>) -> Self {
        self.persist_file = Some(persist_file.into());
        self
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

impl Plugin for MidiParamsPlugin {
    fn build(&self, app: &mut App) {
        // Insert MIDI controller resource
        app.insert_resource(MidiController::new(
            self.persist_file.clone(),
            self.preferred_controller.clone(),
        ));

        // Auto-register all MidiParams types that have been defined
        for registration in inventory::iter::<MidiParamsRegistration> {
            info!("Auto-registering MIDI type: {}", registration.type_name);
            (registration.register_fn)(app);
        }

        if self.auto_connect {
            app.add_systems(Startup, setup_midi_input);
        }

        // #[cfg(feature = "ui")]
        // app.add_systems(Update, midi_control_ui);

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

    /// Get type name for persistence
    fn get_type_name() -> &'static str;

    /// Convert to persistence data
    fn to_persist_data(&self) -> PersistData;

    /// Load from persistence data
    fn from_persist_data(&mut self, data: &PersistData);
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

    // Add systems for this type
    app.add_systems(
        Update,
        (update_and_persist_params::<T>, save_on_ui_change::<T>),
    );
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

/// Load persisted values for all registered types
fn load_all_persisted_values(world: &mut World) {
    let persist_file = {
        let midi_controller = world.resource::<MidiController>();
        match midi_controller.load_persist_file() {
            Ok(file) => file,
            Err(e) => {
                warn!("Failed to load persistence file: {}", e);
                return;
            }
        }
    };

    // Load data for each registered type
    for registration in inventory::iter::<MidiParamsRegistration> {
        if let Some(data) = persist_file.get_type_data(registration.type_name) {
            // This is a bit tricky - we need to call from_persist_data on the right resource
            // For now, we'll do a runtime dispatch. In a real implementation,
            // this could be improved with a trait object or type registry.
            info!("Would load {} from persistence", registration.type_name);
        }
    }
}

/// Generic system to update parameters from MIDI and auto-save changes
fn update_and_persist_params<T: Resource + MidiControllable>(
    midi_controller: Res<MidiController>,
    mut params: ResMut<T>,
) {
    let mut changed = false;

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
                changed = true;
            }
        }
    }

    // Auto-save when values change via MIDI
    if changed {
        if let Err(e) = save_params_to_file(&midi_controller, &*params) {
            error!("Failed to save MIDI parameters: {}", e);
        }
    }
}

/// Save parameters when UI changes them
fn save_on_ui_change<T: Resource + MidiControllable>(
    midi_controller: Res<MidiController>,
    params: Res<T>,
) {
    if params.is_changed() && !params.is_added() {
        if let Err(e) = save_params_to_file(&midi_controller, &*params) {
            error!("Failed to save UI parameter changes: {}", e);
        } else {
            debug!("Auto-saved {} changes", T::get_type_name());
        }
    }
}

/// Helper function to save parameters to file
fn save_params_to_file<T: MidiControllable>(
    midi_controller: &MidiController,
    params: &T,
) -> MidiResult<()> {
    let mut persist_file = midi_controller.load_persist_file()?;
    let new_data = params.to_persist_data();

    persist_file.set_type_data(T::get_type_name().to_string(), new_data);
    midi_controller.save_persist_file(&mut persist_file)?;

    Ok(())
}
