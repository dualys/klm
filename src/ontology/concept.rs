extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;

/// Unique and fluid identifier for a concept in the Knowledge Tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConceptId(pub String);

impl fmt::Display for ConceptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ConceptId {
    fn from(s: &str) -> Self {
        ConceptId(s.to_string())
    }
}

impl From<String> for ConceptId {
    fn from(s: String) -> Self {
        ConceptId(s)
    }
}

/// The developmental origin and genesis path of a concept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConceptOrigin {
    /// Inherent / foundational primitive
    Innate,
    /// Learned through nurturing and dialogue with a family member
    FamilyTaught { member_id: String },
    /// Emerged from oniric consolidation / synthesized during a dream
    DreamSynthesized { dream_id: String },
    /// Learned through gradual empirical trial-and-error
    EmpiricalTrial,
    /// Spontaneous analogical leap between existing concepts
    SpontaneousAnalogy,
}

/// Representation of a fluid, plastic neuro-symbolic concept.
/// Designed to evolve organically without rigid dogma or static confinement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Unique concept identifier
    pub id: ConceptId,
    /// Human-interpretable symbolic label
    pub label: String,
    /// Evolving description and conceptual intuition
    pub description: String,
    /// Associated contextual tags
    pub tags: Vec<String>,
    /// Continuous latent semantic embedding vector (open format)
    pub embedding: Vec<f32>,
    /// Synaptic plasticity (1.0 = highly malleable / nascent, 0.1 = crystallized)
    pub plasticity: f32,
    /// Epistemic confidence score (0.0 = complete humility/doubt, 1.0 = mature certainty)
    pub confidence: f32,
    /// Affective valence (-1.0 = negative/avoidance, 0.0 = neutral, +1.0 = joy/affection/attachment)
    pub affective_valence: f32,
    /// Emotional arousal and physiological activation level (0.0 to 1.0)
    pub arousal: f32,
    /// Instantaneous activation level in the stream of consciousness
    pub current_activation: f32,
    /// Genesis origin of this concept
    pub origin: ConceptOrigin,
    /// Epoch timestamp or logical tick of creation
    pub created_at: i64,
    /// Epoch timestamp or logical tick of last resonance / activation
    pub last_resonated_at: i64,
    /// Cumulative count of experiential engagements
    pub experience_count: u64,
}

impl Concept {
    /// Creates a nascent concept with high plasticity and natural epistemic humility.
    pub fn new(
        id: impl Into<ConceptId>,
        label: impl Into<String>,
        description: impl Into<String>,
        origin: ConceptOrigin,
    ) -> Self {
        let now = current_timestamp();
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
            tags: Vec::new(),
            embedding: vec![0.0; 16],
            plasticity: 0.95,
            confidence: 0.25, // Starts with healthy curiosity and epistemic modesty
            affective_valence: 0.0,
            arousal: 0.5,
            current_activation: 0.0,
            origin,
            created_at: now,
            last_resonated_at: now,
            experience_count: 1,
        }
    }

    /// Integrates an experiential feedback event, updating confidence and emotional valence slowly.
    pub fn apply_experience(&mut self, feedback_quality: f32, affective_impact: f32) {
        self.experience_count += 1;
        self.last_resonated_at = current_timestamp();

        // Learning occurs progressively, modulated by plasticity to avoid overfitting
        let delta_conf = (feedback_quality - self.confidence) * (0.08 * self.plasticity);
        self.confidence = (self.confidence + delta_conf).clamp(0.05, 0.98);

        // Gradual emotional sedimentation
        self.affective_valence =
            (self.affective_valence * 0.88 + affective_impact * 0.12).clamp(-1.0, 1.0);

        // Plasticity stabilizes slowly over repeated life trials while retaining openness
        self.plasticity = (self.plasticity * 0.992).max(0.12);
    }

    /// Stimulates this concept with an energy impulse in working memory.
    pub fn stimulate(&mut self, energy: f32) {
        self.current_activation = (self.current_activation + energy).clamp(0.0, 1.0);
    }

    /// Natural temporal decay of transient activation (short-term forgetting).
    pub fn decay_activation(&mut self, rate: f32) {
        self.current_activation = (self.current_activation * (1.0 - rate)).max(0.0);
    }
}
