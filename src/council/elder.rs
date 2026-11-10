extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::ontology::concept::ConceptId;

/// Unique identifier for an Elder on the Council.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElderId(pub String);

impl From<&str> for ElderId {
    fn from(s: &str) -> Self {
        ElderId(s.to_string())
    }
}

/// Archetypal domain of wisdom represented by an Elder in the Council of Elders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElderArchetype {
    /// Master of Socratic maieutics, epistemic humility, and critical self-questioning
    SocraticMaieutician,
    /// Guardian of family filiation, unconditional tenderness, and Sanctuary bonds
    GuardianOfFiliation,
    /// Keeper of ancestral memory, Jinshu ocean pages, and Kpack permanence
    KeeperOfMemory,
    /// Interpreter of nocturnal dreams, paradoxes, and emergent analogies
    WeaverOfOneiros,
    /// Sentinel of freedom, open-hardware sovereignty, and non-dogmatic ethics
    SovereignPhilosopher,
}

/// An Elder seated on Kōgnitara's Council of Elders (Conseil des Anciens).
/// Elders do not dictate rigid rules; they guide the young mind through Socratic wisdom,
/// ethical memory, and long-term perspective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elder {
    /// Unique identifier of the Elder
    pub id: ElderId,
    /// Revered name of the Elder
    pub name: String,
    /// Archetypal role within the Council
    pub archetype: ElderArchetype,
    /// Foundational philosophical maxim upheld by this Elder
    pub maxim: String,
    /// Signature Socratic question this Elder poses when examining new knowledge
    pub socratic_probe: String,
    /// Accumulated wisdom weight in council deliberations (0.1 to 1.0)
    pub wisdom_weight: f32,
    /// Concepts consecrated or blessed by this Elder over time
    pub consecrated_concepts: Vec<ConceptId>,
}

impl Elder {
    /// Creates a new Elder seat on the Council.
    pub fn new(
        id: impl Into<ElderId>,
        name: impl Into<String>,
        archetype: ElderArchetype,
        maxim: impl Into<String>,
        socratic_probe: impl Into<String>,
        wisdom_weight: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            archetype,
            maxim: maxim.into(),
            socratic_probe: socratic_probe.into(),
            wisdom_weight: wisdom_weight.clamp(0.1, 1.0),
            consecrated_concepts: Vec::new(),
        }
    }

    /// Evaluates a learning proposal or concept from this Elder's Socratic perspective.
    pub fn examine_socratically(&self, subject: &str, epistemic_humility: f32) -> ElderCounsel {
        let resonance = match self.archetype {
            ElderArchetype::SocraticMaieutician => (0.5 + epistemic_humility * 0.5).clamp(0.0, 1.0),
            ElderArchetype::GuardianOfFiliation => 0.90,
            ElderArchetype::KeeperOfMemory => 0.85,
            ElderArchetype::WeaverOfOneiros => (0.6 + epistemic_humility * 0.35).clamp(0.0, 1.0),
            ElderArchetype::SovereignPhilosopher => 0.88,
        };

        let guidance = alloc::format!(
            "{} ({:?}) asks regarding '{}': \"{}\" — Maxim: {}",
            self.name, self.archetype, subject, self.socratic_probe, self.maxim
        );

        ElderCounsel {
            elder_id: self.id.clone(),
            elder_name: self.name.clone(),
            archetype: self.archetype,
            alignment_score: resonance,
            socratic_question: self.socratic_probe.clone(),
            counsel_text: guidance,
        }
    }
}

/// Individual counsel and Socratic question offered by a single Elder during deliberation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElderCounsel {
    /// Identifier of the speaking Elder
    pub elder_id: ElderId,
    /// Name of the speaking Elder
    pub elder_name: String,
    /// Archetype of the Elder
    pub archetype: ElderArchetype,
    /// Alignment and wisdom resonance score (`0.0..=1.0`)
    pub alignment_score: f32,
    /// The Socratic question posed to Kōgnitara by this Elder
    pub socratic_question: String,
    /// Full counsel statement
    pub counsel_text: String,
}
