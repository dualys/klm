extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dream::cycle::CircadianPhase;
use crate::dream::journal::{DreamJournal, DreamRecord};
use crate::dream::weaver::DreamWeaver;
use crate::mind::learning::OrganicLearner;
use crate::mind::state::CognitiveState;
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;
use crate::swarm::cube::NodeCoord;

/// An autonomous cognitive node (brain) positioned on the Hamon Cube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveBrain {
    /// 3-bit hypercube node coordinate
    pub coord: NodeCoord,
    /// Local perspective of the Knowledge Tree
    pub tree: KnowledgeTree,
    /// Internal cognitive and affective state
    pub state: CognitiveState,
    /// Organic experiential learning engine
    pub learner: OrganicLearner,
    /// Local oniric weaver
    pub weaver: DreamWeaver,
    /// Local dream journal history
    pub journal: DreamJournal,
    /// Current circadian phase (Awake, Drowsiness, DeepDream, Crystallization)
    pub phase: CircadianPhase,
}

impl CognitiveBrain {
    /// Initializes a cognitive brain at a specific hypercube node.
    pub fn new(coord: NodeCoord, innate_tree: KnowledgeTree) -> Self {
        Self {
            coord,
            tree: innate_tree,
            state: CognitiveState::new(),
            learner: OrganicLearner::new(),
            weaver: DreamWeaver::new(),
            journal: DreamJournal::new(),
            phase: CircadianPhase::Awake,
        }
    }

    /// Engages with a stimulus during wakefulness.
    pub fn engage(&mut self, stimulus: &str, relational_affinity: f32) -> (String, Vec<ConceptId>) {
        if self.phase == CircadianPhase::Awake {
            self.learner.process_stimulus(&mut self.tree, &mut self.state, stimulus, relational_affinity)
        } else {
            ("I am currently immersed in deep sleep and oniric consolidation.".into(), Vec::new())
        }
    }

    /// Triggers nocturnal dreaming and memory consolidation if in DeepDream or sleep phase.
    pub fn dream(&mut self) -> DreamRecord {
        let unconsolidated: Vec<_> = self.learner.unconsolidated_episodes();
        let record = self.weaver.weave_dream(&mut self.tree, &mut self.state, &unconsolidated);

        for exp in &mut self.learner.episodic_stream {
            exp.mark_consolidated();
        }

        self.journal.record_dream(record.clone());
        record
    }
}
