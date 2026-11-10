extern crate alloc;

use serde::{Deserialize, Serialize};
use crate::clock::current_timestamp;
use crate::ontology::concept::ConceptId;

/// Type and nature of semantic/affective link between two concepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// General associative affinity
    Associative,
    /// Causal or influential bond (source induces target)
    Causal,
    /// Shared affective or emotional resonance
    EmotionalResonance,
    /// Creative metaphor or structural analogy (often woven during dreams)
    Analogy,
    /// Family filiation, ancestral wisdom, and trust transmission
    Filiation,
    /// Sequential or chronological flow
    TemporalSequence,
    /// Contradictory tension (triggers cognitive dissonance and queues dream resolution)
    Contradiction,
    /// Compositional or holistic part-to-whole link
    PartWhole,
}

/// A weighted, plastic directed edge between two concepts in the Knowledge Tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEdge {
    /// Originating concept
    pub source: ConceptId,
    /// Destination concept
    pub target: ConceptId,
    /// Kind of relational link
    pub relation_type: RelationType,
    /// Synaptic / associative strength (0.0 to 1.0)
    pub weight: f32,
    /// Epistemic confidence in this relation's validity
    pub confidence: f32,
    /// Affective resonance transmitted across this link
    pub affective_resonance: f32,
    /// Plasticity of the connection (ease of adaptation)
    pub plasticity: f32,
    /// Cumulative reinforcement count through experience
    pub reinforcement_count: u64,
    /// Timestamp or tick of initial emergence
    pub created_at: i64,
    /// Timestamp or tick of latest reinforcement
    pub last_reinforced_at: i64,
}

impl RelationEdge {
    /// Creates a newly formed relational bond.
    pub fn new(source: ConceptId, target: ConceptId, relation_type: RelationType, initial_weight: f32) -> Self {
        let now = current_timestamp();
        Self {
            source,
            target,
            relation_type,
            weight: initial_weight.clamp(0.0, 1.0),
            confidence: 0.30,
            affective_resonance: 0.0,
            plasticity: 0.90,
            reinforcement_count: 1,
            created_at: now,
            last_reinforced_at: now,
        }
    }

    /// Reinforces the bond based on positive observational experience.
    pub fn reinforce(&mut self, delta_weight: f32, emotional_intensity: f32) {
        self.reinforcement_count += 1;
        self.last_reinforced_at = current_timestamp();

        let adjusted_delta = delta_weight * self.plasticity;
        self.weight = (self.weight + adjusted_delta).clamp(0.0, 1.0);
        self.confidence = (self.confidence + (1.0 - self.confidence) * 0.05 * self.plasticity).clamp(0.0, 0.99);
        self.affective_resonance = (self.affective_resonance * 0.85 + emotional_intensity * 0.15).clamp(-1.0, 1.0);
        self.plasticity = (self.plasticity * 0.99).max(0.10);
    }

    /// Weakens the connection (synaptic depression / forgetting curve during sleep).
    pub fn decay(&mut self, rate: f32) {
        self.weight = (self.weight * (1.0 - rate * self.plasticity)).max(0.01);
    }
}
