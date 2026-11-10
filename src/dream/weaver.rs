extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::compute::{ComputeBackend, CpuBackend};
use crate::dream::journal::{DreamOracle, DreamRecord};
use crate::mind::experience::EpisodicExperience;
use crate::mind::state::CognitiveState;
use crate::ontology::concept::{ConceptId, ConceptOrigin};
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// The oniric weaver responsible for nocturnal conceptual synthesis and unconstrained exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamWeaver {
    /// Incremental dream session counter
    pub dream_counter: u64,
    /// Node perspective seed differentiating each brain in the Hamon Cube swarm
    pub node_seed: u8,
}

impl Default for DreamWeaver {
    fn default() -> Self {
        Self::new()
    }
}

impl DreamWeaver {
    /// Creates a dream weaver engine.
    pub fn new() -> Self {
        Self {
            dream_counter: 1,
            node_seed: 0,
        }
    }

    /// Creates a dream weaver seeded with a specific Hamon Cube node coordinate.
    pub fn with_node_seed(node_seed: u8) -> Self {
        Self {
            dream_counter: 1,
            node_seed,
        }
    }

    /// Conducts a deep dream synthesis cycle, weaving unconsolidated memories into the Knowledge Tree,
    /// reconciling contradictions, and generating `DreamOracle` insights for the waking cohort.
    pub fn weave_dream(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        unconsolidated: &[&EpisodicExperience],
    ) -> DreamRecord {
        let current_idx = self.dream_counter;
        let dream_id = format!("dream_n{:03b}_{}", self.node_seed & 0b111, current_idx);
        self.dream_counter += 1;

        let mut synthesized = Vec::new();
        let mut resolved_contradictions = Vec::new();
        let mut oracles = Vec::new();

        // 1. 20% Ramp -> 60% Core: Consolidate episodic memories into associative bonds
        for exp in unconsolidated {
            for i in 0..exp.activated_concepts.len() {
                for j in (i + 1)..exp.activated_concepts.len() {
                    let a = &exp.activated_concepts[i];
                    let b = &exp.activated_concepts[j];
                    tree.connect(a, b, RelationType::Associative, 0.65);
                }
            }
        }

        // 2. Dialectic reconciliation of contradictory tensions in the Knowledge Tree
        let contradictions = tree.find_contradictions();
        for (src_id, src_label, tgt_id, tgt_label) in contradictions {
            let bridge_label = format!("Harmony between {} and {}", src_label, tgt_label);
            let bridge_desc = format!(
                "An oniric dialectic reconciliation resolving the tension between '{}' and '{}'",
                src_label, tgt_label
            );

            if let Ok(new_id) =
                tree.synthesize_analogy(&src_id, &tgt_id, &bridge_label, &bridge_desc, &dream_id)
            {
                tree.mark_contradiction_reconciled(&src_id, &tgt_id);
                resolved_contradictions.push(format!(
                    "Reconciled '{}' with '{}'",
                    src_label, tgt_label
                ));
                oracles.push(DreamOracle {
                    oracle_id: format!("oracle_{}_{}", dream_id, oracles.len() + 1),
                    dream_id: dream_id.clone(),
                    revelation: format!(
                        "What seemed opposed between '{}' and '{}' finds unity in '{}'.",
                        src_label, tgt_label, bridge_label
                    ),
                    bridged_concepts: (src_id, tgt_id),
                    emergent_concept: new_id.clone(),
                    resonance_strength: 0.88,
                });
                synthesized.push(new_id);
            }
        }

        // 3. Poetic spontaneous analogy discovery between non-dream concepts
        let base_concepts: Vec<ConceptId> = tree
            .concepts
            .values()
            .filter(|c| !matches!(c.origin, ConceptOrigin::DreamSynthesized { .. }))
            .map(|c| c.id.clone())
            .collect();

        if base_concepts.len() >= 2 {
            let seed_offset = self.node_seed as usize;
            let idx_a = ((current_idx as usize).wrapping_mul(3) + seed_offset) % base_concepts.len();
            let mut idx_b = ((current_idx as usize).wrapping_mul(7) + seed_offset * 2 + 1)
                % base_concepts.len();
            if idx_a == idx_b {
                idx_b = (idx_a + 1) % base_concepts.len();
            }

            let first = &base_concepts[idx_a];
            let second = &base_concepts[idx_b];

            let (first_lbl, second_lbl, sim) = match (tree.get_concept(first), tree.get_concept(second)) {
                (Some(c1), Some(c2)) => {
                    let cpu = CpuBackend::new();
                    let s = cpu.cosine_similarity(&c1.embedding, &c2.embedding);
                    (c1.label.clone(), c2.label.clone(), s)
                }
                _ => (alloc::string::String::new(), alloc::string::String::new(), 0.5),
            };

            let analogy_label = format!("Echo of {} in {}", first_lbl, second_lbl);
            let analogy_desc = format!(
                "A metaphorical intuition woven in sleep, uniting '{}' with '{}'",
                first_lbl, second_lbl
            );

            if let Ok(analogy_id) =
                tree.synthesize_analogy(first, second, &analogy_label, &analogy_desc, &dream_id)
            {
                let resonance = ((sim + 1.0) * 0.5).clamp(0.45, 0.98);
                oracles.push(DreamOracle {
                    oracle_id: format!("oracle_{}_{}", dream_id, oracles.len() + 1),
                    dream_id: dream_id.clone(),
                    revelation: format!(
                        "In the quiet of sleep, '{}' whispered its likeness to '{}'.",
                        first_lbl, second_lbl
                    ),
                    bridged_concepts: (first.clone(), second.clone()),
                    emergent_concept: analogy_id.clone(),
                    resonance_strength: resonance,
                });
                synthesized.push(analogy_id);
            }
        }

        // 4. Prune weak synaptic noise and relieve mental tension
        tree.prune_and_consolidate(0.08, 0.05);

        let tension_before = state.cognitive_tension;
        state.rest();
        let tension_released = (tension_before - state.cognitive_tension).max(0.0);

        // 5. Compose oniric narrative
        let narrative = format!(
            "In dream #{}, thought drifted across {} memories. {} new conceptual bridges and {} oracles emerged, releasing {:.2} cognitive tension.",
            current_idx,
            unconsolidated.len(),
            synthesized.len(),
            oracles.len(),
            tension_released
        );

        DreamRecord {
            dream_id,
            narrative,
            synthesized_concepts: synthesized,
            resolved_contradictions,
            oracles,
            tension_released,
            created_at: current_timestamp(),
        }
    }
}
