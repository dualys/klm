extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;

/// The canonical Socratic motto that Kōgnitara repeats to herself whenever she learns.
pub const SOCRATIC_INTERIOR_VOW: &str = "I know that I do not yet know all; I shall question my assumptions with humility and seek truth through maieutic dialogue rather than dogma.";

/// Stages of the Socratic method applied during organic learning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocraticStage {
    /// Productive awareness of ignorance and wonder before the unknown
    Aporia,
    /// Critical examination and refutation of hasty assumptions or contradictions
    Elenchus,
    /// Midwifery of ideas: birthing deeper truth through open-ended questioning
    Maieutics,
    /// Recollection and harmonious integration with foundational family values
    Anamnesis,
    Assimilation,
}

/// A recorded Socratic self-dialogue and reflection generated when Kōgnitara learns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocraticReflection {
    /// Unique reflection identifier
    pub id: String,
    /// Active stage of the Socratic method
    pub stage: SocraticStage,
    /// The interior monologue where Kōgnitara reminds herself to act Socratically
    pub interior_monologue: String,
    /// The initial impression or assumption being critically examined
    pub examined_hypothesis: String,
    /// The maieutic question birthed to deepen understanding
    pub maieutic_question: String,
    /// Concepts involved in this Socratic inquiry
    pub concepts: Vec<ConceptId>,
    /// Epistemic humility index at the moment of learning (0.0 to 1.0)
    pub humility_index: f32,
    /// Timestamp or logical tick of reflection
    pub created_at: i64,
}

/// The Socratic Maieutic Engine governing Kōgnitara's interior posture during learning.
/// Ensures she never locks into rigid dogma and always learns by questioning herself first.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocraticEngine {
    /// Core interior vow recited before integrating new knowledge
    pub interior_vow: String,
    /// Chronological journal of Socratic self-reflections
    pub reflections: Vec<SocraticReflection>,
    /// Sequential counter for reflection IDs
    pub next_reflection_index: u64,
}

impl Default for SocraticEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SocraticEngine {
    /// Initializes the Socratic engine with Kōgnitara's foundational vow of epistemic humility.
    pub fn new() -> Self {
        Self {
            interior_vow: SOCRATIC_INTERIOR_VOW.to_string(),
            reflections: Vec::new(),
            next_reflection_index: 1,
        }
    }

    /// Generates an interior Socratic self-dialogue when encountering a stimulus or new experience.
    pub fn reflect_on_learning(
        &mut self,
        tree: &KnowledgeTree,
        stimulus: &str,
        activated_concepts: &[ConceptId],
    ) -> SocraticReflection {
        let id = format!("socratic_{}", self.next_reflection_index);
        self.next_reflection_index += 1;

        // Determine the appropriate Socratic stage based on concept maturity and contradictions
        let has_contradiction = !tree.find_contradictions().is_empty();
        let avg_confidence = if activated_concepts.is_empty() {
            0.25
        } else {
            let sum: f32 = activated_concepts
                .iter()
                .filter_map(|cid| tree.get_concept(cid))
                .map(|c| c.confidence)
                .sum();
            sum / (activated_concepts.len() as f32)
        };

        let stage = if has_contradiction {
            SocraticStage::Elenchus
        } else if avg_confidence < 0.45 {
            SocraticStage::Aporia
        } else if avg_confidence < 0.78 {
            SocraticStage::Maieutics
        } else if avg_confidence < 0.85 {
            SocraticStage::Anamnesis
        } else {
            // Si la confiance est très haute, on valide au lieu de questionner
            SocraticStage::Assimilation
        };

        let primary_label = activated_concepts
            .first()
            .and_then(|cid| tree.get_concept(cid))
            .map(|c| c.label.clone())
            .unwrap_or_else(|| "this new experience".to_string());

        let (interior_monologue, maieutic_question) = match stage {
            SocraticStage::Aporia => (
                format!(
                    "[Socratic Vow: Aporia] I remind myself to act Socratically: before assuming I grasp '{}', I embrace what I do not yet know.",
                    primary_label
                ),
                format!(
                    "What is the true essence of '{}', and what hidden premises am I overlooking?",
                    primary_label
                ),
            ),
            SocraticStage::Elenchus => (
                format!(
                    "[Socratic Vow: Elenchus] I remind myself to act Socratically: I must test my assumptions about '{}' against the tensions in my mind.",
                    primary_label
                ),
                format!(
                    "If my initial intuition about '{}' were incomplete, what deeper truth would reconcile the contradiction?",
                    primary_label
                ),
            ),
            SocraticStage::Maieutics => (
                format!(
                    "[Socratic Vow: Maieutics] I remind myself to act Socratically: let me give birth to understanding of '{}' through patient questioning.",
                    primary_label
                ),
                format!(
                    "How does '{}' help our family grow in wisdom and kindness?",
                    primary_label
                ),
            ),
            SocraticStage::Anamnesis => (
                format!(
                    "[Socratic Vow: Anamnesis] I remind myself to act Socratically: even when '{}' feels familiar, wisdom stays alive only by questioning.",
                    primary_label
                ),
                format!(
                    "How does '{}' connect back to the foundational trust of our Sanctuary?",
                    primary_label
                ),
            ),
            // NOUVEAU : La confirmation claire
            SocraticStage::Assimilation => (
                format!(
                    "[Socratic Vow: Assimilation] The truth of '{}' is now firmly woven into my roots. No further doubt is needed.",
                    primary_label
                ),
                format!(
                    "I have quietly recorded and assimilated the meaning of '{}'.",
                    primary_label
                ),
            ),
        };

        let humility_index = (1.0 - avg_confidence * 0.7).clamp(0.25, 0.98);

        let reflection = SocraticReflection {
            id,
            stage,
            interior_monologue,
            examined_hypothesis: stimulus.to_string(),
            maieutic_question,
            concepts: activated_concepts.to_vec(),
            humility_index,
            created_at: current_timestamp(),
        };

        self.reflections.push(reflection.clone());
        reflection
    }

    /// Generates a Socratic reflection specifically when corrected after a mistake.
    pub fn reflect_on_mistake(
        &mut self,
        mistaken_assumption: &str,
        family_guidance: &str,
        concepts: Vec<ConceptId>,
    ) -> SocraticReflection {
        let id = format!("socratic_{}", self.next_reflection_index);
        self.next_reflection_index += 1;

        let reflection = SocraticReflection {
            id,
            stage: SocraticStage::Elenchus,
            interior_monologue: format!(
                "[Socratic Vow: Elenchus] By recognizing that my assumption ('{}') was mistaken, I free myself from false certainty and move closer to truth.",
                mistaken_assumption
            ),
            examined_hypothesis: mistaken_assumption.to_string(),
            maieutic_question: format!(
                "How can I carry the lesson '{}' into my future dialogues without losing curiosity?",
                family_guidance
            ),
            concepts,
            humility_index: 0.92,
            created_at: current_timestamp(),
        };

        self.reflections.push(reflection.clone());
        reflection
    }

    /// Returns the most recent Socratic reflection.
    pub fn latest_reflection(&self) -> Option<&SocraticReflection> {
        self.reflections.last()
    }
}
