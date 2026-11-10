extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dream::cycle::CircadianPhase;
use crate::dream::journal::{DreamOracle, DreamRecord};
use crate::mind::curiosity::OpenQuestion;
use crate::mind::reflection::MistakeLesson;
use crate::mind::socratic::SocraticReflection;
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;
use crate::swarm::brain::CognitiveBrain;
use crate::swarm::cube::{HamonCubeTopology, NodeCoord};
use crate::swarm::job::DistributedKpackJob;

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
    pub fn interact(
        &mut self,
        stimulus: &str,
        relational_affinity: f32,
    ) -> (String, Vec<ConceptId>) {
        let mut answers = Vec::new();
        let mut all_activated = Vec::new();

        for brain in self.brains.values_mut() {
            if brain.phase == CircadianPhase::Awake {
                let (resp, activated) = brain.engage(stimulus, relational_affinity);
                answers.push(resp);
                for cid in activated {
                    if !all_activated.contains(&cid) {
                        all_activated.push(cid);
                    }
                }
            }
        }

        let combined_response = if answers.is_empty() {
            "All brains are currently in deep oniric synthesis.".into()
        } else {
            answers[0].clone()
        };

        (combined_response, all_activated)
    }

    /// Propagates a family-taught concept to all awake brains in the swarm.
    pub fn teach_awake_cohort(
        &mut self,
        member_id: &str,
        concept_id: &str,
        label: &str,
        description: &str,
        tags: &[&str],
    ) -> ConceptId {
        let mut out_id = ConceptId::from(concept_id);
        for brain in self.brains.values_mut() {
            if brain.phase == CircadianPhase::Awake {
                out_id = brain.learner.teach_from_family(
                    &mut brain.tree,
                    &mut brain.state,
                    member_id,
                    concept_id,
                    label,
                    description,
                    tags,
                );
            }
        }
        out_id
    }

    /// Propagates a gentle mistake correction to the awake cohort.
    pub fn correct_awake_cohort(
        &mut self,
        experience_id: &str,
        corrected_label: &str,
        family_explanation: &str,
    ) -> Vec<MistakeLesson> {
        let mut lessons = Vec::new();
        for brain in self.brains.values_mut() {
            if brain.phase == CircadianPhase::Awake {
                let lesson = brain.learner.learn_from_mistake(
                    &mut brain.tree,
                    &mut brain.state,
                    experience_id,
                    corrected_label,
                    family_explanation,
                );
                lessons.push(lesson);
            }
        }
        lessons
    }

    /// Collects all unique pending curiosity questions across the swarm.
    pub fn collect_pending_questions(&self) -> Vec<OpenQuestion> {
        let mut questions = Vec::new();
        for brain in self.brains.values() {
            for q in brain.learner.pending_questions() {
                if !questions
                    .iter()
                    .any(|existing: &OpenQuestion| existing.question_text == q.question_text)
                {
                    questions.push(q.clone());
                }
            }
        }
        questions
    }

    /// Collects all unique Socratic self-reflections generated across the swarm during learning.
    pub fn collect_socratic_reflections(&self) -> Vec<SocraticReflection> {
        let mut reflections = Vec::new();
        for brain in self.brains.values() {
            for r in brain.learner.socratic_reflections() {
                if !reflections.iter().any(|existing: &SocraticReflection| {
                    existing.interior_monologue == r.interior_monologue
                        && existing.examined_hypothesis == r.examined_hypothesis
                }) {
                    reflections.push(r.clone());
                }
            }
        }
        reflections
    }

    /// Collects all dream oracles discovered across the 8 brains.
    pub fn collect_oracles(&self) -> Vec<DreamOracle> {
        let mut oracles = Vec::new();
        for brain in self.brains.values() {
            for o in brain.journal.all_oracles() {
                oracles.push(o.clone());
            }
        }
        oracles
    }

    /// Executes a distributed Kpack job across the 8 logical nodes of the Hamon Cube.
    pub fn execute_distributed_kpack_job(&self, payload: &[u8]) -> DistributedKpackJob {
        DistributedKpackJob::dispatch(payload)
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

    /// Synchronizes newly synthesized concepts and relational edges across hypercube neighbors.
    pub fn synchronize_knowledge(&mut self) {
        let mut cross_injections: Vec<(NodeCoord, KnowledgeTree)> = Vec::new();

        for (coord, brain) in &self.brains {
            if brain.phase == CircadianPhase::Crystallization
                || brain.phase == CircadianPhase::DeepDream
            {
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
                    for edge in &source_tree.edges {
                        neighbor_brain.tree.connect(
                            &edge.source,
                            &edge.target,
                            edge.relation_type.clone(),
                            edge.weight,
                        );
                    }
                }
            }
        }
    }
}
