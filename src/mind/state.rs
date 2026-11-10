extern crate alloc;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::ontology::concept::ConceptId;

/// The living cognitive state of Kōgnitara.
/// Reflects epistemic tension, curiosity, emotional valence, and mental fatigue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    /// Active concepts currently held in short-term working memory
    pub working_memory: Vec<ConceptId>,
    /// Epistemic tension / doubt (0.0 = calm certainty, 1.0 = deep perplexity requiring dreaming)
    pub cognitive_tension: f32,
    /// Intrinsic motivation to explore, question, and discover
    pub curiosity: f32,
    /// Mental fatigue accumulating during wakefulness (0.0 = refreshed, 1.0 = exhaustion)
    pub fatigue: f32,
    /// Global affective mood / emotional valence (-1.0 = troubled, +1.0 = serene/joyful)
    pub emotional_valence: f32,
    /// Available attentional focus energy
    pub attention_energy: f32,
    /// Base organic learning rate (slow and grounded, preventing catastrophic forgetting)
    pub learning_rate: f32,
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveState {
    /// Initializes a refreshed, curious cognitive state.
    pub fn new() -> Self {
        Self {
            working_memory: Vec::new(),
            cognitive_tension: 0.15,
            curiosity: 0.85,
            fatigue: 0.05,
            emotional_valence: 0.30,
            attention_energy: 1.0,
            learning_rate: 0.06, // Human-like slow integration rate
        }
    }

    /// Focuses attention on a concept, cycling working memory capacity.
    pub fn focus_on(&mut self, concept_id: ConceptId, max_capacity: usize) {
        if !self.working_memory.contains(&concept_id) {
            if self.working_memory.len() >= max_capacity {
                self.working_memory.remove(0);
            }
            self.working_memory.push(concept_id);
        }
        self.fatigue = (self.fatigue + 0.02).min(1.0);
    }

    /// Records an encounter with an error or contradiction.
    /// In Kōgnitara, errors are not system crashes; they stimulate curiosity and tension.
    pub fn register_dissonance(&mut self, severity: f32) {
        self.cognitive_tension = (self.cognitive_tension + severity * 0.3).min(1.0);
        self.curiosity = (self.curiosity + severity * 0.2).min(1.0);
        self.emotional_valence = (self.emotional_valence - severity * 0.1).max(-1.0);
    }

    /// Relieves cognitive tension and restores energy through restorative sleep or clarity.
    pub fn rest(&mut self) {
        self.fatigue = (self.fatigue * 0.1).max(0.0);
        self.cognitive_tension = (self.cognitive_tension * 0.3).max(0.05);
        self.attention_energy = 1.0;
        self.emotional_valence = (self.emotional_valence + 0.2).min(1.0);
        self.working_memory.clear();
    }
}
