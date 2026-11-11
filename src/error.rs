extern crate alloc;

use alloc::string::String;
use core::fmt;

/// Framework error categories for Kōgnitara (klm).
#[derive(Debug, Clone)]
pub enum KlmError {
    /// Errors occurring within the Knowledge Tree or conceptual structures.
    OntologyError(String),
    /// Errors related to the organic learning engine or cognitive tensions.
    CognitiveError(String),
    /// Errors occurring during sleep cycles, dreaming, or oniric synthesis.
    DreamError(String),
    /// Errors within the family sanctuary or relational trust bonds.
    SanctuaryError(String),
    /// Errors within the multi-brain swarm or Hamon Cube topology.
    SwarmError(String),
    /// Errors in open hardware compute execution (AMD / CPU / Vulkan).
    ComputeError(String),
    /// Errors in Kpack binary persistence, recipes, or BLAKE3 Merkle integrity.
    StorageError(String),
    /// Generic I/O or platform error.
    Io(String),
}

impl fmt::Display for KlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KlmError::OntologyError(msg) => write!(f, "Ontology Error: {msg}"),
            KlmError::CognitiveError(msg) => write!(f, "Cognitive Error: {msg}"),
            KlmError::DreamError(msg) => write!(f, "Dream Engine Error: {msg}"),
            KlmError::SanctuaryError(msg) => write!(f, "Sanctuary Error: {msg}"),
            KlmError::SwarmError(msg) => write!(f, "Swarm/Cube Error: {msg}"),
            KlmError::ComputeError(msg) => write!(f, "Compute Error: {msg}"),
            KlmError::StorageError(msg) => write!(f, "Storage/Kpack Error: {msg}"),
            KlmError::Io(msg) => write!(f, "Platform I/O Error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for KlmError {}

#[cfg(feature = "std")]
impl From<std::io::Error> for KlmError {
    fn from(err: std::io::Error) -> Self {
        KlmError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for KlmError {
    fn from(err: serde_json::Error) -> Self {
        KlmError::StorageError(format!("JSON Serialization Error: {err}"))
    }
}

pub type Result<T> = core::result::Result<T, KlmError>;
