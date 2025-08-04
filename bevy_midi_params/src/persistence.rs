use crate::MidiResult;
use bevy::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Data structure for persisting parameter values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistData {
    pub values: HashMap<String, serde_json::Value>,
}

impl PersistData {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn insert<T: serde::Serialize>(&mut self, key: impl Into<String>, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.values.insert(key.into(), json_value);
        }
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.values
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

impl Default for PersistData {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete persistence file format
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MidiPersistFile {
    #[serde(flatten)]
    pub type_data: HashMap<String, PersistData>,
    pub last_saved: String,
    pub version: String,
}

impl MidiPersistFile {
    pub fn new() -> Self {
        Self {
            type_data: HashMap::new(),
            last_saved: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> MidiResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path).map_err(|e| {
            crate::MidiError::PersistenceError(format!("Failed to read file: {}", e))
        })?;

        // Try RON first, fallback to JSON
        if path.extension().map_or(false, |ext| ext == "ron") {
            ron::from_str(&content)
                .map_err(|e| crate::MidiError::PersistenceError(format!("RON parse error: {}", e)))
        } else {
            serde_json::from_str(&content)
                .map_err(|e| crate::MidiError::PersistenceError(format!("JSON parse error: {}", e)))
        }
    }

    pub fn save_to_file(&mut self, path: impl AsRef<Path>) -> MidiResult<()> {
        let path = path.as_ref();

        // Update timestamp
        self.last_saved = chrono::Utc::now().to_rfc3339();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::MidiError::PersistenceError(format!("Failed to create directory: {}", e))
            })?;
        }

        let content = if path.extension().map_or(false, |ext| ext == "ron") {
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).map_err(|e| {
                crate::MidiError::PersistenceError(format!("RON serialization error: {}", e))
            })?
        } else {
            serde_json::to_string_pretty(self).map_err(|e| {
                crate::MidiError::PersistenceError(format!("JSON serialization error: {}", e))
            })?
        };

        fs::write(path, content).map_err(|e| {
            crate::MidiError::PersistenceError(format!("Failed to write file: {}", e))
        })?;

        debug!("Saved MIDI settings to {}", path.display());
        Ok(())
    }

    pub fn get_type_data(&self, type_name: &str) -> Option<&PersistData> {
        self.type_data.get(type_name)
    }

    pub fn set_type_data(&mut self, type_name: String, data: PersistData) {
        self.type_data.insert(type_name, data);
    }
}
