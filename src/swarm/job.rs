//! Distributed execution of a Kpack job across the 8 logical nodes (`000` to `111`)
//! of the Hamon Cube architecture, with 3D butterfly Merkle reduction ($X \to Y \to Z$).

extern crate alloc;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{KlmError, Result};
use crate::storage::kpack_engine::KpackRecipe;
use crate::swarm::cube::NodeCoord;

/// A single shard of a distributed Kpack job executed by one Hamon Cube node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeKpackShard {
    /// Executing node coordinate (`000` to `111`)
    pub node: NodeCoord,
    /// Byte offset of this shard in the full logical payload
    pub byte_offset: usize,
    /// Compressed Kpack recipe computed by this node
    pub recipe: KpackRecipe,
    /// BLAKE3 digest of this node's uncompressed shard
    pub shard_hash: [u8; 32],
}

/// Result of executing a distributed Kpack job across the 8-node Hamon Cube.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedKpackJob {
    /// The 8 node shards ordered from `000` (0) to `111` (7)
    pub shards: Vec<NodeKpackShard>,
    /// Unified 3D hypercube Merkle root reduced across dimensions $X$, $Y$, and $Z$
    pub cube_merkle_root: [u8; 32],
    /// Total uncompressed payload byte size
    pub total_uncompressed_bytes: usize,
}

impl DistributedKpackJob {
    /// Scatters `payload` across the 8 logical nodes of the Hamon Cube (`000`..=`111`),
    /// encodes each shard into a `KpackRecipe`, and computes the 3D butterfly Merkle root.
    pub fn dispatch(payload: &[u8]) -> Self {
        let total_len = payload.len();
        let base_chunk = total_len / 8;
        let remainder = total_len % 8;

        let mut shards = Vec::with_capacity(8);
        let mut current_offset = 0;

        for idx in 0..8u8 {
            let extra = if (idx as usize) < remainder { 1 } else { 0 };
            let shard_len = base_chunk + extra;
            let slice = &payload[current_offset..current_offset + shard_len];

            let recipe = KpackRecipe::encode(slice);
            let shard_hash = recipe.blake3_hash;

            shards.push(NodeKpackShard {
                node: NodeCoord::new(idx),
                byte_offset: current_offset,
                recipe,
                shard_hash,
            });

            current_offset += shard_len;
        }

        let cube_merkle_root = Self::compute_hypercube_merkle_root(&shards);

        Self {
            shards,
            cube_merkle_root,
            total_uncompressed_bytes: total_len,
        }
    }

    /// Performs a 3-stage hypercube butterfly reduction along axes X (bit 0), Y (bit 1), and Z (bit 2).
    pub fn compute_hypercube_merkle_root(shards: &[NodeKpackShard]) -> [u8; 32] {
        let mut hashes = [[0u8; 32]; 8];
        for (i, shard) in shards.iter().take(8).enumerate() {
            hashes[i] = shard.shard_hash;
        }

        // Stage 1: Reduce along X-axis (stride 1: pairs (0,1), (2,3), (4,5), (6,7))
        let mut x_reduced = [[0u8; 32]; 4];
        for i in 0..4 {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&hashes[2 * i]);
            hasher.update(&hashes[2 * i + 1]);
            x_reduced[i] = *hasher.finalize().as_bytes();
        }

        // Stage 2: Reduce along Y-axis (pairs of X-reduced pairs)
        let mut y_reduced = [[0u8; 32]; 2];
        for i in 0..2 {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&x_reduced[2 * i]);
            hasher.update(&x_reduced[2 * i + 1]);
            y_reduced[i] = *hasher.finalize().as_bytes();
        }

        // Stage 3: Reduce along Z-axis (final cube root)
        let mut root_hasher = blake3::Hasher::new();
        root_hasher.update(&y_reduced[0]);
        root_hasher.update(&y_reduced[1]);
        *root_hasher.finalize().as_bytes()
    }

    /// Reconstructs and cryptographically verifies the full payload from the 8 node shards.
    pub fn gather_and_verify(&self) -> Result<Vec<u8>> {
        if self.shards.len() != 8 {
            return Err(KlmError::SwarmError(
                "Hamon Cube distributed Kpack job requires exactly 8 node shards".into(),
            ));
        }

        let recomputed_root = Self::compute_hypercube_merkle_root(&self.shards);
        if recomputed_root != self.cube_merkle_root {
            return Err(KlmError::SwarmError(
                "Hamon Cube 3D Merkle root verification failed".into(),
            ));
        }

        let mut output = Vec::with_capacity(self.total_uncompressed_bytes);
        for shard in &self.shards {
            let decoded = shard.recipe.decode()?;
            output.extend_from_slice(&decoded);
        }

        Ok(output)
    }
}
