//! Automatic persistence for Bevy resources with change detection.
//!
//! This crate provides automatic saving and loading of Bevy resources,
//! with support for multiple serialization formats and change detection.

use bevy::prelude::*;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;

pub mod prelude {
    pub use crate::{
        AutoPersist, PersistPlugin, Persistable, PersistData, PersistFile,
        PersistError, PersistResult,
    };
}

/// Result type for persistence operations
pub type PersistResult<T> = Result<T, PersistError>;

/// Errors that can occur during persistence operations
#[derive(Debug, Clone)]
pub enum PersistError {
    /// Failed to read/write file
    IoError(String),
    /// Failed to serialize/deserialize
    SerializationError(String),
    /// Resource not found
    ResourceNotFound(String),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::ResourceNotFound(e) => write!(f, "Resource not found: {}", e),
        }
    }
}

impl std::error::Error for PersistError {}

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
pub struct PersistFile {
    #[serde(flatten)]
    pub type_data: HashMap<String, PersistData>,
    pub last_saved: String,
    pub version: String,
}

impl PersistFile {
    pub fn new() -> Self {
        Self {
            type_data: HashMap::new(),
            last_saved: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> PersistResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| PersistError::IoError(format!("Failed to read file: {}", e)))?;

        // Try RON first, fallback to JSON
        if path.extension().map_or(false, |ext| ext == "ron") {
            ron::from_str(&content)
                .map_err(|e| PersistError::SerializationError(format!("RON parse error: {}", e)))
        } else {
            serde_json::from_str(&content)
                .map_err(|e| PersistError::SerializationError(format!("JSON parse error: {}", e)))
        }
    }

    pub fn save_to_file(&mut self, path: impl AsRef<Path>) -> PersistResult<()> {
        let path = path.as_ref();

        // Update timestamp
        self.last_saved = chrono::Utc::now().to_rfc3339();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PersistError::IoError(format!("Failed to create directory: {}", e)))?;
        }

        let content = if path.extension().map_or(false, |ext| ext == "ron") {
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .map_err(|e| PersistError::SerializationError(format!("RON serialization error: {}", e)))?
        } else {
            serde_json::to_string_pretty(self)
                .map_err(|e| PersistError::SerializationError(format!("JSON serialization error: {}", e)))?
        };

        fs::write(path, content)
            .map_err(|e| PersistError::IoError(format!("Failed to write file: {}", e)))?;

        debug!("Saved settings to {}", path.display());
        Ok(())
    }

    pub fn get_type_data(&self, type_name: &str) -> Option<&PersistData> {
        self.type_data.get(type_name)
    }

    pub fn set_type_data(&mut self, type_name: String, data: PersistData) {
        self.type_data.insert(type_name, data);
    }
}

/// Trait for types that can be persisted
pub trait Persistable: Resource + Serialize + for<'de> Deserialize<'de> {
    /// Get the type name for persistence
    fn type_name() -> &'static str;
    
    /// Convert to persistence data
    fn to_persist_data(&self) -> PersistData {
        let mut data = PersistData::new();
        if let Ok(json_value) = serde_json::to_value(self) {
            if let serde_json::Value::Object(map) = json_value {
                for (key, value) in map {
                    data.values.insert(key, value);
                }
            }
        }
        data
    }
    
    /// Load from persistence data
    fn from_persist_data(&mut self, data: &PersistData) {
        // This is a default implementation that tries to deserialize the whole object
        // Types can override this for custom behavior
        if let Ok(value) = serde_json::to_value(&data.values) {
            if let Ok(new_self) = serde_json::from_value(value) {
                *self = new_self;
            }
        }
    }
}

/// Marker component for resources that should auto-persist
#[derive(Component)]
pub struct AutoPersist;

/// Resource that manages persistence
#[derive(Resource)]
pub struct PersistManager {
    /// Path to the persistence file
    pub file_path: PathBuf,
    /// Cached persist file
    persist_file: PersistFile,
    /// Whether auto-save is enabled
    pub auto_save: bool,
}

impl PersistManager {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        let file_path = file_path.into();
        let persist_file = PersistFile::load_from_file(&file_path)
            .unwrap_or_else(|e| {
                error!("Failed to load persist file: {}", e);
                PersistFile::new()
            });
        
        Self {
            file_path,
            persist_file,
            auto_save: true,
        }
    }
    
    pub fn save(&mut self) -> PersistResult<()> {
        self.persist_file.save_to_file(&self.file_path)
    }
    
    pub fn load(&mut self) -> PersistResult<()> {
        self.persist_file = PersistFile::load_from_file(&self.file_path)?;
        Ok(())
    }
    
    pub fn get_persist_file(&self) -> &PersistFile {
        &self.persist_file
    }
    
    pub fn get_persist_file_mut(&mut self) -> &mut PersistFile {
        &mut self.persist_file
    }
}

/// Plugin for automatic persistence
pub struct PersistPlugin {
    /// Path to the persistence file
    pub file_path: String,
    /// Whether to enable auto-save on changes
    pub auto_save: bool,
}

impl Default for PersistPlugin {
    fn default() -> Self {
        Self {
            file_path: "settings.ron".to_string(),
            auto_save: true,
        }
    }
}

impl PersistPlugin {
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            auto_save: true,
        }
    }
    
    pub fn with_auto_save(mut self, enabled: bool) -> Self {
        self.auto_save = enabled;
        self
    }
}

impl Plugin for PersistPlugin {
    fn build(&self, app: &mut App) {
        let mut manager = PersistManager::new(self.file_path.clone());
        manager.auto_save = self.auto_save;
        
        app.insert_resource(manager);
        
        if self.auto_save {
            app.add_systems(Update, auto_save_system);
        }
    }
}

/// System that auto-saves changed resources
fn auto_save_system(
    manager: Res<PersistManager>,
    // We'll need to add specific systems for each persistable type
    // This is where the derive macro will come in handy
) {
    // This will be implemented per-type
}

/// Helper macro to register a persistable resource
#[macro_export]
macro_rules! register_persistable {
    ($app:expr, $type:ty) => {
        $app.add_systems(
            Update,
            $crate::persist_system::<$type>
        );
        
        // Load initial values on startup
        $app.add_systems(
            Startup,
            $crate::load_persisted::<$type>
        );
    };
}

/// Generic system to persist a resource when it changes
pub fn persist_system<T: Persistable>(
    mut manager: ResMut<PersistManager>,
    resource: Res<T>,
) {
    if resource.is_changed() && !resource.is_added() && manager.auto_save {
        let data = resource.to_persist_data();
        manager.get_persist_file_mut().set_type_data(T::type_name().to_string(), data);
        
        if let Err(e) = manager.save() {
            error!("Failed to auto-save {}: {}", T::type_name(), e);
        } else {
            debug!("Auto-saved {}", T::type_name());
        }
    }
}

/// Load persisted values on startup
pub fn load_persisted<T: Persistable>(
    manager: Res<PersistManager>,
    mut resource: ResMut<T>,
) {
    if let Some(data) = manager.get_persist_file().get_type_data(T::type_name()) {
        resource.from_persist_data(data);
        info!("Loaded persisted data for {}", T::type_name());
    }
}