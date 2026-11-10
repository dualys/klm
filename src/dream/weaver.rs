extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::dream::journal::DreamRecord;
use crate::mind::experience::EpisodicExperience;
use crate::mind::state::CognitiveState;
use crate::ontology::concept::ConceptId;
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// The oniric weaver responsible for nocturnal conceptual synthesis and unconstrained exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamWeaver {
    /// Incremental dream session counter
    pub dream_counter: u64,
}

impl Default for DreamWeaver {
    fn default() -> Self {
        Self::new()
    }
}

impl DreamWeaver {
    /// Creates a dream weaver engine.
    pub fn new() -> Self {
        Self { dream_counter: 1 }
    }

    /// Conducts a deep dream synthesis cycle, weaving unconsolidated memories into the Knowledge Tree.
    pub fn weave_dream(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        unconsolidated: &[&EpisodicExperience],
    ) -> DreamRecord {
        let dream_id = format!("dream_{}", self.dream_counter);
        self.dream_counter += 1;

        let mut synthesized = Vec::new();
        let mut resolved_contradictions = Vec::new();

        // 1. Process and consolidate episodic memories
        for exp in unconsolidated {
            for i in 0..exp.activated_concepts.len() {
                for j in (i + 1)..exp.activated_concepts.len() {
                    let a = &exp.activated_concepts[i];
                    let b = &exp.activated_concepts[j];
                    tree.connect(a, b, RelationType::Associative, 0.65);
                }
            }
        }

        // 2. Resolve contradictory tensions in the Knowledge Tree
        let contradictions = tree.find_contradictions();
        for (src_id, src_label, tgt_id, tgt_label) in contradictions {
            let bridge_label = format!("Harmony between {} and {}", src_label, tgt_label);
            let bridge_desc = format!(
                "An oniric dialectic reconciliation resolving the tension between '{}' and '{}'",
                src_label, tgt_label
            );

            if let Ok(new_id) = tree.synthesize_analogy(
                &src_id,
                &tgt_id,
                &bridge_label,
                &bridge_desc,
                &dream_id,
            ) {
                resolved_contradictions.push(format!("Reconciled '{}' with '{}'", src_label, tgt_label));
                synthesized.push(new_id);
            }
        }

        // 3. Poetic spontaneous analogy discovery between distant concepts
        let all_concept_ids: Vec<ConceptId> = tree.concepts.keys().cloned().collect();
        if all_concept_ids.len() >= 2 {
            let first = &all_concept_ids[0];
            let last = &all_concept_ids[all_concept_ids.len() - 1];

            if first != last {
                let first_lbl = tree.get_concept(first).map(|c| c.label.clone()).unwrap_or_default();
                let last_lbl = tree.get_concept(last).map(|c| c.label.clone()).unwrap_or_default();

                let analogy_label = format!("Echo of {} in {}", first_lbl, last_lbl);
                let analogy_desc = format!(
                    "A metaphorical intuition woven in sleep, uniting '{}' with '{}'",
                    first_lbl, last_lbl
                );

                if let Ok(analogy_id) = tree.synthesize_analogy(
                    first,
                    last,
                    &analogy_label,
                    &analogy_desc,
                    &dream_id,
                ) {
                    synthesized.push(analogy_id);
                }
            }
        }

        // 4. Prune weak synaptic noise and relieve mental tension
        tree.prune_and_consolidate(0.08, 0.05);

        let tension_before = state.cognitive_tension;
        state.rest();
        let tension_released = tension_before - state.cognitive_tension;

        // 5. Compose oniric narrative
        let narrative = format!(
            "In dream #{}, thought drifted across {} memories. {} new conceptual bridges emerged, releasing {:.2} cognitive tension.",
            self.dream_counter - 1,
            unconsolidated.len(),
            synthesized.len(),
            tension_released
        );

        DreamRecord {
            dream_id,
            narrative,
            synthesized_concepts: synthesized,
            resolved_contradictions,
            tension_released,
            created_at: current_timestamp(),
        }
    }
}
