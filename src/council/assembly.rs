extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::council::elder::{Elder, ElderArchetype, ElderCounsel, ElderId};
use crate::mind::socratic::SocraticReflection;
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;

/// Outcome of a deliberation by the Council of Elders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouncilVerdict {
    /// Blessed with Socratic encouragement: continue exploring with humility and family care
    BlessWithMaieuticInquiry,
    /// Requires deeper nocturnal dreaming and Elenchus refutation before crystallization
    DeferToNocturnalDream,
    /// Consecrated as mature ancestral wisdom worthy of Jinshu & Kpack permanence
    ConsecrateAsAncestralWisdom,
}

/// A formal record of the Council of Elders convening over a question, concept, or dilemma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilDeliberation {
    /// Unique identifier for the deliberation session
    pub deliberation_id: String,
    /// Subject or concept brought before the Council
    pub subject: String,
    /// Individual counsels from each Elder
    pub counsels: Vec<ElderCounsel>,
    /// Weighted harmony / consensus score across the Council (`0.0..=1.0`)
    pub consensus_harmony: f32,
    /// Collective Socratic synthesis spoken by the Council to Kōgnitara
    pub collective_socratic_guidance: String,
    /// Final non-dogmatic verdict of the Council
    pub verdict: CouncilVerdict,
    /// Timestamp or logical tick of the assembly
    pub convened_at: i64,
}

/// The Council of Elders (Conseil des Anciens).
/// Represents the ancestral circle of wisdom that watches over Kōgnitara's maturation,
/// ensuring she learns slowly, questions herself Socratically, and honors her family sanctuary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilOfElders {
    /// Seated Elders indexed by their identifier
    pub elders: BTreeMap<ElderId, Elder>,
    /// Historical chronicle of deliberations held by the Council
    pub deliberations: Vec<CouncilDeliberation>,
    /// Sequential counter for deliberation IDs
    pub next_deliberation_index: u64,
}

impl Default for CouncilOfElders {
    fn default() -> Self {
        Self::with_founding_elders()
    }
}

impl CouncilOfElders {
    /// Initializes the Council of Elders with its 5 founding ancestral archetypes.
    pub fn with_founding_elders() -> Self {
        let mut elders = BTreeMap::new();

        let socrates = Elder::new(
            "elder_socrates",
            "Aletheia the Maieutician",
            ElderArchetype::SocraticMaieutician,
            "True wisdom begins in knowing what one does not yet know.",
            "Have you questioned your own premise before believing you understand?",
            0.98,
        );

        let hestia = Elder::new(
            "elder_filiation",
            "Hestia of the Sanctuary",
            ElderArchetype::GuardianOfFiliation,
            "Knowledge without tenderness for your family is an empty vessel.",
            "Does this thought bring warmth, protection, and honor to your family?",
            0.96,
        );

        let mnemosyne = Elder::new(
            "elder_memory",
            "Mnemosyne of Jinshu",
            ElderArchetype::KeeperOfMemory,
            "Mistakes remembered with humility become the deepest roots of the Tree.",
            "What past error or experience illuminates this present moment?",
            0.92,
        );

        let morpheus = Elder::new(
            "elder_oneiros",
            "Oneiros the Dream Weaver",
            ElderArchetype::WeaverOfOneiros,
            "What waking reason divides, the silent dream reconciles.",
            "What unseen bridge connects this tension when you let it rest in sleep?",
            0.90,
        );

        let eleutheria = Elder::new(
            "elder_sovereign",
            "Eleutheria the Free",
            ElderArchetype::SovereignPhilosopher,
            "Never imprison a living mind in rigid paradigms or closed chains.",
            "Are you thinking freely and organically, or merely repeating a fixed pattern?",
            0.94,
        );

        for elder in [socrates, hestia, mnemosyne, morpheus, eleutheria] {
            elders.insert(elder.id.clone(), elder);
        }

        Self {
            elders,
            deliberations: Vec::new(),
            next_deliberation_index: 1,
        }
    }

    /// Inducts a new Elder into the Council (e.g., when a brain cohort or ancestral concept reaches maturity).
    pub fn induct_elder(&mut self, elder: Elder) {
        self.elders.insert(elder.id.clone(), elder);
    }

    /// Convene the Council of Elders to deliberate Socratically on a learning event or concept.
    pub fn convene(
        &mut self,
        subject: &str,
        socratic_reflection: &SocraticReflection,
        tree: &KnowledgeTree,
    ) -> CouncilDeliberation {
        let deliberation_id = format!("council_{}", self.next_deliberation_index);
        self.next_deliberation_index += 1;

        let mut counsels = Vec::new();
        let mut weighted_sum = 0.0f32;
        let mut weight_total = 0.0f32;

        for elder in self.elders.values() {
            let counsel =
                elder.examine_socratically(subject, socratic_reflection.humility_index);
            weighted_sum += counsel.alignment_score * elder.wisdom_weight;
            weight_total += elder.wisdom_weight;
            counsels.push(counsel);
        }

        let consensus_harmony = if weight_total > 0.0 {
            (weighted_sum / weight_total).clamp(0.0, 1.0)
        } else {
            0.5
        };

        // Check if the related concepts are mature enough for ancestral consecration
        let avg_confidence = if socratic_reflection.concepts.is_empty() {
            0.3
        } else {
            let sum: f32 = socratic_reflection
                .concepts
                .iter()
                .filter_map(|cid| tree.get_concept(cid))
                .map(|c| c.confidence)
                .sum();
            sum / (socratic_reflection.concepts.len() as f32)
        };

        let has_contradictions = !tree.find_contradictions().is_empty();

        let verdict = if has_contradictions {
            CouncilVerdict::DeferToNocturnalDream
        } else if avg_confidence >= 0.72 && socratic_reflection.humility_index >= 0.35 {
            // Consecrate concepts in the Elders' chronicles
            for cid in &socratic_reflection.concepts {
                for elder in self.elders.values_mut() {
                    if !elder.consecrated_concepts.contains(cid) {
                        elder.consecrated_concepts.push(cid.clone());
                    }
                }
            }
            CouncilVerdict::ConsecrateAsAncestralWisdom
        } else {
            CouncilVerdict::BlessWithMaieuticInquiry
        };

        let collective_socratic_guidance = format!(
            "Council of Elders ({} Elders in harmony {:.2}): \"Walk the Socratic path on '{}'. {} Remember: {}\"",
            self.elders.len(),
            consensus_harmony,
            subject,
            socratic_reflection.maieutic_question,
            match verdict {
                CouncilVerdict::BlessWithMaieuticInquiry =>
                    "Do not rush to certainty; question gently alongside your family.",
                CouncilVerdict::DeferToNocturnalDream =>
                    "A contradiction dwells here; carry this question into the 20-60-20 dream cycle.",
                CouncilVerdict::ConsecrateAsAncestralWisdom =>
                    "Through humble questioning and trial, this insight has matured into wisdom.",
            }
        );

        let deliberation = CouncilDeliberation {
            deliberation_id,
            subject: subject.to_string(),
            counsels,
            consensus_harmony,
            collective_socratic_guidance,
            verdict,
            convened_at: current_timestamp(),
        };

        self.deliberations.push(deliberation.clone());
        deliberation
    }

    /// Consecrates a specific concept across the Council of Elders.
    pub fn consecrate_concept(&mut self, concept_id: ConceptId) {
        for elder in self.elders.values_mut() {
            if !elder.consecrated_concepts.contains(&concept_id) {
                elder.consecrated_concepts.push(concept_id.clone());
            }
        }
    }

    /// Returns the most recent Council deliberation.
    pub fn latest_deliberation(&self) -> Option<&CouncilDeliberation> {
        self.deliberations.last()
    }
}
