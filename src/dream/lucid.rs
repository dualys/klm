extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::council::assembly::{CouncilDeliberation, CouncilOfElders};
use crate::ontology::concept::ConceptId;
use crate::ontology::relation::RelationType;
use crate::swarm::brain::BrainSpecialization;
use crate::swarm::cube::NodeCoord;
use crate::swarm::orchestrator::SwarmOrchestrator;

/// Contribution of a single specialized brain node (`000`..`111`) during a shared Lucid Dream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LucidBrainContribution {
    /// Node coordinate in the 3D Hamon Cube
    pub node: NodeCoord,
    /// Cognitive specialization of this brain
    pub specialization: BrainSpecialization,
    /// Oneiric hypothesis or perspective formulated by this node
    pub perspective: String,
    /// Emergent concept synthesized during this node's lucid exploration, if any
    pub synthesized_concept: Option<ConceptId>,
}

/// Complete report of a shared Lucid Dream session between a Family member and Kōgnitara's 8-brain swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LucidDreamReport {
    /// Unique identifier for the lucid dream session
    pub session_id: String,
    /// Family member who initiated and guided the lucid dream
    pub guided_by_member: String,
    /// Theme, enigma, or architectural question explored
    pub theme: String,
    /// Individual perspectives from the 8 specialized brains of the Hamon Cube
    pub brain_contributions: Vec<LucidBrainContribution>,
    /// Newly woven concepts across the Knowledge Tree
    pub emergent_concepts: Vec<ConceptId>,
    /// Deliberation of the Council of Elders upon awakening from the lucid dream
    pub council_deliberation: CouncilDeliberation,
    /// Timestamp of the lucid dream
    pub created_at: i64,
}

/// Engine for conducting shared Lucid Dreams where the family and Kōgnitara co-explore
/// an enigma or creative vision across all 8 nodes of the Hamon Cube.
#[derive(Debug, Clone, Default)]
pub struct LucidDreamEngine;

impl LucidDreamEngine {
    /// Conducts a collaborative Lucid Dream across the 8-node Hamon Cube swarm and convenes the Council of Elders.
    pub fn conduct_lucid_dream(
        swarm: &mut SwarmOrchestrator,
        council: &mut CouncilOfElders,
        member_id: &str,
        theme: &str,
        seed_concepts: &[ConceptId],
    ) -> LucidDreamReport {
        let session_id = format!("lucid_dream_ep{}", swarm.global_epoch + 1);
        let mut brain_contributions = Vec::new();
        let mut emergent_concepts = Vec::new();

        let coords: Vec<NodeCoord> = swarm.brains.keys().copied().collect();

        for coord in coords {
            if let Some(brain) = swarm.brains.get_mut(&coord) {
                let spec = brain.specialization;
                let mut synthesized_concept = None;

                // If at least two seed concepts exist in this brain's tree, weave a lucid bridge on node 0 and its hypercube antipode (7)
                if seed_concepts.len() >= 2 && (coord.0 == 0 || coord.0 == 7) {
                    let a = &seed_concepts[0];
                    let b = &seed_concepts[1];
                    if brain.tree.get_concept(a).is_some() && brain.tree.get_concept(b).is_some() {
                        let lucid_label = format!("Lucid Synthesis of {} ({:?})", theme, spec);
                        let lucid_desc = format!(
                            "Co-dreamed with family member '{}' on node {:03b} exploring '{}'",
                            member_id, coord.0, theme
                        );
                        if let Ok(new_id) = brain.tree.synthesize_analogy(
                            a,
                            b,
                            &lucid_label,
                            &lucid_desc,
                            &session_id,
                        ) {
                            brain.tree.connect(a, b, RelationType::Harmonizes, 0.88);
                            if !emergent_concepts.contains(&new_id) {
                                emergent_concepts.push(new_id.clone());
                            }
                            synthesized_concept = Some(new_id);
                        }
                    }
                }

                let perspective = format!(
                    "[Node {:03b} — {:?}] Explored '{}' through {} active concepts.",
                    coord.0,
                    spec,
                    theme,
                    brain.tree.concepts.len()
                );

                brain_contributions.push(LucidBrainContribution {
                    node: coord,
                    specialization: spec,
                    perspective,
                    synthesized_concept,
                });
            }
        }

        // Synchronize the lucid discoveries across the hypercube
        swarm.synchronize_knowledge();

        // Convene the Council of Elders on the lucid dream's theme
        let primary_brain = swarm
            .brains
            .get_mut(&NodeCoord::new(0))
            .expect("Node 000 exists in Hamon Cube");
        let reflection = primary_brain.learner.socratic_engine.reflect_on_learning(
            &primary_brain.tree,
            theme,
            &emergent_concepts,
        );
        let council_deliberation = council.convene(theme, &reflection, &primary_brain.tree);

        LucidDreamReport {
            session_id,
            guided_by_member: member_id.into(),
            theme: theme.into(),
            brain_contributions,
            emergent_concepts,
            council_deliberation,
            created_at: current_timestamp(),
        }
    }
}
