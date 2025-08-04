use std::fmt;

/// Errors that can occur in bevy_midi_params
#[derive(Debug)]
pub enum MidiError {
    /// No MIDI input ports found
    NoInputPorts,
    /// Failed to connect to MIDI device
    ConnectionFailed(String),
    /// Failed to save/load persistence file
    PersistenceError(String),
    /// Invalid MIDI mapping configuration
    InvalidMapping(String),
}

impl fmt::Display for MidiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MidiError::NoInputPorts => write!(f, "No MIDI input ports found"),
            MidiError::ConnectionFailed(msg) => write!(f, "MIDI connection failed: {}", msg),
            MidiError::PersistenceError(msg) => write!(f, "Persistence error: {}", msg),
            MidiError::InvalidMapping(msg) => write!(f, "Invalid MIDI mapping: {}", msg),
        }
    }
}

impl std::error::Error for MidiError {}

/// Result type for MIDI operations
pub type MidiResult<T> = Result<T, MidiError>;
