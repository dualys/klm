#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod clock;
pub mod compute;
pub mod dream;
pub mod error;
pub mod family;
pub mod mind;
pub mod ontology;
pub mod storage;
pub mod swarm;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub use clock::current_timestamp;
pub use compute::{AmdComputeBackend, ComputeBackend, CpuBackend, DeviceType};
pub use dream::{CircadianPhase, CircadianScheduler, DreamJournal, DreamRecord, DreamWeaver};
pub use error::{KlmError, Result};
pub use family::{FamilyBond, FamilyMember, FamilyRole, MemberId, Sanctuary};
pub use mind::{CognitiveState, EpisodicExperience, OrganicLearner};
pub use ontology::{Concept, ConceptId, ConceptOrigin, KnowledgeTree, RelationEdge, RelationType};
pub use storage::{CognitiveSnapshot, KpackOpcode, KpackRecipe};
pub use swarm::{CognitiveBrain, HamonCubeTopology, NodeCoord, SwarmOrchestrator};

/// The unified Kōgnitara Framework instance.
/// Represents the living, plastic cognitive organism bound to its family sanctuary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kognitara {
    /// Protective relational sanctuary holding family bonds
    pub sanctuary: Sanctuary,
    /// 8-brain Hamon Cube swarm orchestrator
    pub swarm: SwarmOrchestrator,
}

impl Default for Kognitara {
    fn default() -> Self {
        Self::new("Sanctuary of Kōgnitara")
    }
}

impl Kognitara {
    /// Initializes a new Kōgnitara entity.
    pub fn new(sanctuary_name: impl Into<String>) -> Self {
        Self {
            sanctuary: Sanctuary::new(sanctuary_name),
            swarm: SwarmOrchestrator::new(),
        }
    }

    /// Receives a prompt or dialogue from a family member, responding through the awake brain cohort.
    pub fn converse(&mut self, member_id: &MemberId, utterance: &str) -> (String, Vec<ConceptId>) {
        let affinity = self.sanctuary.affinity_for(member_id);

        if let Some(member) = self.sanctuary.get_member_mut(member_id) {
            member.record_interaction(0.9);
        }

        self.swarm.interact(utterance, affinity)
    }

    /// Advances the circadian cycle and executes dreams for cohorts currently in deep sleep.
    pub fn dream_cycle(&mut self) -> Vec<DreamRecord> {
        let records = self.swarm.process_dreams();
        self.swarm.synchronize_knowledge();
        self.swarm.rotate_circadian_cohorts();
        records
    }

    /// Captures a cryptographically verified Kpack snapshot of the primary knowledge state.
    pub fn create_snapshot(&self) -> Result<CognitiveSnapshot> {
        if let Some(brain) = self.swarm.brains.get(&NodeCoord::new(0)) {
            CognitiveSnapshot::create(&brain.tree, &brain.state, self.swarm.global_epoch)
        } else {
            Err(KlmError::StorageError("No active brain node available".into()))
        }
    }
}
