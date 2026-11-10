extern crate alloc;

use alloc::vec::Vec;
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
    /// Optional parent snapshot Merkle root if this is a differential (`kpack::Opcode::Delta`) snapshot
    pub parent_merkle_root: Option<[u8; 32]>,
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
            parent_merkle_root: None,
            recipe,
            epoch,
            created_at: current_timestamp(),
        })
    }

    /// Creates a differential (`kpack::Opcode::Delta`) snapshot relative to a parent snapshot's raw bytes.
    pub fn create_delta(
        parent: &CognitiveSnapshot,
        tree: &KnowledgeTree,
        state: &CognitiveState,
        epoch: u64,
    ) -> Result<Self> {
        let parent_bytes = parent.raw_bytes()?;
        let payload = SnapshotPayload {
            tree: tree.clone(),
            state: state.clone(),
            epoch,
        };

        let json_bytes = serde_json::to_vec(&payload)?;
        let recipe = KpackRecipe::encode_delta(&json_bytes, &parent_bytes);
        let merkle_root = recipe.blake3_hash;

        Ok(Self {
            merkle_root,
            parent_merkle_root: Some(parent.merkle_root),
            recipe,
            epoch,
            created_at: current_timestamp(),
        })
    }

    /// Decodes the raw serialized bytes of a standalone snapshot.
    pub fn raw_bytes(&self) -> Result<Vec<u8>> {
        self.recipe.decode()
    }

    /// Restores and verifies the Knowledge Tree and cognitive state from this snapshot.
    pub fn restore(&self) -> Result<(KnowledgeTree, CognitiveState)> {
        let json_bytes = self.recipe.decode()?;
        self.parse_and_verify(&json_bytes)
    }

    /// Restores and verifies a differential (`Delta`) snapshot using its parent snapshot.
    pub fn restore_with_parent(
        &self,
        parent: &CognitiveSnapshot,
    ) -> Result<(KnowledgeTree, CognitiveState)> {
        if let Some(expected_parent) = self.parent_merkle_root
            && expected_parent != parent.merkle_root
        {
            return Err(KlmError::StorageError(
                "Parent Merkle root mismatch for differential Kpack snapshot".into(),
            ));
        }

        let parent_bytes = parent.raw_bytes()?;
        let json_bytes = self.recipe.decode_with_reference(Some(&parent_bytes))?;
        self.parse_and_verify(&json_bytes)
    }

    fn parse_and_verify(&self, json_bytes: &[u8]) -> Result<(KnowledgeTree, CognitiveState)> {
        let computed_hash = blake3::hash(json_bytes);
        if computed_hash.as_bytes() != &self.merkle_root {
            return Err(KlmError::StorageError(
                "Merkle root mismatch during snapshot restoration".into(),
            ));
        }
        let payload: SnapshotPayload = serde_json::from_slice(json_bytes)?;
        Ok((payload.tree, payload.state))
    }
}
