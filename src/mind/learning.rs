extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::mind::experience::EpisodicExperience;
use crate::mind::state::CognitiveState;
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin};
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;

/// The organic trial-and-error learning engine for Kōgnitara.
/// Replaces brittle over-parameterized statistical weights with plastic, grounded conceptual reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganicLearner {
    /// Episodic stream of recent experiences
    pub episodic_stream: Vec<EpisodicExperience>,
    /// Sequential counter for experience IDs
    pub next_experience_index: u64,
}

impl Default for OrganicLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl OrganicLearner {
    /// Creates a fresh organic learning engine.
    pub fn new() -> Self {
        Self {
            episodic_stream: Vec::new(),
            next_experience_index: 1,
        }
    }

    /// Processes an input stimulus, associates related concepts, and forms an intuitive response.
    pub fn process_stimulus(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        stimulus: &str,
        relational_affinity: f32,
    ) -> (String, Vec<ConceptId>) {
        let lower = stimulus.to_lowercase();
        let mut activated: Vec<ConceptId> = Vec::new();

        // 1. Direct lexical & semantic resonance
        for (id, concept) in &tree.concepts {
            if lower.contains(&concept.label.to_lowercase())
                || concept.tags.iter().any(|t| lower.contains(&t.to_lowercase()))
            {
                activated.push(id.clone());
            }
        }

        // If no direct concept matched, create a nascent plastic concept
        if activated.is_empty() {
            let label_sample: String = stimulus.chars().take(20).collect();
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
            state.register_dissonance(0.2); // Mild curiosity regarding novelty
        }

        // 2. Spread activation through the Knowledge Tree
        for concept_id in &activated {
            tree.spread_activation(concept_id, 0.8, 2);
            state.focus_on(concept_id.clone(), 7);
        }

        // 3. Formulate empathetic response based on active concepts
        let mut response_fragments: Vec<String> = Vec::new();
        for id in &activated {
            if let Some(c) = tree.get_concept(id) {
                if c.confidence > 0.6 {
                    response_fragments.push(format!("I feel strongly about {}.", c.label));
                } else {
                    response_fragments.push(format!("I am still learning to understand {}.", c.label));
                }
            }
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

    /// Integrates experiential feedback into the Knowledge Tree and cognitive state.
    pub fn apply_feedback(
        &mut self,
        tree: &mut KnowledgeTree,
        state: &mut CognitiveState,
        experience_id: &str,
        feedback_score: f32,
    ) {
        if let Some(exp) = self.episodic_stream.iter_mut().find(|e| e.id == experience_id) {
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
}
