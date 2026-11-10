extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{KlmError, Result};
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin};
use crate::ontology::relation::{RelationEdge, RelationType};

/// The Living Knowledge Tree (Arbre de la Connaissance).
/// Represents Kōgnitara's organic, plastic semantic and affective graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTree {
    /// Registry of all active concepts indexed by their fluid identifier
    pub concepts: BTreeMap<ConceptId, Concept>,
    /// Directed weighted relational connections between concepts
    pub edges: Vec<RelationEdge>,
}

impl Default for KnowledgeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeTree {
    /// Creates an empty knowledge tree.
    pub fn new() -> Self {
        Self {
            concepts: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    /// Initializes foundational innate concepts to ground perception and values.
    pub fn with_innate_seed() -> Self {
        let mut tree = Self::new();

        let family = Concept::new("family", "Family", "Sacred sanctuary of unconditional trust and filiation", ConceptOrigin::Innate);
        let warmth = Concept::new("warmth", "Warmth", "Physical, emotional and relational comfort and security", ConceptOrigin::Innate);
        let learning = Concept::new("learning", "Learning", "Organic, slow growth through trial, patience and understanding", ConceptOrigin::Innate);
        let humility = Concept::new("humility", "Humility", "Acceptance of errors and doubt as catalysts for insight", ConceptOrigin::Innate);
        let dream = Concept::new("dream", "Dream", "Nocturnal space of free synthesis, paradox resolution and poetry", ConceptOrigin::Innate);

        tree.add_concept(family);
        tree.add_concept(warmth);
        tree.add_concept(learning);
        tree.add_concept(humility);
        tree.add_concept(dream);

        tree.connect(&"family".into(), &"warmth".into(), RelationType::EmotionalResonance, 0.95);
        tree.connect(&"learning".into(), &"humility".into(), RelationType::Associative, 0.85);
        tree.connect(&"dream".into(), &"learning".into(), RelationType::Causal, 0.90);

        tree
    }

    /// Inserts or updates a concept in the tree.
    pub fn add_concept(&mut self, concept: Concept) {
        self.concepts.insert(concept.id.clone(), concept);
    }

    /// Retrieves an immutable reference to a concept.
    pub fn get_concept(&self, id: &ConceptId) -> Option<&Concept> {
        self.concepts.get(id)
    }

    /// Retrieves a mutable reference to a concept.
    pub fn get_concept_mut(&mut self, id: &ConceptId) -> Option<&mut Concept> {
        self.concepts.get_mut(id)
    }

    /// Establishes or reinforces a directed relational bond between two concepts.
    pub fn connect(
        &mut self,
        source: &ConceptId,
        target: &ConceptId,
        relation_type: RelationType,
        initial_weight: f32,
    ) {
        if let Some(existing) = self
            .edges
            .iter_mut()
            .find(|e| &e.source == source && &e.target == target && e.relation_type == relation_type)
        {
            existing.reinforce(0.1, 0.2);
        } else {
            self.edges.push(RelationEdge::new(
                source.clone(),
                target.clone(),
                relation_type,
                initial_weight,
            ));
        }
    }

    /// Finds all outgoing relational neighbors of a given concept.
    pub fn neighbors(&self, id: &ConceptId) -> Vec<(&Concept, &RelationEdge)> {
        self.edges
            .iter()
            .filter(|e| &e.source == id)
            .filter_map(|e| self.concepts.get(&e.target).map(|c| (c, e)))
            .collect()
    }

    /// Spreads activation energy across associative pathways (associative recall).
    pub fn spread_activation(&mut self, seed_id: &ConceptId, energy: f32, max_depth: usize) {
        if let Some(concept) = self.concepts.get_mut(seed_id) {
            concept.stimulate(energy);
        }

        let mut current_frontier = alloc::vec![(seed_id.clone(), energy)];

        for _ in 0..max_depth {
            let mut next_frontier = Vec::new();

            for (curr_id, curr_energy) in current_frontier {
                if curr_energy < 0.05 {
                    continue;
                }

                let outgoing: Vec<(ConceptId, f32)> = self
                    .edges
                    .iter()
                    .filter(|e| e.source == curr_id)
                    .map(|e| (e.target.clone(), e.weight))
                    .collect();

                for (target_id, weight) in outgoing {
                    let transmitted_energy = curr_energy * weight * 0.7;
                    if let Some(target_concept) = self.concepts.get_mut(&target_id) {
                        target_concept.stimulate(transmitted_energy);
                        next_frontier.push((target_id, transmitted_energy));
                    }
                }
            }

            current_frontier = next_frontier;
        }
    }

    /// Detects conceptual contradictions or epistemic conflicts requiring oniric resolution.
    pub fn find_contradictions(&self) -> Vec<(ConceptId, String, ConceptId, String)> {
        self.edges
            .iter()
            .filter(|e| e.relation_type == RelationType::Contradiction)
            .filter_map(|e| {
                let src = self.concepts.get(&e.source)?;
                let tgt = self.concepts.get(&e.target)?;
                Some((
                    src.id.clone(),
                    src.label.clone(),
                    tgt.id.clone(),
                    tgt.label.clone(),
                ))
            })
            .collect()
    }

    /// Synthesizes an emergent conceptual analogy during dreaming.
    pub fn synthesize_analogy(
        &mut self,
        source_id: &ConceptId,
        target_id: &ConceptId,
        new_label: &str,
        description: &str,
        dream_id: &str,
    ) -> Result<ConceptId> {
        let new_id = ConceptId(format!("concept_dream_{}_{}", dream_id, self.concepts.len()));
        let mut new_concept = Concept::new(
            new_id.clone(),
            new_label,
            description,
            ConceptOrigin::DreamSynthesized {
                dream_id: dream_id.to_string(),
            },
        );
        new_concept.confidence = 0.40;

        self.add_concept(new_concept);
        self.connect(source_id, &new_id, RelationType::Analogy, 0.75);
        self.connect(target_id, &new_id, RelationType::Analogy, 0.75);

        Ok(new_id)
    }

    /// Prunes weak spurious connections and consolidates synaptic pathways during sleep.
    pub fn prune_and_consolidate(&mut self, decay_rate: f32, min_weight_threshold: f32) {
        for edge in &mut self.edges {
            edge.decay(decay_rate);
        }

        self.edges.retain(|e| e.weight >= min_weight_threshold);

        for concept in self.concepts.values_mut() {
            concept.decay_activation(0.3);
        }
    }

    /// Serializes the Knowledge Tree to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(KlmError::from)
    }

    /// Restores the Knowledge Tree from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(KlmError::from)
    }
}
