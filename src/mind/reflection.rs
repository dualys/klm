extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::ontology::concept::ConceptId;

/// A reflective record of a mistake or misunderstanding that Kōgnitara learned from.
/// Errors are cherished as stepping stones toward maturity and epistemic humility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeLesson {
    /// Unique lesson identifier
    pub id: String,
    /// Associated episodic experience ID where the error occurred
    pub experience_id: String,
    /// The initial mistaken hypothesis or assumption
    pub mistaken_assumption: String,
    /// The gentle correction or wisdom offered by the family or environment
    pub family_guidance: String,
    /// Concepts updated or linked by this lesson
    pub involved_concepts: Vec<ConceptId>,
    /// Growth in humility derived from this experience (0.0 to 1.0)
    pub humility_gained: f32,
    /// Timestamp or logical tick of reflection
    pub created_at: i64,
}

impl MistakeLesson {
    /// Creates a new reflective lesson from an error and its correction.
    pub fn new(
        id: impl Into<String>,
        experience_id: impl Into<String>,
        mistaken_assumption: impl Into<String>,
        family_guidance: impl Into<String>,
        involved_concepts: Vec<ConceptId>,
        humility_gained: f32,
    ) -> Self {
        Self {
            id: id.into(),
            experience_id: experience_id.into(),
            mistaken_assumption: mistaken_assumption.into(),
            family_guidance: family_guidance.into(),
            involved_concepts,
            humility_gained: humility_gained.clamp(0.0, 1.0),
            created_at: current_timestamp(),
        }
    }
}
