use crate::{MidiError, MidiMapping, MidiResult};
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
    pub fn new(preferred_controller: Option<String>) -> Self {
        Self {
            values: HashMap::new(),
            mappings: HashMap::new(),
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

    /// Register a type name for tracking
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
                .find(|p| {
                    midi_in
                        .port_name(p)
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&preferred.to_lowercase())
                })
                .or_else(|| in_ports.first())
                .ok_or(MidiError::NoInputPorts)?
        } else {
            &in_ports[0]
        };

        let port_name = midi_in
            .port_name(in_port)
            .unwrap_or_else(|_| "Unknown".to_string());

        info!("Connecting to MIDI input: {}", port_name);

        // Create shared state for the callback
        let changed_values = Arc::new(Mutex::new(HashMap::new()));
        let changed_values_clone = changed_values.clone();

        // Connect with a callback that updates our shared state
        let connection = midi_in
            .connect(
                in_port,
                "bevy_midi_input",
                move |_timestamp, message, _| {
                    // Parse MIDI CC messages
                    if message.len() >= 3 && (message[0] & 0xF0) == 0xB0 {
                        let cc = message[1];
                        let value = message[2] as f32 / 127.0;

                        if let Ok(mut values) = changed_values_clone.lock() {
                            values.insert(cc, value);
                        }

                        debug!("MIDI CC {} = {:.2}", cc, value);
                    }
                },
                (),
            )
            .map_err(|e| MidiError::ConnectionFailed(format!("Failed to connect: {}", e)))?;

        // Store the connection to keep it alive
        self._connection = Some(Arc::new(Mutex::new(Some(connection))));
        self._changed_values = Some(changed_values);

        info!("MIDI connection established to: {}", port_name);
        Ok(())
    }

    /// Update values from the MIDI connection
    pub fn update_values(&mut self) {
        if let Some(ref changed_values) = self._changed_values {
            if let Ok(mut values) = changed_values.lock() {
                // Move all changed values into our main values map
                for (cc, value) in values.drain() {
                    self.values.insert(cc, value);
                }
            }
        }
    }

    /// Disconnect MIDI input
    pub fn disconnect(&mut self) {
        self._connection = None;
        self._changed_values = None;
        info!("MIDI connection closed");
    }

    /// List available MIDI input ports
    pub fn list_ports() -> MidiResult<Vec<String>> {
        let midi_in = MidiInput::new("bevy_midi_params_list").map_err(|e| {
            MidiError::ConnectionFailed(format!("Failed to create MIDI input: {}", e))
        })?;

        let ports = midi_in.ports();
        let mut port_names = Vec::new();

        for port in ports {
            if let Ok(name) = midi_in.port_name(&port) {
                port_names.push(name);
            }
        }

        Ok(port_names)
    }
}