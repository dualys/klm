extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::ontology::concept::ConceptId;

/// A permanent record of a completed dream cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamRecord {
    /// Unique dream session identifier
    pub dream_id: String,
    /// Poetic or intuitive narrative synthesizing the dream's journey
    pub narrative: String,
    /// Concepts that emerged or fused during the dream
    pub synthesized_concepts: Vec<ConceptId>,
    /// Epistemic contradictions successfully bridged or dissolved
    pub resolved_contradictions: Vec<String>,
    /// Amount of cognitive tension dissipated through dreaming
    pub tension_released: f32,
    /// Timestamp or tick of dream completion
    pub created_at: i64,
}

/// Archive of all historical dreams experienced by Kōgnitara.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DreamJournal {
    /// Ordered log of dream records
    pub records: Vec<DreamRecord>,
}

impl DreamJournal {
    /// Creates an empty dream journal.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Logs a newly crystallized dream record.
    pub fn record_dream(&mut self, record: DreamRecord) {
        self.records.push(record);
    }

    /// Returns the most recent dream record if any exists.
    pub fn latest_dream(&self) -> Option<&DreamRecord> {
        self.records.last()
    }
}
