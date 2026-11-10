//! Integration of `cubefs` (Copy-On-Write `Nucleator`, `Checkpoint`, `Extent`, and `KpackReader`).
//! Provides immutable checkpointing and CoW sector mutation for Kōgnitara's long-term vault.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use cubefs::{BTreeNode, Extent, Inode, KpackReader, Nucleator};

use crate::clock::current_timestamp;
use crate::error::{KlmError, Result};

/// Immutable CubeFS Checkpoint structure (mirroring the `#[repr(C)]` CubeFS layout).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultCheckpoint {
    /// Sequence number of the checkpoint
    pub seq: u64,
    /// UTC hardware timestamp or logical tick
    pub time: u64,
    /// BLAKE3 hash of the root B-Tree metadata node (`cubefs::BTreeNode`)
    pub root_meta_id: [u8; 32],
    /// Global BLAKE3 Merkle root of the cognitive graph
    pub merkle_root: [u8; 32],
    /// 64-byte cryptographic checkpoint signature
    pub signature: [u8; 64],
}

/// Manages `VaultCheckpoint` states, 4096-byte `cubefs::BTreeNode` leaves, `cubefs::Inode` descriptors,
/// and `cubefs::Extent`-mapped Kpack chunks.
pub struct CubeFsVault {
    /// Historical immutable checkpoints ordered by sequence number
    pub checkpoints: Vec<VaultCheckpoint>,
    /// Stored Kpack recipe chunks indexed by their 32-byte BLAKE3 chunk ID
    pub chunk_store: BTreeMap<[u8; 32], Vec<u8>>,
    /// Active file extents mapping logical offsets to Kpack chunk IDs
    pub extents: Vec<Extent>,
    /// Active logical inode tracking total decompressed size
    pub root_inode: Inode,
    /// Current 4096-byte Copy-On-Write root B-Tree metadata node
    pub root_btree_node: BTreeNode,
}

impl Default for CubeFsVault {
    fn default() -> Self {
        Self::new()
    }
}

impl CubeFsVault {
    /// Initializes a clean CubeFS cognitive vault with a genesis `BTreeNode`.
    pub fn new() -> Self {
        let mut root_btree_node = BTreeNode {
            magic: *b"KLMN",
            version: 1,
            level: 0,
            entry_count: 0,
            payload: [0u8; 4056],
            blake3_hash: [0u8; 32],
        };
        root_btree_node.blake3_hash = Self::hash_btree_node(&root_btree_node);

        Self {
            checkpoints: Vec::new(),
            chunk_store: BTreeMap::new(),
            extents: Vec::new(),
            root_inode: Inode {
                id: root_btree_node.blake3_hash,
                size: 0,
            },
            root_btree_node,
        }
    }

    /// Computes the BLAKE3 identity of a `cubefs::BTreeNode` over its header and payload.
    pub fn hash_btree_node(node: &BTreeNode) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&node.magic);
        hasher.update(&[node.version, node.level]);
        hasher.update(&node.entry_count.to_le_bytes());
        hasher.update(&node.payload);
        *hasher.finalize().as_bytes()
    }

    /// Performs a pure Copy-On-Write byte insertion on a 4096-byte page via `cubefs::Nucleator::insert_byte`.
    pub fn cow_mutate_page(
        page: &[u8; 4096],
        offset: usize,
        new_byte: u8,
    ) -> Result<([u8; 4096], u8)> {
        Nucleator::insert_byte(page, offset, new_byte).ok_or_else(|| {
            KlmError::StorageError("CubeFS Nucleator offset out of 4096-byte page bounds".into())
        })
    }

    /// Registers a raw Kpack chunk instruction `[opcode, param_lo, param_hi, payload...]` as a `cubefs::Extent`
    /// and seals a new `VaultCheckpoint`.
    pub fn commit_kpack_extent(
        &mut self,
        opcode: kpack::Opcode,
        param: u16,
        payload: &[u8],
        logical_len: u64,
        merkle_root: [u8; 32],
    ) -> VaultCheckpoint {
        let mut raw_recipe = Vec::with_capacity(3 + payload.len());
        raw_recipe.push(opcode as u8);
        raw_recipe.extend_from_slice(&param.to_le_bytes());
        raw_recipe.extend_from_slice(payload);

        let chunk_id: [u8; 32] = *blake3::hash(&raw_recipe).as_bytes();
        self.chunk_store.insert(chunk_id, raw_recipe);

        let next_off = self.extents.last().map(|e| e.file_off + e.len).unwrap_or(0);
        let extent = Extent {
            file_off: next_off,
            len: logical_len.max(1),
            first_chunk_id: chunk_id,
            count: 1,
            last_len: logical_len as u32,
        };
        self.extents.push(extent);
        self.root_inode.size = self.root_inode.size.saturating_add(logical_len);

        // Update BTreeNode entry count and recompute its BLAKE3 hash
        self.root_btree_node.entry_count = self.root_btree_node.entry_count.saturating_add(1);
        let idx = ((self.root_btree_node.entry_count as usize - 1) * 32) % 4024;
        self.root_btree_node.payload[idx..idx + 32].copy_from_slice(&chunk_id);
        self.root_btree_node.blake3_hash = Self::hash_btree_node(&self.root_btree_node);
        self.root_inode.id = self.root_btree_node.blake3_hash;

        let seq = (self.checkpoints.len() as u64) + 1;
        let time = current_timestamp().max(0) as u64;

        // Derive deterministic 64-byte checkpoint signature from root_meta_id and merkle_root
        let mut sig = [0u8; 64];
        let mut hasher = blake3::Hasher::new();
        hasher.update(&seq.to_le_bytes());
        hasher.update(&self.root_btree_node.blake3_hash);
        hasher.update(&merkle_root);
        let h1 = *hasher.finalize().as_bytes();
        sig[..32].copy_from_slice(&h1);
        sig[32..].copy_from_slice(&merkle_root);

        let cp = VaultCheckpoint {
            seq,
            time,
            root_meta_id: self.root_btree_node.blake3_hash,
            merkle_root,
            signature: sig,
        };
        self.checkpoints.push(cp);
        cp
    }

    /// Reads and reconstructs a logical section at `offset` directly using `cubefs::KpackReader`.
    pub fn read_section(&self, offset: u64, section_size: usize) -> Result<Vec<u8>> {
        let reader = KpackReader::new(&self.extents);
        let mut buffer = vec![0u8; section_size];
        let store = &self.chunk_store;

        reader
            .read_into_section(offset, &mut buffer, |chunk_id| {
                store
                    .get(&chunk_id)
                    .map(|v| v.as_slice())
                    .ok_or("Chunk ID not found in CubeFS vault")
            })
            .map_err(|e| KlmError::StorageError(e.into()))?;

        Ok(buffer)
    }
}
