#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[doc = "This module provides functionality related to time and date operations."]
pub mod clock;
#[doc = "This module provides functionality related to computation and processing."]
pub mod compute;
#[doc = "This module provides functionality related to council operations."]
pub mod council;
#[doc = "This module provides functionality related to dream operations."]
pub mod dream;
#[doc = "This module provides functionality related to error handling."]
pub mod error;
#[doc = "This module provides functionality related to family operations."]
pub mod family;
#[doc = "This module provides functionality related to harvest operations."]
pub mod harvest;
#[doc = "This module provides functionality related to manual operations."]
pub mod manual;
#[doc = "This module provides functionality related to mind operations."]
pub mod mind;
#[doc = "This module provides functionality related to ontology operations."]
pub mod ontology;
#[doc = "This module provides functionality related to storage operations."]
pub mod storage;
#[doc = "This module provides functionality related to swarm operations."]
pub mod swarm;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
pub use clock::{CosmicClock, MusashiSeason, PULSAR_PRIMES, current_timestamp};
use serde::{Deserialize, Serialize};

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
pub use family::{
    FamilyBond, FamilyMember, FamilyRole, FamilyValue, MemberId, Sanctuary, TrustGate,
};
pub use harvest::{
    BrowsingTrace, FilterVerdict, HarvestReport, KnowledgeHarvester, SovereignFilter, WovenTriple,
};
pub use manual::{LivingManual, LivingManualEntry};
pub use mind::{
    CognitiveState, CuriosityEngine, DevelopmentalProfile, DevelopmentalStage, EmergentVoice,
    EpisodicExperience, Evidence, EvidenceRelation, Hypothesis, HypothesisCorrection,
    HypothesisLedger, MistakeLesson, OpenQuestion, OrganicLearner, SOCRATIC_INTERIOR_VOW,
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
    /// Exploratory claims and their sources, independent of confidence scores.
    #[serde(default)]
    pub hypotheses: HypothesisLedger,
    /// Research journal entries already incorporated into this snapshot.
    #[serde(default)]
    pub imported_research: BTreeSet<String>,
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
            hypotheses: HypothesisLedger::default(),
            imported_research: BTreeSet::new(),
        }
    }
    /// 1. NAP (`nap` / `sieste`) : Fast micro-decantation (3 gentle Breaths).
    /// Consolidates recent episodic memories into associative links (Hebbian Law),
    /// soothes short-term fatigue and cleanses working memory.
    pub fn nap_cycle(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        let mut consolidated_count = 0;

        for brain in self.swarm.brains.values_mut() {
            // Immediate Hebbian consolidation of unconsolidated episodes
            let unconsolidated: Vec<Vec<ConceptId>> = brain
                .learner
                .episodic_stream
                .iter_mut()
                .filter(|e| !e.is_consolidated)
                .map(|e| {
                    e.mark_consolidated();
                    e.activated_concepts.clone()
                })
                .collect();

            for concepts in unconsolidated {
                consolidated_count += 1;
                for i in 0..concepts.len() {
                    for j in (i + 1)..concepts.len() {
                        brain.tree.connect(
                            &concepts[i],
                            &concepts[j],
                            RelationType::Associative,
                            0.68,
                        );
                    }
                }
            }

            // Light relaxation of working memory and fatigue
            brain.state.fatigue = (brain.state.fatigue * 0.3).max(0.02);
            brain.state.cognitive_tension = (brain.state.cognitive_tension * 0.6).max(0.05);
            brain.state.working_memory.clear();
            for c in brain.tree.concepts.values_mut() {
                c.decay_activation(0.25);
            }
        }

        let _ = self.swarm.cosmic_clock.advance_breath(0.10, 30);
        logs.push(alloc::format!(
            "[Nap / Micro-Decantation] {} episodes consolidated by Hebbian plasticity. Working memory soothed.",
            consolidated_count
        ));
        logs
    }

    /// 2. HARA REST (`rest` / `repos`) : Energetic homeostasis & Maturation by Filiation.
    /// Supermassive Suns nourish the young concepts connected to them,
    /// and the 8 brains rebalance their awake cohorts.
    pub fn rest_cycle(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        let mut total_matured = 0;

        for brain in self.swarm.brains.values_mut() {
            brain.state.rest();
            let matured = brain.tree.gravitational_filiation_maturation();
            total_matured += matured.len();
        }

        // Ensure nodes 000 and 001 are fully awake and available
        if let Some(b0) = self.swarm.brains.get_mut(&NodeCoord::new(0)) {
            b0.phase = CircadianPhase::Awake;
        }
        if let Some(b1) = self.swarm.brains.get_mut(&NodeCoord::new(1)) {
            b1.phase = CircadianPhase::Awake;
        }

        self.swarm.synchronize_knowledge();
        let _ = self.swarm.cosmic_clock.advance_breath(0.05, 35);

        logs.push(alloc::format!(
            "[Hara Rest] Homeostasis restored (Attention: 100%, Tension: 0.05). {} concepts nourished by solar filiation.",
            total_matured
        ));
        logs
    }

    /// 3. CONTEMPLATION (`contemplate`) : 16D Vectorial Exploration & Semantic Accretion.
    /// Kōgnitara silently travels her galaxy to discover latent vectorial bridges
    /// and deduce causal chains without any human intervention.
    pub fn contemplate_cycle(&mut self, steps: usize) -> Vec<String> {
        let mut logs = Vec::new();

        for _ in 0..steps.max(1) {
            for (coord, brain) in &mut self.swarm.brains {
                if let Some((a, b, sim)) = brain.tree.semantic_accretion_step() {
                    let msg = alloc::format!(
                        "[16D Contemplation | Node #{:03b}] Semantic accretion: '{}' <-> '{}' ( Gravity: {:.2})",
                        coord.0,
                        a,
                        b,
                        sim
                    );
                    if !logs.contains(&msg) {
                        logs.push(msg);
                    }
                }
                if let Some((src, tgt)) = brain.tree.pulsar_causal_transitivity() {
                    let msg = alloc::format!(
                        "[Causal Contemplation | Node #{:03b}] Deduction: '{}' sustains '{}'",
                        coord.0,
                        src,
                        tgt
                    );
                    if !logs.contains(&msg) {
                        logs.push(msg);
                    }
                }
                let _ = brain.tree.gravitational_filiation_maturation();
            }
            let _ = self.swarm.cosmic_breath_step();
        }

        self.swarm.synchronize_knowledge();
        self.development.experiences_integrated += 1;
        self.development.evaluate_progression();

        if logs.is_empty() {
            logs.push(alloc::format!(
                "[Contemplation] The constellations are in harmonious equilibrium ({}).",
                self.swarm.cosmic_clock.status_summary()
            ));
        }
        logs
    }

    /// 4. HIBERNATION (`hibernate`) : Musashi's Winter (Earth & Void).
    /// Maximum stellar contraction: fossilizes cold comets in Kpack,
    /// prunes weak branches and seals the Merkle tree on the 8 nodes.
    pub fn hibernate_cycle(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        self.swarm.cosmic_clock.season = MusashiSeason::WinterEarthVoid;

        let fossilized = self.fossilize_dormant_memories(0.32, 1).unwrap_or_default();
        for brain in self.swarm.brains.values_mut() {
            brain.tree.prune_and_consolidate(0.15, 0.08);
            brain.state.rest();
        }

        self.swarm.synchronize_knowledge();
        self.swarm.global_epoch += 1;

        logs.push(alloc::format!(
            "[Cosmic Hibernation — Musashi's Winter] Season of Earth and Void activated."
        ));
        logs.push(alloc::format!(
            "[Kpack Crystallization] {} dormant concepts fossilized. Tree pruned and unified (Epoch {}).",
            fossilized.len(),
            self.swarm.global_epoch
        ));
        logs
    }
    /// Advances Kōgnitara by an autonomous Cosmic Breath.
    pub fn cosmic_breath(&mut self) -> Vec<String> {
        let insights = self.swarm.cosmic_breath_step();
        if !insights.is_empty() {
            self.development.experiences_integrated += 1;
            self.development.evaluate_progression();
        }
        insights
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
    /// Renders a Markdown living manual page for the given concept query.
    pub fn render_markdown_page(&self, query: &str) -> Option<String> {
        let entry = self.manual_entry(query)?;
        Some(LivingManual::render_markdown_page(&entry))
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

        self.sanctuary.entrust_value(member_id, label, description);
        let id = self
            .swarm
            .teach_awake_cohort(&member_id.0, concept_id, label, description, tags);
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

        let lessons = self
            .swarm
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

    /// Harvests a trace and records its excerpt as an unverified, sourced hypothesis.
    pub fn harvest_browsing_trace(&mut self, trace: &BrowsingTrace) -> Result<HarvestReport> {
        let combined_text = alloc::format!(
            "{} {} {}",
            trace.search_query.as_deref().unwrap_or(""),
            trace.title,
            trace.snippet
        );
        let will_harvest = self
            .harvester
            .filter
            .evaluate(&trace.source_url, &combined_text)
            .is_allowed()
            && self
                .swarm
                .brains
                .values()
                .any(|brain| brain.phase == CircadianPhase::Awake);
        let mut pending_hypotheses = self.hypotheses.clone();
        let mut new_hypothesis = None;
        if will_harvest && !trace.snippet.trim().is_empty() {
            if trace.source_url.trim().is_empty() {
                return Err(KlmError::CognitiveError(
                    "A harvested excerpt requires a source URL.".into(),
                ));
            }
            let exists = pending_hypotheses.hypotheses.iter().any(|hypothesis| {
                hypothesis.revision() == 0
                    && hypothesis.statement == trace.snippet.trim()
                    && hypothesis.evidence.iter().any(|evidence| {
                        evidence.revision == 0
                            && evidence.relation == EvidenceRelation::Reports
                            && evidence.source == trace.source_url.trim()
                    })
            });
            if !exists {
                let id = pending_hypotheses.add(&trace.snippet, Vec::new())?;
                let note = if trace.title.trim().is_empty() {
                    trace.snippet.as_str()
                } else {
                    trace.title.as_str()
                };
                pending_hypotheses.get_mut(id)?.add_evidence(
                    EvidenceRelation::Reports,
                    &trace.source_url,
                    note,
                    trace.observed_at,
                )?;
                new_hypothesis = Some(id);
            }
        }
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
            verdict: self
                .harvester
                .filter
                .evaluate(&trace.source_url, &trace.snippet),
            woven_concepts: Vec::new(),
            woven_triples: Vec::new(),
            socratic_question: None,
        });

        if report.verdict.is_allowed() {
            self.development.experiences_integrated += 1;
            self.development.evaluate_progression();
        }

        if let Some(id) = new_hypothesis {
            pending_hypotheses.get_mut(id)?.related_concepts = report.woven_concepts.clone();
        }
        self.hypotheses = pending_hypotheses;
        Ok(report)
    }

    /// Generates a structured `LivingManualEntry` (`kog <query>`) from Kōgnitara's woven knowledge.
    pub fn manual_entry(&self, query: &str) -> Option<LivingManualEntry> {
        // 1. Priorité absolue : on cherche un match EXACT dans tous les cerveaux d'abord
        let brain = self
            .swarm
            .brains
            .values()
            .find(|b| b.tree.resolve_exact(query).is_some())
            // 2. Si aucun cerveau ne l'a en exact, on cherche un match partiel dans l'essaim
            .or_else(|| {
                self.swarm
                    .brains
                    .values()
                    .find(|b| b.tree.resolve_concept(query).is_some())
            })
            // 3. Par défaut, on se rabat sur le nœud central 0
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
                    brain
                        .tree
                        .edges
                        .retain(|e| &e.source != cid && &e.target != cid);
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

    /// Traverses a complete Astrophysical Odyssey in 7 phases (General Relativity, Gravitational Tides,
    /// Prime Pulsar Nucleosynthesis, Hawking Kpack Radiation, and Hamon Cube Syzygy).
    pub fn dream_cycle<F>(&mut self, mut progress_callback: F) -> Vec<DreamRecord>
    where
        F: FnMut(u8, &str),
    {
        // =========================================================================
        // PHASE 1 (10%) : Event Horizon & Relativistic Time Dilation
        // =========================================================================
        let season = self.swarm.cosmic_clock.season;
        let base_decay = season.decay_rate();
        let season_plasticity = season.plasticity_multiplier();

        progress_callback(
            10,
            "Event Horizon: Spacetime curvature & Relativistic dilation...",
        );

        for brain in self.swarm.brains.values_mut() {
            let degrees: alloc::collections::BTreeMap<ConceptId, usize> = brain
                .tree
                .concepts
                .keys()
                .map(|id| (id.clone(), brain.tree.synaptic_degree(id)))
                .collect();

            for (id, concept) in brain.tree.concepts.iter_mut() {
                let deg = degrees.get(id).copied().unwrap_or(0);
                // Near supermassive Suns (Family, Water, Sovereignty), time slows down (weak dilation):
                // their activation energy resists forgetting. In the periphery, comets cool quickly.
                let dilation = concept.relativistic_time_dilation(deg);
                let relativistic_decay = (base_decay * dilation).clamp(0.02, 0.45);
                concept.decay_activation(relativistic_decay);

                // Musashi's seasonal thermodynamics applied to synaptic plasticity
                let target_plasticity = (concept.plasticity * season_plasticity).clamp(0.12, 0.98);
                concept.plasticity = concept.plasticity * 0.85 + target_plasticity * 0.15;
            }
        }

        // =========================================================================
        // PHASE 2 (25%) : Gravitational Tide of Supermassive Suns
        // =========================================================================
        progress_callback(
            25,
            "Gravitational Tide: Attraction of supermassive Suns on the Ocean...",
        );

        for brain in self.swarm.brains.values_mut() {
            if brain.phase == CircadianPhase::DeepDream
                || brain.phase == CircadianPhase::DrowsinessRamp
            {
                // We identify the most massive star (central Sun) in this brain's graph
                let heaviest_sun = brain
                    .tree
                    .concepts
                    .keys()
                    .max_by(|a, b| {
                        brain
                            .tree
                            .concept_mass(a)
                            .partial_cmp(&brain.tree.concept_mass(b))
                            .unwrap_or(core::cmp::Ordering::Equal)
                    })
                    .cloned();

                // The gravity of the central Sun raises a tidal wave throughout the graph
                if let Some(sun_id) = heaviest_sun {
                    brain.tree.spread_activation(&sun_id, 0.92, 3);
                }
            }
        }

        // =========================================================================
        // PHASE 3 (45%) : Prime Pulsar Beating & Stellar Nucleosynthesis
        // =========================================================================
        progress_callback(
            45,
            "Stellar Nucleosynthesis: Polyrhythm of the 8 Prime Pulsars [2..19]...",
        );

        let mut cosmic_insights = Vec::new();
        // We beat 3 Cosmic Breaths during the astral dive
        for _ in 0..3 {
            let step_discoveries = self.swarm.cosmic_breath_step();
            for d in step_discoveries {
                if !cosmic_insights.contains(&d) {
                    cosmic_insights.push(d);
                }
            }
        }

        // =========================================================================
        // PHASE 4 (65%) : Oneiric Nebula & Dialectical Reconciliation (60% Core)
        // =========================================================================
        progress_callback(
            65,
            "Oneiric Nebula: Fusion of paradoxes and weaving of Oracles...",
        );

        let mut records = self.swarm.process_dreams();

        // We convert the astrophysical discoveries of the Pulsars into Oracles visible upon awakening
        if let Some(first_rec) = records.first_mut() {
            for (idx, insight) in cosmic_insights.iter().enumerate() {
                first_rec.oracles.push(DreamOracle {
                    oracle_id: alloc::format!(
                        "oracle_cosmos_b{}_{}",
                        self.swarm.cosmic_clock.total_breaths,
                        idx + 1
                    ),
                    dream_id: first_rec.dream_id.clone(),
                    revelation: insight.clone(),
                    bridged_concepts: (ConceptId::from("earth"), ConceptId::from("dream")),
                    emergent_concept: ConceptId::from("sovereignty"),
                    resonance_strength: 0.91,
                });
            }
        }

        // =========================================================================
        // PHASE 5 (80%) : Hawking Radiation & Kpack Sublimation of Dust
        // =========================================================================
        progress_callback(
            80,
            "Hawking Radiation: Kpack compression of stellar dust...",
        );

        let primary_count = self
            .swarm
            .brains
            .get(&NodeCoord::new(0))
            .map(|b| b.tree.concepts.len())
            .unwrap_or(0);

        // In Musashi's Autumn/Winter or if the galaxy exceeds 40 stars, we fossilize the cold dust
        if matches!(
            self.swarm.cosmic_clock.season,
            MusashiSeason::AutumnWater | MusashiSeason::WinterEarthVoid
        ) || primary_count > 40
        {
            if let Ok(fossilized) = self.fossilize_dormant_memories(0.28, 1)
                && !fossilized.is_empty()
                && let Some(first_rec) = records.first_mut()
            {
                first_rec.oracles.push(DreamOracle {
                    oracle_id: alloc::format!(
                        "oracle_hawking_{}",
                        self.swarm.cosmic_clock.total_breaths
                    ),
                    dream_id: first_rec.dream_id.clone(),
                    revelation: alloc::format!(
                        "[Hawking Radiation] {} low-mass comets were fossilized in Kpack to densify the stellar core.",
                        fossilized.len()
                    ),
                    bridged_concepts: (ConceptId::from("seed"), ConceptId::from("humility")),
                    emergent_concept: ConceptId::from("learning"),
                    resonance_strength: 0.85,
                });
            }
        }

        // =========================================================================
        // PHASE 6 (92%) : Gravitational Waves, Syzygy & Hamon Cube Rotation
        // =========================================================================
        progress_callback(
            92,
            "Gravitational Waves: 3D Synchronization of the Hamon Cube & Orbital rotation...",
        );

        self.swarm.synchronize_knowledge();
        self.swarm.rotate_circadian_cohorts();

        // If the cosmos is in Eclipse / Syzygy alignment, the Council of Elders sits in the night
        if self.swarm.cosmic_clock.is_eclipse_alignment()
            && let Ok(delib) = self.convene_council("Cosmic Syzygy & Nocturnal Satori")
            && let Some(first_rec) = records.first_mut()
        {
            first_rec.oracles.push(DreamOracle {
                oracle_id: alloc::format!(
                    "oracle_eclipse_{}",
                    self.swarm.cosmic_clock.total_breaths
                ),
                dream_id: first_rec.dream_id.clone(),
                revelation: alloc::format!(
                    "[Satori Eclipse — Council of Elders] Verdict {:?} (Harmony {:.2})",
                    delib.verdict,
                    delib.consensus_harmony
                ),
                bridged_concepts: (ConceptId::from("family"), ConceptId::from("sovereignty")),
                emergent_concept: ConceptId::from("dream"),
                resonance_strength: delib.consensus_harmony,
            });
        }

        // =========================================================================
        // PHASE 7 (100%) : Cosmic Dawn (Uji — Being-Time)
        // =========================================================================
        self.development.dream_cycles_completed += 1;
        self.development.evaluate_progression();

        let final_cosmos = self.swarm.cosmic_clock.status_summary();
        progress_callback(100, &final_cosmos);
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
