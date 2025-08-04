use crate::{MidiError, MidiMapping, MidiPersistFile, MidiResult};
use bevy::prelude::*;
use log::{debug, info};
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Resource that manages MIDI controller input and state
#[derive(Resource)]
pub struct MidiController {
    /// Current MIDI CC values (normalized 0.0-1.0)
    pub values: HashMap<u8, f32>,
    /// All registered MIDI mappings
    mappings: HashMap<u8, MidiMapping>,
    /// Path to persistence file
    persist_file_path: String,
    /// List of registered type names
    registered_types: Vec<&'static str>,
    /// MIDI connection (kept alive)
    _connection: Option<Arc<Mutex<Option<MidiInputConnection<()>>>>>,
    /// A shared pointer to values which are updated by the connection
    _changed_values: Option<Arc<Mutex<HashMap<u8, f32>>>>,
    /// Preferred MIDI controller name (partial match)
    preferred_controller: Option<String>,
}

impl MidiController {
    pub fn new(persist_path: Option<String>, preferred_controller: Option<String>) -> Self {
        Self {
            values: HashMap::new(),
            mappings: HashMap::new(),
            persist_file_path: persist_path.unwrap_or_else(|| "midi_settings.ron".to_string()),
            registered_types: Vec::new(),
            _connection: None,
            _changed_values: None,
            preferred_controller,
        }
    }

    /// Get current value for a CC (normalized 0.0-1.0)
    pub fn get_value(&self, cc: u8) -> f32 {
        self.values.get(&cc).copied().unwrap_or(0.0)
    }

    /// Get the number of registered types
    pub fn number_of_registered_types(&self) -> usize {
        self.registered_types.len()
    }

    /// Get scaled value for a CC using the registered mapping
    pub fn get_scaled_value(&self, cc: u8) -> Option<f32> {
        let mapping = self.mappings.get(&cc)?;
        let normalized = self.get_value(cc);
        Some(mapping.scale_value(normalized))
    }

    /// Register a MIDI mapping
    pub fn register_mapping(&mut self, mapping: MidiMapping) {
        self.values.insert(mapping.cc, 0.0);
        self.mappings.insert(mapping.cc, mapping);
    }

    /// Register a type name for persistence tracking
    pub fn register_type(&mut self, type_name: &'static str) {
        if !self.registered_types.contains(&type_name) {
            self.registered_types.push(type_name);
            info!("Registered MIDI type: {}", type_name);
        }
    }

    /// Get all registered mappings
    pub fn get_mappings(&self) -> &HashMap<u8, MidiMapping> {
        &self.mappings
    }

    /// Load persistence file
    pub fn load_persist_file(&self) -> MidiResult<MidiPersistFile> {
        MidiPersistFile::load_from_file(&self.persist_file_path)
    }

    /// Save persistence file
    pub fn save_persist_file(&self, data: &mut MidiPersistFile) -> MidiResult<()> {
        data.save_to_file(&self.persist_file_path)
    }

    /// Connect to MIDI input device
    pub fn connect_midi(&mut self) -> MidiResult<()> {
        let mut midi_in = MidiInput::new("bevy_midi_params").map_err(|e| {
            MidiError::ConnectionFailed(format!("Failed to create MIDI input: {}", e))
        })?;

        midi_in.ignore(Ignore::None);

        let in_ports = midi_in.ports();
        if in_ports.is_empty() {
            return Err(MidiError::NoInputPorts);
        }

        // Use preferred controller if specified, otherwise first available
        let in_port = if let Some(ref preferred) = self.preferred_controller {
            in_ports
                .iter()
                .find(|port| {
                    midi_in
                        .port_name(port)
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&preferred.to_lowercase())
                })
                .or_else(|| in_ports.first())
        } else {
            in_ports.first()
        }
        .unwrap();

        let port_name = midi_in.port_name(in_port).unwrap_or("Unknown".to_string());
        info!("Connecting to MIDI port: {}", port_name);

        // Shared values for the callback
        let raw_values = Arc::new(Mutex::new(HashMap::<u8, f32>::new()));
        self._changed_values = Some(raw_values.clone());
        let values_clone = raw_values.clone();

        let connection = midi_in
            .connect(
                in_port,
                "bevy-midi-params",
                move |_stamp, message, _| {
                    if message.len() >= 3 && message[0] == 0xB0 {
                        // Control Change
                        let cc = message[1];
                        let value = message[2] as f32 / 127.0; // Normalize to 0.0-1.0

                        if let Ok(mut values) = values_clone.lock() {
                            values.insert(cc, value);
                        }

                        info!("MIDI CC {}: {:.3}", cc, value);
                    }
                },
                (),
            )
            .map_err(|e| MidiError::ConnectionFailed(format!("Connection failed: {}", e)))?;

        self._connection = Some(Arc::new(Mutex::new(Some(connection))));
        Ok(())
    }

    /// Update values from MIDI (called by system)
    pub(crate) fn update_values(&mut self) {
        let Some(changed_values) = &self._changed_values else {
            return;
        };

        if let Ok(mut changed_values_lock) = changed_values.lock() {
            // Move all values out instead of cloning
            for (cc, value) in changed_values_lock.drain() {
                self.values.insert(cc, value);
            }
        }
    }
}

impl Default for MidiController {
    fn default() -> Self {
        Self::new(None, None)
    }
}
