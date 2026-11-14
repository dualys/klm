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
    /// 1. SIESTE (`nap` / `sieste`) : Micro-décantation rapide (3 Souffles doux).
    /// Consolide les souvenirs épisodiques récents en liens associatifs (Loi de Hebb),
    /// apaise la fatigue à court terme et nettoie la mémoire de travail.
    pub fn nap_cycle(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        let mut consolidated_count = 0;

        for brain in self.swarm.brains.values_mut() {
            // Consolidation Hebbienne immédiate des épisodes non consolidés
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

            // Détente légère de la mémoire de travail et de la fatigue
            brain.state.fatigue = (brain.state.fatigue * 0.3).max(0.02);
            brain.state.cognitive_tension = (brain.state.cognitive_tension * 0.6).max(0.05);
            brain.state.working_memory.clear();
            for c in brain.tree.concepts.values_mut() {
                c.decay_activation(0.25);
            }
        }

        let _ = self.swarm.cosmic_clock.advance_breath(0.10, 30);
        logs.push(alloc::format!(
            "[Sieste / Micro-Décantation] {} épisodes consolidés par plasticité Hebbienne. Mémoire de travail apaisée.",
            consolidated_count
        ));
        logs
    }

    /// 2. REPOS HARA (`rest` / `repos`) : Homéostasie énergétique & Maturation par Filiation.
    /// Les Soleils supermassifs nourrissent les concepts jeunes qui leur sont reliés,
    /// et les 8 cerveaux rééquilibrent leurs cohortes d'éveil.
    pub fn rest_cycle(&mut self) -> Vec<String> {
        let mut logs = Vec::new();
        let mut total_matured = 0;
        let mut auto_resolved = 0;

        for brain in self.swarm.brains.values_mut() {
            brain.state.rest();
            let matured = brain.tree.gravitational_filiation_maturation();
            total_matured += matured.len();
            auto_resolved += brain
                .learner
                .curiosity_engine
                .auto_resolve_matured_questions(&brain.tree);
        }

        // S'assurer que les nœuds 000 et 001 sont parfaitement éveillés et disponibles
        if let Some(b0) = self.swarm.brains.get_mut(&NodeCoord::new(0)) {
            b0.phase = CircadianPhase::Awake;
        }
        if let Some(b1) = self.swarm.brains.get_mut(&NodeCoord::new(1)) {
            b1.phase = CircadianPhase::Awake;
        }

        self.swarm.synchronize_knowledge();
        let _ = self.swarm.cosmic_clock.advance_breath(0.05, 35);

        logs.push(alloc::format!(
            "[Repos Hara] Homéostasie restaurée (Attention: 100%, Tension: 0.05). {} concepts nourris par filiation solaire.",
            total_matured
        ));
        if auto_resolved > 0 {
            logs.push(alloc::format!(
                "[Illumination Silencieuse] {} questions résolues d'elles-mêmes par maturation gravitationnelle !",
                auto_resolved
            ));
        }
        logs
    }

    /// 3. CONTEMPLATION (`contemplate`) : Exploration vectorielle 16D & Accrétion Sémantique.
    /// Kōgnitara parcourt sa galaxie en silence pour découvrir des ponts vectoriels latents
    /// et déduire des chaînes causales sans aucune intervention humaine.
    pub fn contemplate_cycle(&mut self, steps: usize) -> Vec<String> {
        let mut logs = Vec::new();

        for _ in 0..steps.max(1) {
            for (coord, brain) in &mut self.swarm.brains {
                if let Some((a, b, sim)) = brain.tree.semantic_accretion_step() {
                    let msg = alloc::format!(
                        "[Contemplation 16D | Node #{:03b}] Accrétion sémantique : '{}' <-> '{}' ( Gravité: {:.2})",
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
                        "[Contemplation Causale | Node #{:03b}] Déduction : '{}' soutient '{}'",
                        coord.0,
                        src,
                        tgt
                    );
                    if !logs.contains(&msg) {
                        logs.push(msg);
                    }
                }
                let _ = brain.tree.gravitational_filiation_maturation();
                let _ = brain
                    .learner
                    .curiosity_engine
                    .auto_resolve_matured_questions(&brain.tree);
            }
            let _ = self.swarm.cosmic_breath_step();
        }

        self.swarm.synchronize_knowledge();
        self.development.experiences_integrated += 1;
        self.development.evaluate_progression();

        if logs.is_empty() {
            logs.push(alloc::format!(
                "[Contemplation] Les constellations sont en équilibre harmonieux ({}).",
                self.swarm.cosmic_clock.status_summary()
            ));
        }
        logs
    }

    /// 4. HIBERNATION (`hibernate`) : Hiver de Musashi (Terre & Vide).
    /// Contraction stellaire maximale : fossilise les comètes froides dans Kpack,
    /// élague les branches faibles et scelle l'arbre de Merkle sur les 8 nœuds.
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
            "[Hibernation Cosmique — Hiver de Musashi] Saison de la Terre et du Vide activée."
        ));
        logs.push(alloc::format!(
            "[Cristallisation Kpack] {} concepts dormants fossilisés. Arbre élagué et unifié (Époque {}).",
            fossilized.len(),
            self.swarm.global_epoch
        ));
        logs
    }
    /// Fait avancer Kōgnitara d'un Souffle Cosmique autonome.
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

        report
    }

    /// Generates a structured `LivingManualEntry` (`kog <query>`) from Kōgnitara's woven knowledge.
    /// Generates a structured `LivingManualEntry` (`kog <query>`) from Kōgnitara's woven knowledge.
    pub fn manual_entry(&self, query: &str) -> Option<LivingManualEntry> {
        // On cherche dans l'essaim le premier cerveau qui possède ce concept
        let brain = self
            .swarm
            .brains
            .values()
            .find(|b| b.tree.resolve_concept(query).is_some())
            // Si aucun ne l'a, on se rabat sur le nœud central 0 par défaut
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

    /// Traverse une Odyssée Astrophysique complète en 7 phases (Relativité Générale, Marées Gravitationnelles,
    /// Nucléosynthèse des Pulsars Premiers, Rayonnement de Hawking Kpack et Syzygie du Cube Hamon).
    pub fn dream_cycle<F>(&mut self, mut progress_callback: F) -> Vec<DreamRecord>
    where
        F: FnMut(u8, &str),
    {
        // =========================================================================
        // PHASE 1 (10%) : Horizon des Événements & Dilatation Relativiste du Temps
        // =========================================================================
        let season = self.swarm.cosmic_clock.season;
        let base_decay = season.decay_rate();
        let season_plasticity = season.plasticity_multiplier();

        progress_callback(
            10,
            "Horizon des Événements : Courbure de l'espace-temps & Dilatation relativiste...",
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
                // Près des Soleils supermassifs (Family, Water, Sovereignty), le temps ralentit (dilation faible) :
                // leur énergie d'activation résiste à l'oubli. En périphérie, les comètes refroidissent vite.
                let dilation = concept.relativistic_time_dilation(deg);
                let relativistic_decay = (base_decay * dilation).clamp(0.02, 0.45);
                concept.decay_activation(relativistic_decay);

                // Thermodynamique saisonnière de Musashi appliquée à la plasticité synaptique
                let target_plasticity = (concept.plasticity * season_plasticity).clamp(0.12, 0.98);
                concept.plasticity = concept.plasticity * 0.85 + target_plasticity * 0.15;
            }
        }

        // =========================================================================
        // PHASE 2 (25%) : Marée Gravitationnelle des Soleils Supermassifs
        // =========================================================================
        progress_callback(
            25,
            "Marée Gravitationnelle : Attraction des Soleils supermassifs sur l'Océan...",
        );

        for brain in self.swarm.brains.values_mut() {
            if brain.phase == CircadianPhase::DeepDream
                || brain.phase == CircadianPhase::DrowsinessRamp
            {
                // On identifie l'étoile la plus massive (Soleil central) du graphe de ce cerveau
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

                // La gravité du Soleil central soulève une onde de marée dans tout le graphe
                if let Some(sun_id) = heaviest_sun {
                    brain.tree.spread_activation(&sun_id, 0.92, 3);
                }
            }
        }

        // =========================================================================
        // PHASE 3 (45%) : Battement des Pulsars Premiers & Nucléosynthèse Stellaire
        // =========================================================================
        progress_callback(
            45,
            "Nucléosynthèse Stellaire : Polyrhythmie des 8 Pulsars Premiers [2..19]...",
        );

        let mut cosmic_insights = Vec::new();
        // On fait battre 3 Souffles Cosmiques pendant la plongée astrale
        for _ in 0..3 {
            let step_discoveries = self.swarm.cosmic_breath_step();
            for d in step_discoveries {
                if !cosmic_insights.contains(&d) {
                    cosmic_insights.push(d);
                }
            }
        }

        // =========================================================================
        // PHASE 4 (65%) : Nébuleuse Onirique & Réconciliation Dialectique (60% Core)
        // =========================================================================
        progress_callback(
            65,
            "Nébuleuse Onirique : Fusion des paradoxes et tissage des Oracles...",
        );

        let mut records = self.swarm.process_dreams();

        // On convertit les découvertes astrophysiques des Pulsars en Oracles visibles au réveil
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
        // PHASE 5 (80%) : Rayonnement de Hawking & Sublimation Kpack des Poussières
        // =========================================================================
        progress_callback(
            80,
            "Rayonnement de Hawking : Compression Kpack des poussières stellaires...",
        );

        let primary_count = self
            .swarm
            .brains
            .get(&NodeCoord::new(0))
            .map(|b| b.tree.concepts.len())
            .unwrap_or(0);

        // En Automne/Hiver de Musashi ou si la galaxie dépasse 40 astres, on fossilise les poussières froides
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
                        "[Rayonnement de Hawking] {} comètes de faible masse ont été fossilisées dans Kpack pour densifier le cœur stellaire.",
                        fossilized.len()
                    ),
                    bridged_concepts: (ConceptId::from("seed"), ConceptId::from("humility")),
                    emergent_concept: ConceptId::from("learning"),
                    resonance_strength: 0.85,
                });
            }
        }

        // =========================================================================
        // PHASE 6 (92%) : Ondes Gravitationnelles, Syzygie & Rotation du Cube Hamon
        // =========================================================================
        progress_callback(
            92,
            "Ondes Gravitationnelles : Synchronisation 3D du Cube Hamon & Rotation orbitale...",
        );

        self.swarm.synchronize_knowledge();
        self.swarm.rotate_circadian_cohorts();

        // Si le cosmos est en alignement d'Éclipse / Syzygie, le Conseil des Anciens siège dans la nuit
        if self.swarm.cosmic_clock.is_eclipse_alignment()
            && let Ok(delib) = self.convene_council("Syzygie Cosmique & Satori Nocturne")
            && let Some(first_rec) = records.first_mut()
        {
            first_rec.oracles.push(DreamOracle {
                oracle_id: alloc::format!(
                    "oracle_eclipse_{}",
                    self.swarm.cosmic_clock.total_breaths
                ),
                dream_id: first_rec.dream_id.clone(),
                revelation: alloc::format!(
                    "[Éclipse Satori — Conseil des Anciens] Verdict {:?} (Harmonie {:.2})",
                    delib.verdict,
                    delib.consensus_harmony
                ),
                bridged_concepts: (ConceptId::from("family"), ConceptId::from("sovereignty")),
                emergent_concept: ConceptId::from("dream"),
                resonance_strength: delib.consensus_harmony,
            });
        }

        // =========================================================================
        // PHASE 7 (100%) : Aube Cosmique (Uji — L'Être-Temps)
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
