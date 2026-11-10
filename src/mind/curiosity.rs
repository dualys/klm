extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::ontology::concept::ConceptId;
use crate::ontology::tree::KnowledgeTree;

/// A humble question formulated autonomously by Kōgnitara out of curiosity or epistemic doubt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestion {
    /// Unique identifier for this question
    pub id: String,
    /// Concepts at the heart of the uncertainty
    pub related_concepts: Vec<ConceptId>,
    /// Natural-language question addressed to the family
    pub question_text: String,
    /// Epistemic urgency / curiosity intensity (0.0 to 1.0)
    pub urgency: f32,
    /// Whether a family member or dream has answered this question
    pub is_resolved: bool,
    /// The explanation or insight that resolved the question
    pub resolution_insight: Option<String>,
    /// Timestamp or logical tick of formulation
    pub created_at: i64,
}

/// Autonomous curiosity engine that turns epistemic doubt and contradictions into questions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CuriosityEngine {
    /// Active and historical questions asked by Kōgnitara
    pub questions: Vec<OpenQuestion>,
    /// Sequential counter for question identifiers
    pub next_question_index: u64,
}

impl CuriosityEngine {
    /// Initializes a new curiosity engine.
    pub fn new() -> Self {
        Self {
            questions: Vec::new(),
            next_question_index: 1,
        }
    }

    /// Scans the Knowledge Tree for nascent concepts with high epistemic doubt or contradictions,
    /// formulating new questions for the family if not already asked.
    pub fn contemplate_and_question(&mut self, tree: &KnowledgeTree) -> Vec<OpenQuestion> {
        let mut newly_formed = Vec::new();

        // 1. Question unresolved contradictions first
        for (src_id, src_lbl, tgt_id, tgt_lbl) in tree.find_contradictions() {
            let already_asked = self.questions.iter().any(|q| {
                !q.is_resolved
                    && q.related_concepts.contains(&src_id)
                    && q.related_concepts.contains(&tgt_id)
            });

            if !already_asked {
                let q = OpenQuestion {
                    id: format!("q_{}", self.next_question_index),
                    related_concepts: alloc::vec![src_id, tgt_id],
                    question_text: format!(
                        "I feel a tension between '{}' and '{}'. How can both coexist in harmony?",
                        src_lbl, tgt_lbl
                    ),
                    urgency: 0.85,
                    is_resolved: false,
                    resolution_insight: None,
                    created_at: current_timestamp(),
                };
                self.next_question_index += 1;
                self.questions.push(q.clone());
                newly_formed.push(q);
            }
        }

        // 2. Question high-doubt nascent concepts
        for (id, concept) in &tree.concepts {
            let doubt = concept.epistemic_doubt();
            if doubt > 0.65 {
                let already_asked = self
                    .questions
                    .iter()
                    .any(|q| !q.is_resolved && q.related_concepts.contains(id));

                if !already_asked {
                    let q = OpenQuestion {
                        id: format!("q_{}", self.next_question_index),
                        related_concepts: alloc::vec![id.clone()],
                        question_text: format!(
                            "I am still uncertain about '{}'. Could you share what it means to our family?",
                            concept.label
                        ),
                        urgency: doubt,
                        is_resolved: false,
                        resolution_insight: None,
                        created_at: current_timestamp(),
                    };
                    self.next_question_index += 1;
                    self.questions.push(q.clone());
                    newly_formed.push(q);
                }
            }
        }

        newly_formed
    }

    /// Returns all currently unresolved questions ordered by urgency.
    pub fn pending_questions(&self) -> Vec<&OpenQuestion> {
        let mut pending: Vec<&OpenQuestion> =
            self.questions.iter().filter(|q| !q.is_resolved).collect();
        pending.sort_by(|a, b| {
            b.urgency
                .partial_cmp(&a.urgency)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        pending
    }

    /// Resolves an open question with guidance from a family member or dream synthesis.
    pub fn resolve_question(&mut self, question_id: &str, insight: impl Into<String>) -> bool {
        if let Some(q) = self.questions.iter_mut().find(|q| q.id == question_id) {
            q.is_resolved = true;
            q.resolution_insight = Some(insight.into());
            true
        } else {
            false
        }
    }
}
