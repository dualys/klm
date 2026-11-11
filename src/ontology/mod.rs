#[doc = "This module provides functionality related to ontology operations."]
pub mod concept;

#[doc = "This module provides functionality related to relation operations."]
pub mod relation;

#[doc = "This module provides functionality related to tree operations."]
pub mod tree;

pub use concept::{Concept, ConceptId, ConceptOrigin, EMBEDDING_DIM, compute_text_embedding};
pub use relation::{RelationEdge, RelationType};
pub use tree::KnowledgeTree;
