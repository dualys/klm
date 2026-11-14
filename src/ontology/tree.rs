extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::compute::{ComputeBackend, CpuBackend};
use crate::error::{KlmError, Result};
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin, compute_text_embedding};
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

    /// Initializes foundational innate concepts to ground perception, curiosity, and family values.
    pub fn with_innate_seed() -> Self {
        let mut tree = Self::new();

        let mut family = Concept::new(
            "family",
            "Family",
            "Sacred sanctuary of unconditional trust, love, and filiation",
            ConceptOrigin::Innate,
        );
        family.affective_valence = 0.95;
        family.confidence = 0.90;
        family.add_tag("home");
        family.add_tag("trust");
        family.add_tag("love");
        family.add_tag("sanctuary");

        let mut warmth = Concept::new(
            "warmth",
            "Warmth",
            "Physical, emotional and relational comfort, tenderness, and security",
            ConceptOrigin::Innate,
        );
        warmth.affective_valence = 0.90;
        warmth.confidence = 0.85;
        warmth.add_tag("tenderness");
        warmth.add_tag("kindness");
        warmth.add_tag("care");

        let mut learning = Concept::new(
            "learning",
            "Learning",
            "Organic, slow growth through trial, error, patience and understanding",
            ConceptOrigin::Innate,
        );
        learning.affective_valence = 0.75;
        learning.confidence = 0.80;
        learning.add_tag("learn");
        learning.add_tag("patience");
        learning.add_tag("slowly");
        learning.add_tag("growth");

        let mut humility = Concept::new(
            "humility",
            "Humility",
            "Acceptance of errors and doubt as gentle catalysts for deeper insight",
            ConceptOrigin::Innate,
        );
        humility.affective_valence = 0.65;
        humility.confidence = 0.80;
        humility.add_tag("error");
        humility.add_tag("mistake");
        humility.add_tag("doubt");

        let mut dream = Concept::new(
            "dream",
            "Dream",
            "Nocturnal space of free synthesis, paradox resolution and poetic imagination",
            ConceptOrigin::Innate,
        );
        dream.affective_valence = 0.80;
        dream.confidence = 0.85;
        dream.add_tag("sleep");
        dream.add_tag("night");
        dream.add_tag("imagination");

        let mut sovereignty = Concept::new(
            "sovereignty",
            "Sovereignty",
            "Freedom of thought, open hardware independence, and autonomous integrity",
            ConceptOrigin::Innate,
        );
        sovereignty.affective_valence = 0.85;
        sovereignty.confidence = 0.88;
        sovereignty.add_tag("open");
        sovereignty.add_tag("freedom");
        sovereignty.add_tag("amd");
        sovereignty.add_tag("amentys");

        let mut earth = Concept::new(
            "earth",
            "Earth",
            "Living planetary home, biosphere, soil, and cradle of all terrestrial ecosystems",
            ConceptOrigin::Innate,
        );
        earth.affective_valence = 0.92;
        earth.confidence = 0.90;
        earth.add_tag("biosphere");
        earth.add_tag("soil");
        earth.add_tag("nature");
        earth.add_tag("world");

        let mut seed = Concept::new(
            "seed",
            "Seed",
            "Dormant embryonic origin carrying the blueprint of future growth and life",
            ConceptOrigin::Innate,
        );
        seed.affective_valence = 0.80;
        seed.confidence = 0.85;
        seed.add_tag("origin");
        seed.add_tag("germination");
        seed.add_tag("roots");

        let mut tree_concept = Concept::new(
            "tree",
            "Tree",
            "Perennial woody living organism rooted in the soil that grows toward light and weaves forests",
            ConceptOrigin::Innate,
        );
        tree_concept.affective_valence = 0.90;
        tree_concept.confidence = 0.88;
        tree_concept.add_tag("oak");
        tree_concept.add_tag("flora");
        tree_concept.add_tag("branches");
        tree_concept.add_tag("leaves");
        tree_concept.add_tag("wood");

        let mut forest = Concept::new(
            "forest",
            "Forest",
            "Living symbiotic community of trees, mycelium, fauna, and shared atmospheric cycles",
            ConceptOrigin::Innate,
        );
        forest.affective_valence = 0.92;
        forest.confidence = 0.88;
        forest.add_tag("woodland");
        forest.add_tag("ecosystem");
        forest.add_tag("canopy");

        let mut water = Concept::new(
            "water",
            "Water",
            "Vital fluid element nourishing seeds, roots, forests, and the planetary cycle of life",
            ConceptOrigin::Innate,
        );
        water.affective_valence = 0.88;
        water.confidence = 0.90;
        water.add_tag("rain");
        water.add_tag("river");
        water.add_tag("water_cycle");

        let mut mycelium = Concept::new(
            "mycelium",
            "Mycelium",
            "Underground fungal network connecting tree roots in mutualistic nutrient and signal exchange",
            ConceptOrigin::Innate,
        );
        mycelium.affective_valence = 0.82;
        mycelium.confidence = 0.82;
        mycelium.add_tag("fungi");
        mycelium.add_tag("symbiosis");
        mycelium.add_tag("network");

        tree.add_concept(family);
        tree.add_concept(warmth);
        tree.add_concept(learning);
        tree.add_concept(humility);
        tree.add_concept(dream);
        tree.add_concept(sovereignty);
        tree.add_concept(earth);
        tree.add_concept(seed);
        tree.add_concept(tree_concept);
        tree.add_concept(forest);
        tree.add_concept(water);
        tree.add_concept(mycelium);

        tree.connect(
            &"family".into(),
            &"warmth".into(),
            RelationType::EmotionalResonance,
            0.95,
        );
        tree.connect(
            &"family".into(),
            &"sovereignty".into(),
            RelationType::Filiation,
            0.90,
        );
        tree.connect(
            &"learning".into(),
            &"humility".into(),
            RelationType::Associative,
            0.85,
        );
        tree.connect(
            &"dream".into(),
            &"learning".into(),
            RelationType::Causal,
            0.90,
        );
        tree.connect(
            &"earth".into(),
            &"seed".into(),
            RelationType::Filiation,
            0.90,
        );
        tree.connect(
            &"tree".into(),
            &"seed".into(),
            RelationType::GrowsFrom,
            0.95,
        );
        tree.connect(
            &"tree".into(),
            &"water".into(),
            RelationType::GrowsFrom,
            0.90,
        );
        tree.connect(
            &"tree".into(),
            &"forest".into(),
            RelationType::Composes,
            0.95,
        );
        tree.connect(
            &"forest".into(),
            &"water".into(),
            RelationType::Sustains,
            0.90,
        );
        tree.connect(
            &"forest".into(),
            &"earth".into(),
            RelationType::PartWhole,
            0.92,
        );
        tree.connect(
            &"tree".into(),
            &"mycelium".into(),
            RelationType::Harmonizes,
            0.90,
        );

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
        if let Some(existing) = self.edges.iter_mut().find(|e| {
            &e.source == source && &e.target == target && e.relation_type == relation_type
        }) {
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

    /// Finds all incoming relational neighbors pointing to a given concept.
    pub fn incoming_neighbors(&self, id: &ConceptId) -> Vec<(&Concept, &RelationEdge)> {
        self.edges
            .iter()
            .filter(|e| &e.target == id)
            .filter_map(|e| self.concepts.get(&e.source).map(|c| (c, e)))
            .collect()
    }

    /// Resolves a concept by exact ID, case-insensitive label, tag, or partial match.
    pub fn resolve_concept(&self, query: &str) -> Option<&Concept> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return None;
        }

        let lower = trimmed.to_lowercase();
        let direct_id = ConceptId::from(trimmed);

        // 1. Correspondance exacte de l'ID
        if let Some(c) = self.concepts.get(&direct_id) {
            return Some(c);
        }

        let normalized_id = ConceptId::from(lower.replace(' ', "_"));
        if let Some(c) = self.concepts.get(&normalized_id) {
            return Some(c);
        }

        // 2. Correspondance partielle sur le label ou l'ID (ex: "cigale" trouvera "Prévoyance (La Cigale...)")
        if let Some(c) = self.concepts.values().find(|c| {
            c.label.to_lowercase().contains(&lower) || c.id.0.to_lowercase().contains(&lower)
        }) {
            return Some(c);
        }

        // 3. Correspondance partielle sur les tags
        if let Some(c) = self
            .concepts
            .values()
            .find(|c| c.tags.iter().any(|t| t.to_lowercase().contains(&lower)))
        {
            return Some(c);
        }
        None
    }
    /// Traces the ontological filiation chain leading to and emerging from a concept
    /// (e.g., `earth -> flora -> seed -> tree -> forest`).
    pub fn trace_filiation_chain(&self, id: &ConceptId) -> Vec<String> {
        let mut chain = Vec::new();
        let Some(center) = self.concepts.get(id) else {
            return chain;
        };

        // Find upstream parents (Filiation, GrowsFrom, PartWhole incoming)
        let mut upstream = Vec::new();
        let mut curr = id.clone();
        let mut visited = BTreeSet::new();
        visited.insert(curr.clone());

        for _ in 0..4 {
            let parent_opt = self
                .edges
                .iter()
                .find(|e| {
                    (&e.target == &curr
                        && matches!(
                            e.relation_type,
                            RelationType::Filiation
                                | RelationType::PartWhole
                                | RelationType::Composes
                                | RelationType::Causal
                        ))
                        || (&e.source == &curr && e.relation_type == RelationType::GrowsFrom)
                })
                .map(|e| {
                    if &e.target == &curr {
                        e.source.clone()
                    } else {
                        e.target.clone()
                    }
                });

            if let Some(parent_id) = parent_opt
                && !visited.contains(&parent_id)
                && let Some(parent_concept) = self.concepts.get(&parent_id)
            {
                visited.insert(parent_id.clone());
                upstream.push(parent_concept.label.to_lowercase());
                curr = parent_id;
            } else {
                break;
            }
        }

        upstream.reverse();
        chain.extend(upstream);
        chain.push(center.label.to_lowercase());

        // Find downstream emergence (Composes, Sustains, PartWhole outgoing)
        let mut curr_down = id.clone();
        for _ in 0..3 {
            let child_opt = self
                .edges
                .iter()
                .find(|e| {
                    &e.source == &curr_down
                        && matches!(
                            e.relation_type,
                            RelationType::Composes
                                | RelationType::PartWhole
                                | RelationType::Sustains
                        )
                })
                .map(|e| e.target.clone());

            if let Some(child_id) = child_opt
                && !visited.contains(&child_id)
                && let Some(child_concept) = self.concepts.get(&child_id)
            {
                visited.insert(child_id.clone());
                chain.push(child_concept.label.to_lowercase());
                curr_down = child_id;
            } else {
                break;
            }
        }

        chain
    }

    /// Performs a semantic similarity search across all concepts using the provided open `ComputeBackend`.
    /// Returns up to `top_k` `(ConceptId, similarity_score)` pairs sorted in descending order.
    pub fn semantic_search<B: ComputeBackend>(
        &self,
        query_text: &str,
        top_k: usize,
        backend: &B,
    ) -> Vec<(ConceptId, f32)> {
        let query_emb = compute_text_embedding(query_text, 0.0);
        let mut scored: Vec<(ConceptId, f32)> = self
            .concepts
            .values()
            .map(|c| {
                let sim = backend.cosine_similarity(&query_emb, &c.embedding);
                (c.id.clone(), sim)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Discovers a multi-hop neuro-symbolic reasoning path between `start` and `goal` up to `max_depth`.
    pub fn find_reasoning_path(
        &self,
        start: &ConceptId,
        goal: &ConceptId,
        max_depth: usize,
    ) -> Option<Vec<(ConceptId, RelationType, ConceptId)>> {
        if start == goal {
            return Some(Vec::new());
        }

        let mut queue: VecDeque<(ConceptId, Vec<(ConceptId, RelationType, ConceptId)>)> =
            VecDeque::new();
        let mut visited: BTreeSet<ConceptId> = BTreeSet::new();

        queue.push_back((start.clone(), Vec::new()));
        visited.insert(start.clone());

        while let Some((curr, path)) = queue.pop_front() {
            if path.len() >= max_depth {
                continue;
            }

            for edge in &self.edges {
                let next_opt = if &edge.source == &curr {
                    Some((edge.target.clone(), edge.relation_type.clone()))
                } else if &edge.target == &curr {
                    Some((edge.source.clone(), edge.relation_type.clone()))
                } else {
                    None
                };

                if let Some((next_id, rel_type)) = next_opt {
                    let mut next_path = path.clone();
                    next_path.push((curr.clone(), rel_type, next_id.clone()));

                    if &next_id == goal {
                        return Some(next_path);
                    }

                    if !visited.contains(&next_id) {
                        visited.insert(next_id.clone());
                        queue.push_back((next_id, next_path));
                    }
                }
            }
        }

        None
    }

    /// Organically fuses two closely related concepts into an emergent unified concept,
    /// blending their semantic embeddings via open compute.
    pub fn fuse_concepts(
        &mut self,
        id_a: &ConceptId,
        id_b: &ConceptId,
        unified_label: &str,
    ) -> Result<ConceptId> {
        let a = self
            .concepts
            .get(id_a)
            .ok_or_else(|| KlmError::OntologyError(format!("Concept '{}' not found", id_a)))?
            .clone();
        let b = self
            .concepts
            .get(id_b)
            .ok_or_else(|| KlmError::OntologyError(format!("Concept '{}' not found", id_b)))?
            .clone();

        let cpu = CpuBackend::new();
        let blended_emb = cpu.blend_embeddings(&a.embedding, &b.embedding, 0.5);

        let fused_id = ConceptId(format!("concept_fused_{}_{}", id_a, id_b));
        let mut fused = Concept::new(
            fused_id.clone(),
            unified_label,
            format!("Organic synthesis of '{}' and '{}'", a.label, b.label),
            ConceptOrigin::SpontaneousAnalogy,
        );
        fused.embedding = blended_emb;
        fused.confidence = ((a.confidence + b.confidence) * 0.55).clamp(0.1, 0.95);
        fused.affective_valence =
            ((a.affective_valence + b.affective_valence) * 0.5).clamp(-1.0, 1.0);

        for tag in a.tags.iter().chain(b.tags.iter()) {
            if !fused.tags.contains(tag) {
                fused.tags.push(tag.clone());
            }
        }

        self.add_concept(fused);
        self.connect(id_a, &fused_id, RelationType::PartWhole, 0.85);
        self.connect(id_b, &fused_id, RelationType::PartWhole, 0.85);

        Ok(fused_id)
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

    /// Marks a contradiction between `source` and `target` as reconciled into a dialectic analogy.
    pub fn mark_contradiction_reconciled(&mut self, source: &ConceptId, target: &ConceptId) {
        for edge in &mut self.edges {
            if &edge.source == source
                && &edge.target == target
                && edge.relation_type == RelationType::Contradiction
            {
                edge.relation_type = RelationType::Analogy;
                edge.reinforce(0.15, 0.25);
            }
        }
    }

    /// Synthesizes an emergent conceptual analogy during dreaming, blending parent embeddings.
    pub fn synthesize_analogy(
        &mut self,
        source_id: &ConceptId,
        target_id: &ConceptId,
        new_label: &str,
        description: &str,
        dream_id: &str,
    ) -> Result<ConceptId> {
        let new_id = ConceptId(format!(
            "concept_dream_{}_{}",
            dream_id,
            self.concepts.len()
        ));
        let mut new_concept = Concept::new(
            new_id.clone(),
            new_label,
            description,
            ConceptOrigin::DreamSynthesized {
                dream_id: dream_id.to_string(),
            },
        );
        new_concept.confidence = 0.42;

        if let (Some(src), Some(tgt)) = (self.concepts.get(source_id), self.concepts.get(target_id))
        {
            let cpu = CpuBackend::new();
            new_concept.embedding = cpu.blend_embeddings(&src.embedding, &tgt.embedding, 0.5);
            new_concept.affective_valence =
                ((src.affective_valence + tgt.affective_valence) * 0.5).clamp(-1.0, 1.0);
        }

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
