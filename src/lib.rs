#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod clock;
pub mod compute;
pub mod council;
pub mod dream;
pub mod error;
pub mod family;
pub mod harvest;
pub mod manual;
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
    LucidBrainContribution, LucidDreamEngine, LucidDreamReport,
};
pub use error::{KlmError, Result};
pub use family::{FamilyBond, FamilyMember, FamilyRole, FamilyValue, MemberId, Sanctuary, TrustGate};
pub use harvest::{
    BrowsingTrace, FilterVerdict, HarvestReport, KnowledgeHarvester, SovereignFilter, WovenTriple,
};
pub use manual::{LivingManual, LivingManualEntry};
pub use mind::{
    CognitiveState, CuriosityEngine, DevelopmentalProfile, DevelopmentalStage, EmergentVoice,
    EpisodicExperience, MistakeLesson, OpenQuestion, OrganicLearner, SOCRATIC_INTERIOR_VOW,
    SocraticEngine, SocraticReflection, SocraticStage,
};
pub use ontology::{
    Concept, ConceptId, ConceptOrigin, EMBEDDING_DIM, KnowledgeTree, RelationEdge, RelationType,
    compute_text_embedding,
};
pub use storage::{
    CognitiveOcean, CognitiveSnapshot, CubeFsVault, FossilArchive, FossilizedConcept, KpackOpcode,
    KpackRecipe, MemoryBlockDevice, OceanEntrySummary, VaultCheckpoint,
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
    /// Sovereign Whitelist/Blacklist harvester for browser searches and reading traces
    pub harvester: KnowledgeHarvester,
    /// Organic developmental profile tracking childhood, adolescence, and maturity
    pub development: DevelopmentalProfile,
    /// Kpack fossil archive for gentle synaptic forgetting and Proustian Anamnesis recall
    pub fossil_archive: FossilArchive,
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
            harvester: KnowledgeHarvester::new(),
            development: DevelopmentalProfile::new(),
            fossil_archive: FossilArchive::new(),
        }
    }

    /// Receives a prompt or dialogue from a family member, responding through the awake brain cohort.
    /// Automatically checks the `FossilArchive` for Socratic Anamnesis recall before responding.
    pub fn converse(&mut self, member_id: &MemberId, utterance: &str) -> (String, Vec<ConceptId>) {
        let affinity = self.sanctuary.affinity_for(member_id);

        if let Some(member) = self.sanctuary.get_member_mut(member_id) {
            member.record_interaction(0.9);
        }

        // Trigger Anamnesis recall if the utterance resonates with a forgotten Kpack fossil
        let _ = self.anamnesis_recall(utterance);

        let result = self.swarm.interact(utterance, affinity);
        self.development.experiences_integrated += 1;
        self.development.evaluate_progression();
        result
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
        self.development.experiences_integrated += 1;
        self.development.evaluate_progression();
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
        self.development.mistakes_transformed += 1;
        self.development.evaluate_progression();
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

        self.development.council_deliberations += 1;
        self.development.evaluate_progression();
        Ok(self.council.convene(subject, &reflection, &brain.tree))
    }

    /// Harvests a browser search query or reading trace through the Sovereign Whitelist/Blacklist filter,
    /// weaving extracted atomic concepts and ecological relations into all awake brains.
    pub fn harvest_browsing_trace(&mut self, trace: &BrowsingTrace) -> HarvestReport {
        let mut primary_report: Option<HarvestReport> = None;

        for brain in self.swarm.brains.values_mut() {
            if brain.phase == CircadianPhase::Awake {
                let mut local_harvester = self.harvester.clone();
                let report = local_harvester.ingest_trace(
                    trace,
                    &mut brain.tree,
                    &mut brain.state,
                    &mut brain.learner,
                );
                if primary_report.is_none() {
                    self.harvester = local_harvester;
                    primary_report = Some(report);
                }
            }
        }

        let report = primary_report.unwrap_or_else(|| HarvestReport {
            verdict: self.harvester.filter.evaluate(&trace.source_url, &trace.snippet),
            woven_concepts: Vec::new(),
            woven_triples: Vec::new(),
            socratic_question: None,
        });

        if report.verdict.is_allowed() {
            self.development.experiences_integrated += 1;
            self.development.evaluate_progression();
        }

        report
    }

    /// Generates a structured `LivingManualEntry` (`kog <query>`) from Kōgnitara's woven knowledge.
    pub fn manual_entry(&self, query: &str) -> Option<LivingManualEntry> {
        let brain = self
            .swarm
            .brains
            .values()
            .find(|b| b.phase == CircadianPhase::Awake)
            .or_else(|| self.swarm.brains.get(&NodeCoord::new(0)))?;

        let oracles = self.dream_oracles();
        let reflections = self.socratic_reflections();

        LivingManual::generate_entry(
            &brain.tree,
            &self.sanctuary,
            &oracles,
            &reflections,
            &self.council.deliberations,
            query,
        )
    }

    /// Renders a formatted Unix-style `KOG(1)` living manual page for the given concept query.
    /// Ready to be consumed directly by an external `kog` binary.
    pub fn render_man_page(&self, query: &str) -> Option<String> {
        let entry = self.manual_entry(query)?;
        Some(LivingManual::render_man_page(&entry))
    }

    /// Renders a self-contained offline HTML page for a concept in the Local Web Ark.
    pub fn render_html_ark_page(&self, query: &str) -> Option<String> {
        let entry = self.manual_entry(query)?;
        Some(LivingManual::render_html_ark_page(&entry))
    }

    /// Renders the offline HTML index portal of the Local Web Ark containing all woven concepts.
    pub fn render_html_ark_index(&self) -> String {
        if let Some(brain) = self.swarm.brains.get(&NodeCoord::new(0)) {
            LivingManual::render_html_ark_index(&brain.tree, &self.sanctuary.sanctuary_name)
        } else {
            String::new()
        }
    }

    /// Gently fossilizes dormant, low-confidence concepts into compressed `Kpack` recipes (`FossilArchive`),
    /// preserving Innate and FamilyTaught concepts intact.
    pub fn fossilize_dormant_memories(
        &mut self,
        max_confidence: f32,
        max_experience_count: u64,
    ) -> Result<Vec<ConceptId>> {
        let mut fossilized = Vec::new();
        if let Some(primary_brain) = self.swarm.brains.get_mut(&NodeCoord::new(0)) {
            fossilized = self.fossil_archive.fossilize_dormant_concepts(
                &mut primary_brain.tree,
                max_confidence,
                max_experience_count,
            )?;
        }

        // Remove fossilized concept IDs from peer brains so active RAM stays light
        for cid in &fossilized {
            for (coord, brain) in &mut self.swarm.brains {
                if coord.0 != 0 {
                    brain.tree.concepts.remove(cid);
                    brain.tree.edges.retain(|e| &e.source != cid && &e.target != cid);
                }
            }
        }

        Ok(fossilized)
    }

    /// Triggers Socratic Anamnesis (Proustian recall), decompressing any matching `Kpack` fossil
    /// back into the living `KnowledgeTree` across the swarm.
    pub fn anamnesis_recall(&mut self, trigger_query: &str) -> Result<Vec<ConceptId>> {
        if self.fossil_archive.fossils.is_empty() {
            return Ok(Vec::new());
        }

        let mut recalled_ids = Vec::new();
        let mut recalled_concepts = Vec::new();

        if let Some(primary_brain) = self.swarm.brains.get_mut(&NodeCoord::new(0)) {
            recalled_ids = self
                .fossil_archive
                .anamnesis_recall(&mut primary_brain.tree, trigger_query)?;
            for cid in &recalled_ids {
                if let Some(c) = primary_brain.tree.get_concept(cid) {
                    recalled_concepts.push(c.clone());
                }
            }
        }

        for concept in recalled_concepts {
            for (coord, brain) in &mut self.swarm.brains {
                if coord.0 != 0 {
                    brain.tree.add_concept(concept.clone());
                }
            }
        }

        Ok(recalled_ids)
    }

    /// Initiates a shared Lucid Dream session between a trusted family member and Kōgnitara's 8-brain swarm.
    pub fn lucid_dream(
        &mut self,
        member_id: &MemberId,
        theme: &str,
        seed_concepts: &[ConceptId],
    ) -> Result<LucidDreamReport> {
        if !self.sanctuary.is_family(member_id) {
            return Err(KlmError::SanctuaryError(
                "Only trusted family members may enter a shared Lucid Dream session".into(),
            ));
        }

        let report = LucidDreamEngine::conduct_lucid_dream(
            &mut self.swarm,
            &mut self.council,
            &member_id.0,
            theme,
            seed_concepts,
        );
        self.development.dream_cycles_completed += 1;
        self.development.council_deliberations += 1;
        self.development.evaluate_progression();
        Ok(report)
    }

    /// Returns Kōgnitara's current developmental stage (`AwakeningChildhood`, `MaieuticAdolescence`, `AutonomousMaturity`).
    pub fn developmental_stage(&self) -> DevelopmentalStage {
        self.development.stage
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
        self.development.dream_cycles_completed += 1;
        self.development.evaluate_progression();
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
