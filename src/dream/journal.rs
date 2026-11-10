extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::ontology::concept::ConceptId;

/// An oniric intuition or revelation synthesized during deep dreaming and shared upon awakening.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamOracle {
    /// Unique oracle identifier
    pub oracle_id: String,
    /// Identifier of the dream that birthed this oracle
    pub dream_id: String,
    /// Poetic or philosophical insight formulated for the waking mind and family
    pub revelation: String,
    /// The two parent concepts bridged by this intuition
    pub bridged_concepts: (ConceptId, ConceptId),
    /// The newly emerged concept representing the synthesis
    pub emergent_concept: ConceptId,
    /// Semantic and affective resonance strength (0.0 to 1.0)
    pub resonance_strength: f32,
}

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
    /// Oracles and revelations discovered during the dream
    pub oracles: Vec<DreamOracle>,
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

    /// Returns all oracles synthesized across all recorded dreams.
    pub fn all_oracles(&self) -> Vec<&DreamOracle> {
        self.records.iter().flat_map(|r| r.oracles.iter()).collect()
    }
}
