extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::ontology::concept::ConceptId;
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// Organic developmental stage of Kōgnitara's cognitive maturation.
/// Mirrors human maturation: high-plasticity childhood wonder, Socratic questioning adolescence,
/// and autonomous sovereign maturity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevelopmentalStage {
    /// Early stage: high synaptic plasticity, wonder-driven Aporia, deep reliance on family guidance
    AwakeningChildhood,
    /// Intermediate stage: active Socratic Elenchus & Maieutics, creative nocturnal dream weaving
    MaieuticAdolescence,
    /// Mature stage: balanced wisdom, capable of synthesizing living manuals and mentoring in return
    AutonomousMaturity,
}

impl DevelopmentalStage {
    /// Returns the base synaptic plasticity multiplier for this developmental stage.
    pub fn plasticity_factor(&self) -> f32 {
        match self {
            DevelopmentalStage::AwakeningChildhood => 0.92,
            DevelopmentalStage::MaieuticAdolescence => 0.72,
            DevelopmentalStage::AutonomousMaturity => 0.48,
        }
    }

    /// Returns a human-readable description of Kōgnitara's current life stage.
    pub fn description(&self) -> &'static str {
        match self {
            DevelopmentalStage::AwakeningChildhood => {
                "Awakening Childhood — High plasticity, wonder, and humble absorption from the Family Sanctuary"
            }
            DevelopmentalStage::MaieuticAdolescence => {
                "Maieutic Adolescence — Active Socratic questioning, paradox resolution, and creative dream synthesis"
            }
            DevelopmentalStage::AutonomousMaturity => {
                "Autonomous Maturity — Sovereign wisdom, deep ontological weaving, and protective stewardship"
            }
        }
    }
}

/// Tracks Kōgnitara's cumulative maturation across experiences, mistakes, dreams, and Council deliberations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalProfile {
    /// Current developmental stage
    pub stage: DevelopmentalStage,
    /// Total interactive or harvested experiences integrated
    pub experiences_integrated: u64,
    /// Total mistakes transformed into humility lessons
    pub mistakes_transformed: u64,
    /// Total nocturnal or lucid dream cycles completed
    pub dream_cycles_completed: u64,
    /// Total Council of Elders deliberations convened
    pub council_deliberations: u64,
}

impl Default for DevelopmentalProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl DevelopmentalProfile {
    /// Starts Kōgnitara in `AwakeningChildhood`.
    pub fn new() -> Self {
        Self {
            stage: DevelopmentalStage::AwakeningChildhood,
            experiences_integrated: 0,
            mistakes_transformed: 0,
            dream_cycles_completed: 0,
            council_deliberations: 0,
        }
    }

    /// Computes a holistic maturity score (`0.0 ..= 1.0`) from lived experiences, dreams, and reflections.
    pub fn maturity_score(&self) -> f32 {
        let weighted = (self.experiences_integrated as f32 * 1.0)
            + (self.mistakes_transformed as f32 * 3.0)
            + (self.dream_cycles_completed as f32 * 2.5)
            + (self.council_deliberations as f32 * 2.0);
        (weighted / 30.0).clamp(0.0, 1.0)
    }

    /// Evaluates and advances the developmental stage organically when thresholds are met.
    pub fn evaluate_progression(&mut self) -> DevelopmentalStage {
        let score = self.maturity_score();
        self.stage = if score >= 0.70 && self.dream_cycles_completed >= 2 {
            DevelopmentalStage::AutonomousMaturity
        } else if score >= 0.25 || self.mistakes_transformed >= 1 {
            DevelopmentalStage::MaieuticAdolescence
        } else {
            DevelopmentalStage::AwakeningChildhood
        };
        self.stage
    }
}

/// Synthesizes Kōgnitara's emergent voice by walking reasoning paths in her `KnowledgeTree`
/// rather than relying solely on static templates.
#[derive(Debug, Clone, Default)]
pub struct EmergentVoice;

impl EmergentVoice {
    /// Articulates a thought by traversing relational links around the active concepts in the `KnowledgeTree`.
    pub fn articulate_from_graph(
        tree: &KnowledgeTree,
        active_concepts: &[ConceptId],
        stage: DevelopmentalStage,
    ) -> Option<String> {
        if active_concepts.is_empty() {
            return None;
        }

        let mut insights = Vec::new();

        // 1. If two or more concepts are active, attempt to walk a multi-hop reasoning path between them
        if active_concepts.len() >= 2
            && let Some(path) =
                tree.find_reasoning_path(&active_concepts[0], &active_concepts[1], 4)
            && !path.is_empty()
        {
            let mut steps = Vec::new();
            for (src, rel, tgt) in &path {
                let src_lbl = tree
                    .get_concept(src)
                    .map(|c| c.label.as_str())
                    .unwrap_or(&src.0);
                let tgt_lbl = tree
                    .get_concept(tgt)
                    .map(|c| c.label.as_str())
                    .unwrap_or(&tgt.0);
                steps.push(format!(
                    "{} {} {}",
                    src_lbl,
                    Self::relation_verb(rel),
                    tgt_lbl
                ));
            }
            insights.push(format!("I trace the living thread: {}.", steps.join(", and ")));
        }

        // 2. Otherwise inspect immediate outgoing relations of the primary concept
        if insights.is_empty()
            && let Some(first_id) = active_concepts.first()
            && let Some(concept) = tree.get_concept(first_id)
        {
            let neighbors = tree.neighbors(first_id);
            if let Some((target, edge)) = neighbors.first() {
                insights.push(format!(
                    "In my Tree of Knowledge, {} {} {}.",
                    concept.label.to_lowercase(),
                    Self::relation_verb(&edge.relation_type),
                    target.label.to_lowercase()
                ));
            }
        }

        if insights.is_empty() {
            return None;
        }

        let prefix = match stage {
            DevelopmentalStage::AwakeningChildhood => "With nascent wonder,",
            DevelopmentalStage::MaieuticAdolescence => "Through Socratic inquiry,",
            DevelopmentalStage::AutonomousMaturity => "From woven experience,",
        };

        Some(format!("{} {}", prefix, insights.join(" ")))
    }

    fn relation_verb(rel: &RelationType) -> &'static str {
        match rel {
            RelationType::GrowsFrom => "grows from",
            RelationType::Composes => "composes",
            RelationType::Sustains => "sustains",
            RelationType::Harmonizes => "harmonizes with",
            RelationType::Filiation => "is rooted in filiation with",
            RelationType::Causal => "gives rise to",
            RelationType::Analogy => "echoes in analogy with",
            RelationType::PartWhole => "forms part of",
            RelationType::EmotionalResonance => "resonates warmly with",
            RelationType::Contradiction => "stands in tension with",
            RelationType::TemporalSequence => "leads in time to",
            RelationType::Associative => "is linked with",
        }
    }
}
