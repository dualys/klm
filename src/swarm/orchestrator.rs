extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dream::cycle::CircadianPhase;
use crate::dream::journal::DreamRecord;
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;
use crate::swarm::brain::CognitiveBrain;
use crate::swarm::cube::{HamonCubeTopology, NodeCoord};

/// Orchestrator for the 8-brain multi-cognitive Hamon Cube swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmOrchestrator {
    /// Network topology
    pub topology: HamonCubeTopology,
    /// The 8 individual cognitive brains indexed by their node coordinate
    pub brains: BTreeMap<NodeCoord, CognitiveBrain>,
    /// Global swarm step / epoch counter
    pub global_epoch: u64,
}

impl Default for SwarmOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl SwarmOrchestrator {
    /// Initializes an 8-node swarm with an innate knowledge seed in each brain.
    pub fn new() -> Self {
        let topology = HamonCubeTopology::new();
        let mut brains = BTreeMap::new();

        for coord in &topology.nodes {
            let innate_tree = KnowledgeTree::with_innate_seed();
            let mut brain = CognitiveBrain::new(*coord, innate_tree);

            // Stagger initial circadian phases across cohorts
            match coord.0 {
                0..=2 => brain.phase = CircadianPhase::Awake,
                3..=5 => brain.phase = CircadianPhase::DeepDream,
                _ => brain.phase = CircadianPhase::Crystallization,
            }

            brains.insert(*coord, brain);
        }

        Self {
            topology,
            brains,
            global_epoch: 0,
        }
    }

    /// Broadcasts an interactive stimulus to the currently awake cohort of brains.
    pub fn interact(&mut self, stimulus: &str, relational_affinity: f32) -> (String, Vec<ConceptId>) {
        let mut answers = Vec::new();
        let mut all_activated = Vec::new();

        for brain in self.brains.values_mut() {
            if brain.phase == CircadianPhase::Awake {
                let (resp, activated) = brain.engage(stimulus, relational_affinity);
                answers.push(resp);
                all_activated.extend(activated);
            }
        }

        let combined_response = if answers.is_empty() {
            "All brains are currently in deep oniric synthesis.".into()
        } else {
            answers[0].clone()
        };

        (combined_response, all_activated)
    }

    /// Triggers nocturnal dreaming for all brains currently in DeepDream phase.
    pub fn process_dreams(&mut self) -> Vec<DreamRecord> {
        let mut records = Vec::new();

        for brain in self.brains.values_mut() {
            if brain.phase == CircadianPhase::DeepDream {
                let record = brain.dream();
                records.push(record);
            }
        }

        records
    }

    /// Rotates the circadian cohorts to ensure continuous vigilance while allowing every brain to dream.
    pub fn rotate_circadian_cohorts(&mut self) {
        self.global_epoch += 1;

        for brain in self.brains.values_mut() {
            brain.phase = match brain.phase {
                CircadianPhase::Awake => CircadianPhase::DrowsinessRamp,
                CircadianPhase::DrowsinessRamp => CircadianPhase::DeepDream,
                CircadianPhase::DeepDream => CircadianPhase::Crystallization,
                CircadianPhase::Crystallization => CircadianPhase::Awake,
            };
        }
    }

    /// Synchronizes newly synthesized concepts across hypercube neighbors.
    pub fn synchronize_knowledge(&mut self) {
        let mut cross_injections: Vec<(NodeCoord, KnowledgeTree)> = Vec::new();

        for (coord, brain) in &self.brains {
            if brain.phase == CircadianPhase::Crystallization {
                cross_injections.push((*coord, brain.tree.clone()));
            }
        }

        for (source_coord, source_tree) in cross_injections {
            let neighbors = source_coord.direct_neighbors();
            for n_coord in &neighbors {
                if let Some(neighbor_brain) = self.brains.get_mut(n_coord) {
                    for (id, concept) in &source_tree.concepts {
                        if !neighbor_brain.tree.concepts.contains_key(id) {
                            neighbor_brain.tree.add_concept(concept.clone());
                        }
                    }
                }
            }
        }
    }
}
