extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use noun::Noun;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::compute::{ComputeBackend, CpuBackend};

/// Dimensionality of Kōgnitara's continuous latent concept embeddings.
pub const EMBEDDING_DIM: usize = 16;

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

/// Projects arbitrary text or symbolic features into a unit-normalized 16D embedding vector
/// using BLAKE3 feature projection and open compute normalization.
pub fn compute_text_embedding(text: &str, valence_hint: f32) -> Vec<f32> {
    let mut vec = alloc::vec![0.0f32; EMBEDDING_DIM];
    let lower = text.to_lowercase();

    // Project word tokens so semantically overlapping phrases share directional components
    let mut has_words = false;
    for token in lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
    {
        has_words = true;
        let digest = blake3::hash(token.as_bytes());
        let bytes = digest.as_bytes();
        for i in 0..EMBEDDING_DIM {
            let b = bytes[i] as f32 / 127.5 - 1.0;
            vec[i] += b;
        }
    }

    if !has_words {
        let digest = blake3::hash(lower.as_bytes());
        let bytes = digest.as_bytes();
        for i in 0..EMBEDDING_DIM {
            vec[i] = bytes[i] as f32 / 127.5 - 1.0;
        }
    }

    // Encode affective tone subtly into dimension 0
    vec[0] += valence_hint * 0.35;

    let cpu = CpuBackend::new();
    cpu.normalize(&mut vec);
    vec
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
    /// Continuous latent semantic embedding vector (16D open format)
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
    /// Creates a nascent concept with high plasticity, normalized semantic embedding, and natural epistemic humility.
    pub fn new(
        id: impl Into<ConceptId>,
        label: impl Into<String>,
        description: impl Into<String>,
        origin: ConceptOrigin,
    ) -> Self {
        let id = id.into();
        let label = label.into();
        let description = description.into();
        let now = current_timestamp();

        let seed_text = alloc::format!("{} {}", label, description);
        let embedding = compute_text_embedding(&seed_text, 0.0);

        Self {
            id,
            label,
            description,
            tags: Vec::new(),
            embedding,
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

    /// Computes the 32-byte Amentys `Noun` cryptographic identity for this concept.
    pub fn noun(&self) -> Noun {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.id.0.as_bytes());
        hasher.update(self.label.as_bytes());
        hasher.update(self.description.as_bytes());
        let bytes: [u8; 32] = *hasher.finalize().as_bytes();
        Noun::from_bytes(&bytes).unwrap_or_else(Noun::null)
    }

    /// Returns the epistemic doubt (uncertainty modulated by plasticity) in `[0.0, 1.0]`.
    /// High epistemic doubt drives Kōgnitara's curiosity to ask questions and dream.
    pub fn epistemic_doubt(&self) -> f32 {
        ((1.0 - self.confidence) * (0.5 + 0.5 * self.plasticity)).clamp(0.0, 1.0)
    }

    /// Adds a semantic tag and gently adapts the concept's embedding vector.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag_str = tag.into();
        if !self.tags.contains(&tag_str) {
            let tag_emb = compute_text_embedding(&tag_str, self.affective_valence);
            let cpu = CpuBackend::new();
            self.embedding = cpu.blend_embeddings(&self.embedding, &tag_emb, 0.20);
            self.tags.push(tag_str);
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
        self.last_resonated_at = current_timestamp();
    }

    /// Natural temporal decay of transient activation (short-term forgetting).
    pub fn decay_activation(&mut self, rate: f32) {
        self.current_activation = (self.current_activation * (1.0 - rate)).max(0.0);
    }
}
