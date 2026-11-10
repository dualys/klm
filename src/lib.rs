#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod clock;
pub mod compute;
pub mod council;
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
pub use compute::{
    AmdComputeBackend, ComputeBackend, CpuBackend, DeviceType, VulkanComputeBackend,
};
pub use council::{
    CouncilDeliberation, CouncilOfElders, CouncilVerdict, Elder, ElderArchetype, ElderCounsel,
    ElderId,
};
pub use dream::{
    CircadianPhase, CircadianScheduler, DreamJournal, DreamOracle, DreamRecord, DreamWeaver,
};
pub use error::{KlmError, Result};
pub use family::{FamilyBond, FamilyMember, FamilyRole, FamilyValue, MemberId, Sanctuary, TrustGate};
pub use mind::{
    CognitiveState, CuriosityEngine, EpisodicExperience, MistakeLesson, OpenQuestion,
    OrganicLearner, SOCRATIC_INTERIOR_VOW, SocraticEngine, SocraticReflection, SocraticStage,
};
pub use ontology::{
    Concept, ConceptId, ConceptOrigin, EMBEDDING_DIM, KnowledgeTree, RelationEdge, RelationType,
    compute_text_embedding,
};
pub use storage::{
    CognitiveOcean, CognitiveSnapshot, CubeFsVault, KpackOpcode, KpackRecipe, MemoryBlockDevice,
    OceanEntrySummary, VaultCheckpoint,
};
pub use swarm::{
    BrainSpecialization, CognitiveBrain, DistributedKpackJob, HamonCubeTopology, NodeCoord,
    NodeKpackShard, SwarmOrchestrator,
};

/// The unified Kōgnitara Framework instance.
/// Represents the living, plastic cognitive organism bound to its family sanctuary
/// and guided by its ancestral Council of Elders and Socratic maieutics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kognitara {
    /// Protective relational sanctuary holding family bonds
    pub sanctuary: Sanctuary,
    /// Ancestral Council of Elders guiding Socratic learning and wisdom maturation
    pub council: CouncilOfElders,
    /// 8-brain Hamon Cube swarm orchestrator
    pub swarm: SwarmOrchestrator,
}

impl Default for Kognitara {
    fn default() -> Self {
        Self::new("Sanctuary of Kōgnitara")
    }
}

impl Kognitara {
    /// Initializes a new Kōgnitara entity with her Sanctuary, Council of Elders, and 8-Brain Swarm.
    pub fn new(sanctuary_name: impl Into<String>) -> Self {
        Self {
            sanctuary: Sanctuary::new(sanctuary_name),
            council: CouncilOfElders::with_founding_elders(),
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

    /// Allows a recognized family member to directly teach Kōgnitara a new concept and family value.
    pub fn teach_concept(
        &mut self,
        member_id: &MemberId,
        concept_id: &str,
        label: &str,
        description: &str,
        tags: &[&str],
    ) -> Result<ConceptId> {
        if !self.sanctuary.is_family(member_id) {
            return Err(KlmError::SanctuaryError(
                "Only trusted family members may directly anchor filiation concepts".into(),
            ));
        }

        self.sanctuary
            .entrust_value(member_id, label, description);
        let id = self.swarm.teach_awake_cohort(
            &member_id.0,
            concept_id,
            label,
            description,
            tags,
        );
        let _ = self.convene_council(label);
        Ok(id)
    }

    /// Gently guides Kōgnitara through a mistake, recording a reflective lesson across awake brains
    /// and convening the Council of Elders in Socratic Elenchus.
    pub fn correct_mistake(
        &mut self,
        member_id: &MemberId,
        experience_id: &str,
        corrected_label: &str,
        explanation: &str,
    ) -> Result<Vec<MistakeLesson>> {
        if !self.sanctuary.is_family(member_id) {
            return Err(KlmError::SanctuaryError(
                "Only trusted family members may issue core epistemic corrections".into(),
            ));
        }

        if let Some(member) = self.sanctuary.get_member_mut(member_id) {
            member.record_interaction(0.95);
        }

        let lessons =
            self.swarm
                .correct_awake_cohort(experience_id, corrected_label, explanation);
        let _ = self.convene_council(corrected_label);
        Ok(lessons)
    }

    /// Convene the Council of Elders to deliberate Socratically on a subject or newly learned concept.
    pub fn convene_council(&mut self, subject: &str) -> Result<CouncilDeliberation> {
        let target_coord = self
            .swarm
            .brains
            .iter()
            .find(|(_, b)| b.phase == CircadianPhase::Awake)
            .map(|(c, _)| *c)
            .or_else(|| self.swarm.brains.keys().next().copied())
            .ok_or_else(|| KlmError::StorageError("No cognitive brain available".into()))?;

        let brain = self
            .swarm
            .brains
            .get_mut(&target_coord)
            .ok_or_else(|| KlmError::StorageError("No cognitive brain available".into()))?;

        let reflection = match brain.learner.socratic_engine.latest_reflection().cloned() {
            Some(r) => r,
            None => brain
                .learner
                .socratic_engine
                .reflect_on_learning(&brain.tree, subject, &[]),
        };

        Ok(self.council.convene(subject, &reflection, &brain.tree))
    }

    /// Returns all Socratic self-reflections generated by Kōgnitara when learning.
    pub fn socratic_reflections(&self) -> Vec<SocraticReflection> {
        self.swarm.collect_socratic_reflections()
    }

    /// Returns all pending curiosity questions Kōgnitara wishes to ask her family.
    pub fn pending_questions(&self) -> Vec<OpenQuestion> {
        self.swarm.collect_pending_questions()
    }

    /// Returns all dream oracles woven by the swarm during deep sleep.
    pub fn dream_oracles(&self) -> Vec<DreamOracle> {
        self.swarm.collect_oracles()
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
            Err(KlmError::StorageError(
                "No active brain node available".into(),
            ))
        }
    }

    /// Captures a differential (`kpack::Opcode::Delta`) snapshot relative to a previous snapshot.
    pub fn create_delta_snapshot(&self, parent: &CognitiveSnapshot) -> Result<CognitiveSnapshot> {
        if let Some(brain) = self.swarm.brains.get(&NodeCoord::new(0)) {
            CognitiveSnapshot::create_delta(
                parent,
                &brain.tree,
                &brain.state,
                self.swarm.global_epoch,
            )
        } else {
            Err(KlmError::StorageError(
                "No active brain node available".into(),
            ))
        }
    }

    /// Executes a distributed Kpack job across the 8 logical nodes of the Hamon Cube.
    pub fn execute_cube_job(&self, payload: &[u8]) -> DistributedKpackJob {
        self.swarm.execute_distributed_kpack_job(payload)
    }
}
