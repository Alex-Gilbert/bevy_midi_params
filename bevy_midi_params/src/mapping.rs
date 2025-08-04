/// MIDI control mapping information
#[derive(Debug, Clone, PartialEq)]
pub struct MidiMapping {
    /// MIDI CC number (0-127), None means persist-only (no MIDI control)
    pub cc: Option<u8>,
    /// Field name this maps to
    pub field_name: String,
    /// Control type ("Range" or "Button")
    pub control_type: ControlType,
    /// Minimum value for range controls
    pub min_value: f32,
    /// Maximum value for range controls  
    pub max_value: f32,
}

/// Type of MIDI control
#[derive(Debug, Clone, PartialEq)]
pub enum ControlType {
    /// Continuous range control (knobs, faders)
    Range { min: f32, max: f32 },
    /// Toggle button control
    Button,
}

impl MidiMapping {
    /// Create a new range mapping
    pub fn range(cc: Option<u8>, field_name: impl Into<String>, min: f32, max: f32) -> Self {
        Self {
            cc,
            field_name: field_name.into(),
            control_type: ControlType::Range { min, max },
            min_value: min,
            max_value: max,
        }
    }
    
    /// Create a new button mapping
    pub fn button(cc: Option<u8>, field_name: impl Into<String>) -> Self {
        Self {
            cc,
            field_name: field_name.into(),
            control_type: ControlType::Button,
            min_value: 0.0,
            max_value: 1.0,
        }
    }
    
    /// Create a persist-only range mapping (no MIDI control)
    pub fn persist_range(field_name: impl Into<String>, min: f32, max: f32) -> Self {
        Self::range(None, field_name, min, max)
    }
    
    /// Create a persist-only button mapping (no MIDI control)
    pub fn persist_button(field_name: impl Into<String>) -> Self {
        Self::button(None, field_name)
    }
    
    /// Check if this mapping has MIDI control enabled
    pub fn has_midi_control(&self) -> bool {
        self.cc.is_some()
    }
    
    /// Scale a normalized MIDI value (0.0-1.0) to this mapping's range
    pub fn scale_value(&self, normalized: f32) -> f32 {
        match self.control_type {
            ControlType::Range { min, max } => min + normalized * (max - min),
            ControlType::Button => if normalized > 0.5 { 1.0 } else { 0.0 },
        }
    }
}
