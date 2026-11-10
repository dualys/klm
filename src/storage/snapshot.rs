extern crate alloc;

use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::error::{KlmError, Result};
use crate::mind::state::CognitiveState;
use crate::ontology::tree::KnowledgeTree;
use crate::storage::kpack_engine::KpackRecipe;

/// Intermediate payload bundle for full cognitive state serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotPayload {
    tree: KnowledgeTree,
    state: CognitiveState,
    epoch: u64,
}

/// A cryptographically verified and Kpack-compressed cognitive snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSnapshot {
    /// Global BLAKE3 Merkle root digest of the snapshot
    pub merkle_root: [u8; 32],
    /// Compressed Kpack formula recipe of the payload
    pub recipe: KpackRecipe,
    /// Swarm epoch at snapshot creation
    pub epoch: u64,
    /// Epoch timestamp or logical tick
    pub created_at: i64,
}

impl CognitiveSnapshot {
    /// Creates a verified compressed snapshot from the Knowledge Tree and cognitive state.
    pub fn create(tree: &KnowledgeTree, state: &CognitiveState, epoch: u64) -> Result<Self> {
        let payload = SnapshotPayload {
            tree: tree.clone(),
            state: state.clone(),
            epoch,
        };

        let json_bytes = serde_json::to_vec(&payload)?;
        let recipe = KpackRecipe::encode(&json_bytes);
        let merkle_root = recipe.blake3_hash;

        Ok(Self {
            merkle_root,
            recipe,
            epoch,
            created_at: current_timestamp(),
        })
    }

    /// Restores and verifies the Knowledge Tree and cognitive state from this snapshot.
    pub fn restore(&self) -> Result<(KnowledgeTree, CognitiveState)> {
        let json_bytes = self.recipe.decode()?;
        let payload: SnapshotPayload = serde_json::from_slice(&json_bytes)?;

        // Verify root match
        let computed_hash = blake3::hash(&json_bytes);
        if computed_hash.as_bytes() != &self.merkle_root {
            return Err(KlmError::StorageError(
                "Merkle root mismatch during snapshot restoration".into(),
            ));
        }

        Ok((payload.tree, payload.state))
    }
}
