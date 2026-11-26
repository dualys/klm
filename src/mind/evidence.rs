extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::error::{KlmError, Result};
use crate::ontology::concept::ConceptId;

/// How a source relates to an individual version of a hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceRelation {
    Reports,
    Supports,
    Contradicts,
}

/// A source and its reported relationship to a hypothesis; not a verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub revision: usize,
    pub relation: EvidenceRelation,
    pub source: String,
    pub note: String,
    pub observed_at: i64,
}

/// A correction preserves the previous statement and the reason it changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisCorrection {
    pub previous_statement: String,
    pub reason: String,
    pub corrected_at: i64,
}

/// An exploratory statement, not an established fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: u64,
    pub statement: String,
    pub related_concepts: Vec<ConceptId>,
    pub evidence: Vec<Evidence>,
    pub corrections: Vec<HypothesisCorrection>,
    pub created_at: i64,
}

impl Hypothesis {
    /// Evidence refers to the statement at this revision, not later corrections.
    pub fn revision(&self) -> usize {
        self.corrections.len()
    }

    pub fn add_evidence(
        &mut self,
        relation: EvidenceRelation,
        source: &str,
        note: &str,
        observed_at: i64,
    ) -> Result<()> {
        if source.trim().is_empty() || note.trim().is_empty() {
            return Err(KlmError::CognitiveError(
                "Evidence requires a source and a note.".into(),
            ));
        }
        self.evidence.push(Evidence {
            revision: self.revision(),
            relation,
            source: source.trim().to_string(),
            note: note.trim().to_string(),
            observed_at,
        });
        Ok(())
    }

    pub fn correct(&mut self, statement: &str, reason: &str) -> Result<()> {
        let statement = statement.trim();
        let reason = reason.trim();
        if statement.is_empty() || reason.is_empty() {
            return Err(KlmError::CognitiveError(
                "A correction requires a new statement and a reason.".into(),
            ));
        }
        if statement == self.statement {
            return Err(KlmError::CognitiveError(
                "The corrected statement must be different.".into(),
            ));
        }
        self.corrections.push(HypothesisCorrection {
            previous_statement: core::mem::replace(&mut self.statement, statement.to_string()),
            reason: reason.to_string(),
            corrected_at: current_timestamp(),
        });
        Ok(())
    }
}

/// Persistent record of exploratory statements and their sources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypothesisLedger {
    pub hypotheses: Vec<Hypothesis>,
}

impl HypothesisLedger {
    pub fn add(&mut self, statement: &str, related_concepts: Vec<ConceptId>) -> Result<u64> {
        let statement = statement.trim();
        if statement.is_empty() {
            return Err(KlmError::CognitiveError(
                "A hypothesis requires a statement.".into(),
            ));
        }
        let id = self
            .hypotheses
            .iter()
            .map(|hypothesis| hypothesis.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| KlmError::CognitiveError("Hypothesis IDs exhausted.".into()))?;
        self.hypotheses.push(Hypothesis {
            id,
            statement: statement.to_string(),
            related_concepts,
            evidence: Vec::new(),
            corrections: Vec::new(),
            created_at: current_timestamp(),
        });
        Ok(id)
    }

    pub fn get_mut(&mut self, id: u64) -> Result<&mut Hypothesis> {
        self.hypotheses
            .iter_mut()
            .find(|hypothesis| hypothesis.id == id)
            .ok_or_else(|| KlmError::CognitiveError(alloc::format!("Hypothesis {id} not found.")))
    }
}
