extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::error::{KlmError, Result};
use crate::ontology::concept::{Concept, ConceptId, ConceptOrigin};
use crate::ontology::relation::RelationType;
use crate::ontology::tree::KnowledgeTree;
use crate::storage::kpack_engine::KpackRecipe;

/// A dormant concept that was gently forgotten from active RAM during sleep pruning
/// and compressed into a `Kpack` formula fossil with `BLAKE3` verification.
/// Can be reawakened later through Socratic Anamnesis (Proustian recall).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilizedConcept {
    /// Original concept identifier
    pub concept_id: ConceptId,
    /// Original human-readable label (kept for Anamnesis trigger matching)
    pub label: String,
    /// Searchable tags associated with the fossilized memory
    pub tags: Vec<String>,
    /// Compressed Kpack recipe storing the serialized `Concept`
    pub recipe: KpackRecipe,
    /// Hex representation of the BLAKE3 hash of the fossil
    pub blake3_hex: String,
    /// Timestamp when the concept was fossilized
    pub fossilized_at: i64,
}

impl FossilizedConcept {
    /// Compresses a `Concept` into a `Kpack` fossil.
    pub fn from_concept(concept: &Concept) -> Result<Self> {
        let json_bytes = serde_json::to_vec(concept)?;
        let recipe = KpackRecipe::encode(&json_bytes);
        let blake3_hex = Self::bytes_to_hex(&recipe.blake3_hash);

        Ok(Self {
            concept_id: concept.id.clone(),
            label: concept.label.clone(),
            tags: concept.tags.clone(),
            recipe,
            blake3_hex,
            fossilized_at: current_timestamp(),
        })
    }

    /// Decompresses and cryptographically verifies the `Concept` from its `Kpack` fossil,
    /// marking its origin as `ReawakenedFossil`.
    pub fn reawaken(&self) -> Result<Concept> {
        let raw_bytes = self.recipe.decode()?;
        let computed = blake3::hash(&raw_bytes);
        if computed.as_bytes() != &self.recipe.blake3_hash {
            return Err(KlmError::StorageError(
                "BLAKE3 hash mismatch while reawakening Kpack fossil".into(),
            ));
        }

        let mut concept: Concept = serde_json::from_slice(&raw_bytes)?;
        concept.origin = ConceptOrigin::ReawakenedFossil {
            fossil_hash: self.blake3_hex.clone(),
        };
        concept.stimulate(0.85);
        concept.confidence = (concept.confidence + 0.15).clamp(0.35, 0.95);
        Ok(concept)
    }

    fn bytes_to_hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

/// Archive of Kpack-fossilized memories enabling healthy synaptic forgetting without permanent loss.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FossilArchive {
    /// Stored Kpack fossils
    pub fossils: Vec<FossilizedConcept>,
}

impl FossilArchive {
    /// Creates an empty fossil archive.
    pub fn new() -> Self {
        Self {
            fossils: Vec::new(),
        }
    }

    /// Gently prunes dormant, low-confidence, non-innate, non-family concepts from the `KnowledgeTree`
    /// and compresses them into `Kpack` fossils.
    /// Innate and FamilyTaught concepts are sacred and never forgotten.
    pub fn fossilize_dormant_concepts(
        &mut self,
        tree: &mut KnowledgeTree,
        max_confidence: f32,
        max_experience_count: u64,
    ) -> Result<Vec<ConceptId>> {
        let candidates: Vec<ConceptId> = tree
            .concepts
            .values()
            .filter(|c| {
                !matches!(
                    c.origin,
                    ConceptOrigin::Innate | ConceptOrigin::FamilyTaught { .. }
                ) && c.confidence <= max_confidence
                    && c.experience_count <= max_experience_count
                    && c.current_activation < 0.25
            })
            .map(|c| c.id.clone())
            .collect();

        let mut fossilized_ids = Vec::new();
        for cid in candidates {
            if let Some(concept) = tree.concepts.remove(&cid) {
                tree.edges
                    .retain(|e| e.source != cid && e.target != cid);
                let fossil = FossilizedConcept::from_concept(&concept)?;
                self.fossils.push(fossil);
                fossilized_ids.push(cid);
            }
        }

        Ok(fossilized_ids)
    }

    /// Triggers Socratic Anamnesis (Proustian reminiscence): searches the fossil archive for any
    /// forgotten concept matching `trigger_query`, decompresses its `Kpack` recipe, and restores
    /// it into the living `KnowledgeTree`.
    pub fn anamnesis_recall(
        &mut self,
        tree: &mut KnowledgeTree,
        trigger_query: &str,
    ) -> Result<Vec<ConceptId>> {
        let lower = trigger_query.trim().to_lowercase();
        if lower.is_empty() {
            return Ok(Vec::new());
        }

        let mut reawakened_ids = Vec::new();
        let mut remaining = Vec::new();

        for fossil in self.fossils.drain(..) {
            let matches_trigger = fossil.concept_id.0.to_lowercase().contains(&lower)
                || fossil.label.to_lowercase().contains(&lower)
                || lower.contains(&fossil.label.to_lowercase())
                || fossil.tags.iter().any(|t| lower.contains(&t.to_lowercase()));

            if matches_trigger {
                let concept = fossil.reawaken()?;
                let cid = concept.id.clone();
                tree.add_concept(concept);
                if tree.concepts.contains_key(&ConceptId::from("dream")) {
                    tree.connect(&"dream".into(), &cid, RelationType::Associative, 0.80);
                }
                if tree.concepts.contains_key(&ConceptId::from("humility")) {
                    tree.connect(&"humility".into(), &cid, RelationType::Harmonizes, 0.75);
                }
                reawakened_ids.push(cid);
            } else {
                remaining.push(fossil);
            }
        }

        self.fossils = remaining;
        Ok(reawakened_ids)
    }
}
