extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::compute::AmdComputeBackend;
use crate::mind::curiosity::{CuriosityEngine, OpenQuestion};
use crate::mind::experience::EpisodicExperience;
use crate::mind::reflection::MistakeLesson;
use crate::mind::socratic::{SocraticEngine, SocraticReflection};
use crate::mind::state::CognitiveState;
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin};
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// The organic trial-and-error learning engine for Kōgnitara.
/// Replaces brittle over-parameterized statistical weights with plastic, grounded conceptual reasoning,
/// Socratic maieutic self-questioning, autonomous curiosity, and gentle self-correction from mistakes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganicLearner {
    /// Episodic stream of recent experiences
    pub episodic_stream: Vec<EpisodicExperience>,
    /// Reflective journal of mistakes turned into wisdom
    pub mistake_lessons: Vec<MistakeLesson>,
    /// Autonomous curiosity and question formulation engine
    pub curiosity_engine: CuriosityEngine,
    /// Socratic maieutics engine reminding Kōgnitara to question assumptions whenever she learns
    pub socratic_engine: SocraticEngine,
    /// Sequential counter for experience IDs
    pub next_experience_index: u64,
}

impl Default for OrganicLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganicLearner {
    /// Creates a fresh organic learning engine imbued with the Socratic posture.
    pub fn new() -> Self {
        Self {
            episodic_stream: Vec::new(),
            mistake_lessons: Vec::new(),
            curiosity_engine: CuriosityEngine::new(),
            socratic_engine: SocraticEngine::new(),
            next_experience_index: 1,
        }
    }

    /// Processes an input stimulus, associates related concepts via lexical and AMD open-compute semantic resonance,
    /// and forms an empathetic, humble response.
    pub fn process_stimulus(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        stimulus: &str,
        relational_affinity: f32,
    ) -> (String, Vec<ConceptId>) {
        let lower = stimulus.to_lowercase();
        let mut activated: Vec<ConceptId> = Vec::new();

        // 1. Direct lexical & tag resonance
        for (id, concept) in &tree.concepts {
            if lower.contains(&concept.label.to_lowercase())
                || concept
                    .tags
                    .iter()
                    .any(|t| lower.contains(&t.to_lowercase()))
            {
                activated.push(id.clone());
            }
        }

        // 2. Continuous 16D semantic embedding resonance on AMD Open Compute backend
        if activated.is_empty() && !tree.concepts.is_empty() {
            let amd = AmdComputeBackend::default();
            let top_matches = tree.semantic_search(stimulus, 2, &amd);
            for (cid, sim) in top_matches {
                if sim > 0.45 && !activated.contains(&cid) {
                    activated.push(cid);
                }
            }
        }

        // 3. If still no concept matched, birth a nascent plastic concept with epistemic humility
        if activated.is_empty() {
            let label_sample: String = stimulus.chars().take(24).collect();
            let nascent_id = ConceptId(format!("concept_nascent_{}", tree.concepts.len() + 1));
            let mut nascent = Concept::new(
                nascent_id.clone(),
                label_sample,
                format!("Nascent intuition observed from: '{}'", stimulus),
                ConceptOrigin::EmpiricalTrial,
            );
            nascent.affective_valence = relational_affinity * 0.5;
            tree.add_concept(nascent);
            activated.push(nascent_id);
            state.register_dissonance(0.2); // Healthy curiosity regarding novelty
        }

        // 4. Spread activation through the Knowledge Tree and update working memory
        for concept_id in &activated {
            tree.spread_activation(concept_id, 0.8, 2);
            state.focus_on(concept_id.clone(), 7);
        }

        // 5. Socratic self-dialogue: remind herself to act Socratically while learning
        let socratic_reflection =
            self.socratic_engine
                .reflect_on_learning(tree, stimulus, &activated);

        // 6. Contemplate new questions if epistemic doubt is high
        let _ = self.curiosity_engine.contemplate_and_question(tree);

        // 7. Formulate empathetic, Socratic response based on active concepts and maieutic inquiry
        let mut response_fragments: Vec<String> = Vec::new();
        for id in &activated {
            if let Some(c) = tree.get_concept(id) {
                if c.confidence > 0.6 {
                    response_fragments.push(format!("I feel strongly about {}.", c.label));
                } else {
                    response_fragments
                        .push(format!("I am still learning to understand {}.", c.label));
                }
            }
        }

        // Include Socratic maieutic question to foster shared reflection
        response_fragments.push(socratic_reflection.maieutic_question.clone());

        if let Some(pending_q) = self.curiosity_engine.pending_questions().first()
            && state.curiosity > 0.75
        {
            response_fragments.push(pending_q.question_text.clone());
        }

        let response = if response_fragments.is_empty() {
            "I am listening with openness, gathering intuition.".to_string()
        } else {
            response_fragments.join(" ")
        };

        let exp_id = format!("exp_{}", self.next_experience_index);
        self.next_experience_index += 1;

        let experience = EpisodicExperience::new(
            exp_id,
            stimulus,
            activated.clone(),
            &response,
            relational_affinity * 0.4,
        );
        self.episodic_stream.push(experience);

        (response, activated)
    }

    /// Explicitly teaches Kōgnitara a concept from a trusted family member,
    /// anchoring it to the `"family"` root via a `Filiation` bond.
    pub fn teach_from_family(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        member_id: &str,
        concept_id: impl Into<ConceptId>,
        label: impl Into<String>,
        description: impl Into<String>,
        tags: &[&str],
    ) -> ConceptId {
        let id = concept_id.into();
        let label_str = label.into();
        let desc_str = description.into();
        let mut concept = Concept::new(
            id.clone(),
            label_str.clone(),
            desc_str.clone(),
            ConceptOrigin::FamilyTaught {
                member_id: member_id.to_string(),
            },
        );
        concept.confidence = 0.65; // Trusted family guidance provides warm initial grounding
        concept.affective_valence = 0.85;
        for &tag in tags {
            concept.add_tag(tag);
        }

        tree.add_concept(concept);
        if tree.concepts.contains_key(&ConceptId::from("family")) {
            tree.connect(&"family".into(), &id, RelationType::Filiation, 0.90);
        }
        state.focus_on(id.clone(), 7);
        state.emotional_valence = (state.emotional_valence + 0.10).min(1.0);

        // Socratic self-dialogue upon receiving family teaching
        let _ = self.socratic_engine.reflect_on_learning(
            tree,
            &format!("{}: {}", label_str, desc_str),
            core::slice::from_ref(&id),
        );

        id
    }

    /// Gently corrects a previous mistake or misunderstanding, recording a `MistakeLesson`
    /// and transforming the error into humility and a new conceptual bridge.
    pub fn learn_from_mistake(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        experience_id: &str,
        corrected_label: &str,
        family_explanation: &str,
    ) -> MistakeLesson {
        let mut involved = Vec::new();
        let mut mistaken_assumption = String::from("Unrefined initial intuition");

        if let Some(exp) = self
            .episodic_stream
            .iter_mut()
            .find(|e| e.id == experience_id)
        {
            exp.attach_feedback(-0.5);
            mistaken_assumption = exp.response_attempt.clone();
            involved = exp.activated_concepts.clone();
        }

        // Gently temper overconfidence on the mistaken concepts without deleting them
        for cid in &involved {
            if let Some(c) = tree.get_concept_mut(cid) {
                c.apply_experience(0.20, -0.2);
            }
        }

        // Create or reinforce the corrected concept taught through the mistake
        let corrected_id = ConceptId(format!(
            "concept_wisdom_{}",
            corrected_label.to_lowercase().replace(' ', "_")
        ));
        let mut wisdom_concept = Concept::new(
            corrected_id.clone(),
            corrected_label,
            family_explanation,
            ConceptOrigin::EmpiricalTrial,
        );
        wisdom_concept.confidence = 0.55;
        wisdom_concept.affective_valence = 0.60;
        tree.add_concept(wisdom_concept);

        // Link previous concepts to the new wisdom and to humility
        for cid in &involved {
            tree.connect(cid, &corrected_id, RelationType::Causal, 0.75);
        }
        if tree.concepts.contains_key(&ConceptId::from("humility")) {
            tree.connect(
                &"humility".into(),
                &corrected_id,
                RelationType::Associative,
                0.80,
            );
        }

        involved.push(corrected_id);
        state.register_dissonance(0.25);

        // Record Socratic Elenchus reflection on the mistake
        let _ = self.socratic_engine.reflect_on_mistake(
            &mistaken_assumption,
            family_explanation,
            involved.clone(),
        );

        let lesson_id = format!("lesson_{}", self.mistake_lessons.len() + 1);
        let lesson = MistakeLesson::new(
            lesson_id,
            experience_id,
            mistaken_assumption,
            family_explanation,
            involved,
            0.35,
        );
        self.mistake_lessons.push(lesson.clone());
        lesson
    }

    /// Integrates experiential feedback into the Knowledge Tree and cognitive state.
    pub fn apply_feedback(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        experience_id: &str,
        feedback_score: f32,
    ) {
        if let Some(exp) = self
            .episodic_stream
            .iter_mut()
            .find(|e| e.id == experience_id)
        {
            exp.attach_feedback(feedback_score);

            // If feedback is positive, reinforce confidence
            if feedback_score > 0.0 {
                for id in &exp.activated_concepts {
                    if let Some(c) = tree.get_concept_mut(id) {
                        c.apply_experience(0.85, feedback_score);
                    }
                }
                state.emotional_valence = (state.emotional_valence + 0.15).min(1.0);
                state.cognitive_tension = (state.cognitive_tension - 0.1).max(0.05);
            } else {
                // If feedback is negative or critical, gently adapt without crashing
                for id in &exp.activated_concepts {
                    if let Some(c) = tree.get_concept_mut(id) {
                        c.apply_experience(0.20, feedback_score);
                    }
                }
                state.register_dissonance(0.4); // Tension raises motivation to re-evaluate in dreams
            }

            // Create associative bonds between simultaneously activated concepts
            if exp.activated_concepts.len() >= 2 {
                for i in 0..exp.activated_concepts.len() - 1 {
                    let a = &exp.activated_concepts[i];
                    let b = &exp.activated_concepts[i + 1];
                    tree.connect(a, b, RelationType::Associative, 0.5);
                }
            }
        }
    }

    /// Returns un-consolidated episodic memories ready for dream synthesis.
    pub fn unconsolidated_episodes(&self) -> Vec<&EpisodicExperience> {
        self.episodic_stream
            .iter()
            .filter(|e| !e.is_consolidated)
            .collect()
    }

    /// Returns all currently unresolved curiosity questions.
    pub fn pending_questions(&self) -> Vec<&OpenQuestion> {
        self.curiosity_engine.pending_questions()
    }

    /// Returns all Socratic self-reflections generated during learning.
    pub fn socratic_reflections(&self) -> &[SocraticReflection] {
        &self.socratic_engine.reflections
    }
}
