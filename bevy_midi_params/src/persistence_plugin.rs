use crate::{MidiResult, PersistData};
use bevy::prelude::*;
use log::{debug, error, info, warn};

/// Core plugin for parameter persistence (always available)
#[derive(Default)]
pub struct ParamsPersistencePlugin {
    /// Path to persistence file
    pub persist_file: Option<String>,
}


impl ParamsPersistencePlugin {
    /// Create new plugin with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom persistence file path
    pub fn with_persist(mut self, persist_file: impl Into<String>) -> Self {
        self.persist_file = Some(persist_file.into());
        self
    }
}

impl Plugin for ParamsPersistencePlugin {
    fn build(&self, app: &mut App) {
        // Insert persistence controller resource
        app.insert_resource(PersistenceController::new(self.persist_file.clone()));

        // Auto-register all PersistableParams types that have been defined
        for registration in inventory::iter::<ParamsRegistration> {
            info!("Auto-registering persistable type: {}", registration.type_name);
            (registration.register_fn)(app);
        }

        // Load persisted values on startup
        app.add_systems(Startup, load_all_persisted_values);
    }
}

/// Registration data for auto-discovered PersistableParams types
#[derive(Debug)]
pub struct ParamsRegistration {
    pub type_name: &'static str,
    pub register_fn: fn(&mut App),
}

inventory::collect!(ParamsRegistration);

/// Trait for types that can be persisted and optionally controlled via MIDI
pub trait PersistableParams {
    /// Update fields from MIDI input (if MIDI feature enabled), returns true if any field changed
    #[cfg(feature = "midi")]
    fn update_from_midi(&mut self, cc: u8, value: f32) -> bool;

    /// Get all parameter mappings for this type
    fn get_param_mappings() -> Vec<crate::MidiMapping>;

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

/// Controller for parameter persistence (lightweight, no MIDI dependencies)
#[derive(Resource)]
pub struct PersistenceController {
    /// Path to persistence file
    pub persist_file: Option<String>,
    /// Registered type names
    pub registered_types: Vec<String>,
}

impl PersistenceController {
    pub fn new(persist_file: Option<String>) -> Self {
        Self {
            persist_file,
            registered_types: Vec::new(),
        }
    }

    pub fn register_type(&mut self, type_name: &str) {
        if !self.registered_types.contains(&type_name.to_string()) {
            self.registered_types.push(type_name.to_string());
        }
    }

    pub fn load_persist_file(&self) -> MidiResult<crate::MidiPersistFile> {
        let path = self.persist_file.as_deref().unwrap_or("params.ron");
        crate::MidiPersistFile::load_from_file(path)
    }

    pub fn save_persist_file(&self, persist_file: &mut crate::MidiPersistFile) -> MidiResult<()> {
        let path = self.persist_file.as_deref().unwrap_or("params.ron");
        persist_file.save_to_file(path)
    }
}

/// Register a PersistableParams type with the persistence controller
pub fn register_persistable_type<T: Resource + PersistableParams + Default>(app: &mut App) {
    let type_name = T::get_type_name();

    let world = app.world_mut();

    // Ensure resource exists
    if !world.contains_resource::<T>() {
        world.init_resource::<T>();
    }

    // Register type with the persistence controller
    if let Some(mut controller) = world.get_resource_mut::<PersistenceController>() {
        controller.register_type(type_name);
    }

    // Add systems for this type
    app.add_systems(Update, save_on_change::<T>);
}

// ===== SYSTEM IMPLEMENTATIONS =====

/// Load persisted values for all registered types on startup
fn load_all_persisted_values(world: &mut World) {
    let persist_file = {
        let controller = world.resource::<PersistenceController>();
        match controller.load_persist_file() {
            Ok(file) => file,
            Err(e) => {
                warn!("Failed to load persistence file: {}", e);
                return;
            }
        }
    };

    // Load data for each registered type
    for registration in inventory::iter::<ParamsRegistration> {
        if let Some(data) = persist_file.get_type_data(registration.type_name) {
            info!("Loading {} from persistence", registration.type_name);
            // The actual loading will be handled by the specific type's system
            // This is a placeholder for the loading mechanism
        }
    }
}

/// Save parameters when they change (UI or other modifications)
fn save_on_change<T: Resource + PersistableParams>(
    controller: Res<PersistenceController>,
    params: Res<T>,
) {
    if params.is_changed() && !params.is_added() {
        if let Err(e) = save_params_to_file(&controller, &*params) {
            error!("Failed to save parameter changes: {}", e);
        } else {
            debug!("Auto-saved {} changes", T::get_type_name());
        }
    }
}

/// Helper function to save parameters to file
fn save_params_to_file<T: PersistableParams>(
    controller: &PersistenceController,
    params: &T,
) -> MidiResult<()> {
    let mut persist_file = controller.load_persist_file()?;
    let new_data = params.to_persist_data();

    persist_file.set_type_data(T::get_type_name().to_string(), new_data);
    controller.save_persist_file(&mut persist_file)?;

    Ok(())
}
