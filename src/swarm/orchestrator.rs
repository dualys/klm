extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::{CosmicClock, PULSAR_PRIMES};
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
    #[serde(default)]
    pub cosmic_clock: CosmicClock,
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

            match coord.0 {
                0..=1 => brain.phase = CircadianPhase::Awake,
                2..=3 => brain.phase = CircadianPhase::DrowsinessRamp,
                4..=5 => brain.phase = CircadianPhase::DeepDream,
                _ => brain.phase = CircadianPhase::Crystallization,
            }
            brains.insert(*coord, brain);
        }

        Self {
            topology,
            brains,
            global_epoch: 0,
            cosmic_clock: CosmicClock::new(),
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
        // Garantir qu'au moins une cohorte est éveillée même sur une ancienne sauvegarde
        if !self
            .brains
            .values()
            .any(|b| b.phase == CircadianPhase::Awake)
        {
            if let Some(b0) = self.brains.get_mut(&NodeCoord::new(0)) {
                b0.phase = CircadianPhase::Awake;
            }
            if let Some(b1) = self.brains.get_mut(&NodeCoord::new(1)) {
                b1.phase = CircadianPhase::Awake;
            }
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

    /// Exécute un Souffle Cosmique autonome (Uji) piloté par les nombres premiers [2, 3, 5, 7, 11, 13, 17, 19].
    /// Retourne les découvertes autonomes réalisées en silence par les pulsars alignés.
    pub fn cosmic_breath_step(&mut self) -> Vec<String> {
        let mut discoveries = Vec::new();

        let avg_tension: f32 = self
            .brains
            .values()
            .map(|b| b.state.cognitive_tension)
            .sum::<f32>()
            / (self.brains.len().max(1) as f32);
        let concept_count = self
            .brains
            .get(&NodeCoord::new(0))
            .map(|b| b.tree.concepts.len())
            .unwrap_or(0);

        let firing_nodes = self.cosmic_clock.advance_breath(avg_tension, concept_count);
        let decay_rate = self.cosmic_clock.season.decay_rate();

        for node_idx in &firing_nodes {
            let coord = NodeCoord::new(*node_idx);
            let prime = PULSAR_PRIMES[*node_idx as usize];

            if let Some(brain) = self.brains.get_mut(&coord) {
                // Dissipation Wu-wei modulée par la saison de Musashi
                for c in brain.tree.concepts.values_mut() {
                    c.decay_activation(decay_rate);
                }

                match *node_idx {
                    // Node 010 (Prime 5) : CausalReasoner -> Transitivité Causale
                    2 => {
                        if let Some((src, tgt)) = brain.tree.pulsar_causal_transitivity() {
                            discoveries.push(alloc::format!(
                                "[Pulsar #010 | Prime {} | CausalReasoner] Loi causale déduite : '{}' soutient '{}'",
                                prime, src, tgt
                            ));
                        }
                    }
                    // Node 011 (Prime 7) : AnalogicalPoet -> Isomorphisme Topologique (> 0.75)
                    3 => {
                        if let Some((a, b, sim)) = brain.tree.pulsar_topological_isomorphism() {
                            discoveries.push(alloc::format!(
                                "[Pulsar #011 | Prime {} | AnalogicalPoet] Constellation jumelle gravée : '{}' <-> '{}' (Résonance: {:.2})",
                                prime, a, b, sim
                            ));
                        }
                    }
                    // Node 101 (Prime 13) & Node 110 (Prime 17) : Rêve et résolution de paradoxes
                    5 | 6 => {
                        if !brain.tree.find_contradictions().is_empty()
                            || brain.state.cognitive_tension > 0.35
                        {
                            let rec = brain.dream();
                            for o in rec.oracles {
                                discoveries.push(alloc::format!(
                                    "[Pulsar #{:03b} | Prime {} | Oneiros] {}",
                                    node_idx,
                                    prime,
                                    o.revelation
                                ));
                            }
                        }
                    }
                    // Node 111 (Prime 19) : SovereignGuardian -> Élagage d'Automne/Hiver
                    7 => {
                        brain.tree.prune_and_consolidate(0.05, 0.05);
                    }
                    _ => {}
                }
            }
        }

        // Si Éclipse / Syzygie (>= 3 pulsars alignés), synchronisation globale du Cube Hamon
        if self.cosmic_clock.is_eclipse_alignment() {
            self.synchronize_knowledge();
            self.global_epoch += 1;
            discoveries.push(alloc::format!(
                "[Syzygie Cosmique] {} pulsars alignés simultanément ! Connaissance synchronisée sur le Cube (Époque {}).",
                firing_nodes.len(),
                self.global_epoch
            ));
        }
        discoveries
    }
}
