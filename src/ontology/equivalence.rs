extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::error::{KlmError, Result};
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceChange {
    pub active: bool,
    pub reason: String,
    pub changed_at: i64,
}

/// A possible equivalence, separate from graph edges and never a verified identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PossibleEquivalence {
    pub id: u64,
    pub left: ConceptId,
    pub right: ConceptId,
    pub left_label: String,
    pub right_label: String,
    pub left_created_at: i64,
    pub right_created_at: i64,
    pub source: String,
    pub reason: String,
    pub created_at: i64,
    pub changes: Vec<EquivalenceChange>,
}

impl PossibleEquivalence {
    pub fn is_active(&self) -> bool {
        self.changes.last().is_none_or(|change| change.active)
    }

    pub fn endpoints_match(&self, tree: &KnowledgeTree) -> bool {
        tree.concepts.get(&self.left).is_some_and(|concept| {
            concept.label == self.left_label && concept.created_at == self.left_created_at
        }) && tree.concepts.get(&self.right).is_some_and(|concept| {
            concept.label == self.right_label && concept.created_at == self.right_created_at
        })
    }

    pub fn display(&self) -> String {
        format!(
            "#{} [{}] {} ({}) <-> {} ({}); source: {}; reason: {}",
            self.id,
            if self.is_active() {
                "possible, unverified"
            } else {
                "withdrawn"
            },
            self.left_label,
            self.left,
            self.right_label,
            self.right,
            self.source,
            self.reason,
        )
    }
}

impl KnowledgeTree {
    /// Record a candidate without adding graph edges or combining either concept.
    pub fn propose_equivalence(
        &mut self,
        left: &ConceptId,
        right: &ConceptId,
        source: &str,
        reason: &str,
    ) -> Result<u64> {
        if left == right || source.trim().is_empty() || reason.trim().is_empty() {
            return Err(KlmError::OntologyError(
                "Two different concepts, a source and a reason are required.".into(),
            ));
        }
        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        let left_concept = self.concepts.get(left).ok_or_else(|| {
            KlmError::OntologyError(format!("Concept '{}' not found in this brain.", left))
        })?;
        let right_concept = self.concepts.get(right).ok_or_else(|| {
            KlmError::OntologyError(format!("Concept '{}' not found in this brain.", right))
        })?;
        if self
            .possible_equivalences
            .iter()
            .any(|link| &link.left == left && &link.right == right)
        {
            return Err(KlmError::OntologyError(
                "A proposal for this pair already exists, including withdrawn proposals.".into(),
            ));
        }
        let id = self
            .possible_equivalences
            .iter()
            .map(|link| link.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| KlmError::OntologyError("Equivalence IDs exhausted.".into()))?;
        self.possible_equivalences.push(PossibleEquivalence {
            id,
            left: left.clone(),
            right: right.clone(),
            left_label: left_concept.label.clone(),
            right_label: right_concept.label.clone(),
            left_created_at: left_concept.created_at,
            right_created_at: right_concept.created_at,
            source: source.trim().to_string(),
            reason: reason.trim().to_string(),
            created_at: current_timestamp(),
            changes: Vec::new(),
        });
        Ok(id)
    }

    /// Find same-label candidates for one concept. Matching names alone is not evidence of identity.
    pub fn suggest_equivalences(&mut self, concept_id: &ConceptId) -> Result<usize> {
        fn normalized_label(label: &str) -> String {
            label
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        }
        let concept = self.concepts.get(concept_id).ok_or_else(|| {
            KlmError::OntologyError(format!("Concept '{}' not found in this brain.", concept_id))
        })?;
        let label = normalized_label(&concept.label);
        if label.is_empty() {
            return Err(KlmError::OntologyError("Concept label is empty.".into()));
        }
        let candidates: Vec<ConceptId> = self
            .concepts
            .values()
            .filter(|candidate| {
                &candidate.id != concept_id
                    && normalized_label(&candidate.label) == label
                    && !self.possible_equivalences.iter().any(|link| {
                        (&link.left == concept_id && link.right == candidate.id)
                            || (&link.right == concept_id && link.left == candidate.id)
                    })
            })
            .map(|candidate| candidate.id.clone())
            .collect();
        let count = u64::try_from(candidates.len())
            .map_err(|_| KlmError::OntologyError("Too many equivalence candidates.".into()))?;
        self.possible_equivalences
            .iter()
            .map(|link| link.id)
            .max()
            .unwrap_or(0)
            .checked_add(count)
            .ok_or_else(|| KlmError::OntologyError("Equivalence IDs exhausted.".into()))?;
        for candidate in &candidates {
            self.propose_equivalence(
                concept_id,
                candidate,
                "Local concept labels",
                "Labels match ignoring case and whitespace; this does not establish identity.",
            )?;
        }
        Ok(candidates.len())
    }

    /// Withdraw or reopen a proposal, retaining the reason and prior changes.
    pub fn set_equivalence_active(&mut self, id: u64, active: bool, reason: &str) -> Result<()> {
        if reason.trim().is_empty() {
            return Err(KlmError::OntologyError("A reason is required.".into()));
        }
        let index = self
            .possible_equivalences
            .iter()
            .position(|link| link.id == id)
            .ok_or_else(|| KlmError::OntologyError(format!("Equivalence #{id} not found.")))?;
        let link = &self.possible_equivalences[index];
        if link.is_active() == active {
            return Err(KlmError::OntologyError(
                "Proposal already has that status.".into(),
            ));
        }
        if active && !link.endpoints_match(self) {
            return Err(KlmError::OntologyError(
                "Cannot reopen: a concept is absent or has changed since the proposal.".into(),
            ));
        }
        self.possible_equivalences[index]
            .changes
            .push(EquivalenceChange {
                active,
                reason: reason.trim().to_string(),
                changed_at: current_timestamp(),
            });
        Ok(())
    }
}
