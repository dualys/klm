//! Integration of the `jinshu` Merkle-Trie Ocean and `noun::Noun` cryptographic identity engine.
//! Allows Kōgnitara to store and navigate cognitive concepts and dreams as 4096-byte NVMe-aligned `TrieNode` pages.

extern crate alloc;

use crate::error::{KlmError, Result};
use crate::ontology::concept::Concept;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::RefCell;
use jinshu::node::TrieNode;
use jinshu::router::NounIndex;
use jinshu::storage::{BlockDevice, StorageEngine};
use noun::Noun;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;

/// In-memory 4096-byte page block device implementing `jinshu::storage::BlockDevice`.
/// Compatible with `no_std` + `alloc` for Amentys OS and user-space testing.
#[derive(Debug, Default)]
pub struct MemoryBlockDevice {
    /// Maps Logical Block Address (LBA) -> 4096-byte sector group
    blocks: RefCell<BTreeMap<u64, [u8; 4096]>>,
}

pub struct PhysicalBlockDevice {
    file: File,
}

impl PhysicalBlockDevice {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self { file })
    }
}

impl BlockDevice for PhysicalBlockDevice {
    fn write_node(&self, lba: u64, data_phys_addr: u64) {
        let offset = lba * 4096;
        let data_slice = unsafe { core::slice::from_raw_parts(data_phys_addr as *const u8, 4096) };
        self.file
            .write_at(data_slice, offset)
            .expect("Erreur I/O écriture brute");
    }

    fn read_node(&self, lba: u64, dest_phys_addr: u64) {
        let offset = lba * 4096;
        let dest_slice =
            unsafe { core::slice::from_raw_parts_mut(dest_phys_addr as *mut u8, 4096) };
        self.file
            .read_at(dest_slice, offset)
            .expect("Erreur I/O lecture brute");
    }
}
impl MemoryBlockDevice {
    /// Creates a new empty virtual NVMe block device.
    pub fn new() -> Self {
        Self {
            blocks: RefCell::new(BTreeMap::new()),
        }
    }

    /// Returns the number of 4096-byte `TrieNode` pages written to the device.
    pub fn allocated_pages(&self) -> usize {
        self.blocks.borrow().len()
    }
}

impl BlockDevice for MemoryBlockDevice {
    fn write_node(&self, lba: u64, data_phys_addr: u64) {
        if data_phys_addr == 0 {
            return;
        }
        let mut page = [0u8; 4096];
        // SAFETY: `data_phys_addr` points to a valid `TrieNode` in memory of size 4096 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(
                data_phys_addr as *const u8,
                page.as_mut_ptr(),
                core::mem::size_of::<TrieNode>(),
            );
        }
        self.blocks.borrow_mut().insert(lba, page);
    }

    fn read_node(&self, lba: u64, dest_phys_addr: u64) {
        if dest_phys_addr == 0 {
            return;
        }
        if let Some(page) = self.blocks.borrow().get(&lba) {
            // SAFETY: `dest_phys_addr` points to a valid `TrieNode` destination buffer in memory.
            unsafe {
                core::ptr::copy_nonoverlapping(
                    page.as_ptr(),
                    dest_phys_addr as *mut u8,
                    core::mem::size_of::<TrieNode>(),
                );
            }
        }
    }
}

/// Summary descriptor for a concept or memory crystallized into the Jinshu Ocean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OceanEntrySummary {
    /// 32-byte BLAKE3 Noun cryptographic identity
    pub noun_bytes: [u8; 32],
    /// Assigned Logical Block Address (LBA) in the Jinshu allocator
    pub lba: u64,
    /// Payload byte length encoded in the `TrieNode`
    pub payload_len: u32,
}

/// High-level manager bridging Kōgnitara's cognitive memory with `jinshu::node::TrieNode` and `NounIndex`.
pub struct CognitiveOcean {
    /// In-memory `CoreOcean` mapping `Noun` -> `TrieNode`
    pub core_ocean: BTreeMap<Noun, TrieNode>,
    /// Physical LBA lookup index (`jinshu::router::NounIndex`)
    pub noun_index: NounIndex,
    /// Root `TrieNode` collecting up to 16 nibble branches (`0..16`)
    pub root_node: TrieNode,
}

impl Default for CognitiveOcean {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveOcean {
    /// Initializes an empty Jinshu `CognitiveOcean`.
    pub fn new() -> Self {
        Self {
            core_ocean: BTreeMap::new(),
            noun_index: NounIndex::new(),
            root_node: TrieNode::new(),
        }
    }

    /// Packs a byte slice (up to 3576 bytes) into a `jinshu::node::TrieNode` using `kpack::Opcode::Lit`,
    /// persists it to the provided `BlockDevice`, indexes its `Noun`, and links it to the root nibble branch.
    pub fn crystallize_bytes<D: BlockDevice>(
        &mut self,
        data: &[u8],
        storage: &mut StorageEngine<'_, D>,
    ) -> Result<OceanEntrySummary> {
        if data.len() > 3576 {
            return Err(KlmError::StorageError(
                "Payload exceeds single Jinshu TrieNode payload capacity (3576 bytes)".into(),
            ));
        }

        let mut node = TrieNode::new();
        node.opcode = kpack::Opcode::Lit as u8;
        node.param = data.len() as u32;
        node.payload[..data.len()].copy_from_slice(data);

        let noun_bytes = node.calculate_noun();
        let noun = Noun::from_bytes(&noun_bytes)
            .ok_or_else(|| KlmError::StorageError("Invalid 32-byte Noun digest".into()))?;

        let node_addr = core::ptr::from_ref::<TrieNode>(&node) as u64;
        let lba = storage
            .persist_node(node_addr)
            .map_err(|e| KlmError::StorageError(e.into()))?;

        self.noun_index
            .insert(noun.clone(), lba)
            .map_err(|e| KlmError::StorageError(e.into()))?;

        // Link into root_node branch corresponding to the first nibble of the BLAKE3 hash
        let nibble = noun_bytes[0] & 0x0F;
        self.root_node.set_branch(nibble, noun.clone());
        self.core_ocean.insert(noun, node);

        Ok(OceanEntrySummary {
            noun_bytes,
            lba,
            payload_len: data.len() as u32,
        })
    }

    /// Crystallizes a single `Concept` into a 4096-byte Jinshu `TrieNode`.
    pub fn crystallize_concept<D: BlockDevice>(
        &mut self,
        concept: &Concept,
        storage: &mut StorageEngine<'_, D>,
    ) -> Result<OceanEntrySummary> {
        let bytes = serde_json::to_vec(concept)?;
        self.crystallize_bytes(&bytes, storage)
    }

    /// Reconstructs the payload of a `TrieNode` identified by its 32-byte `Noun` digest using `TrieNode::execute_recipe`.
    pub fn recall_bytes(&self, noun_bytes: &[u8; 32]) -> Result<Vec<u8>> {
        let noun = Noun::from_bytes(noun_bytes)
            .ok_or_else(|| KlmError::StorageError("Invalid Noun bytes".into()))?;

        let node = self
            .core_ocean
            .get(&noun)
            .ok_or_else(|| KlmError::StorageError("Noun not found in CognitiveOcean".into()))?;

        let len = (node.param as usize).min(node.payload.len());
        let mut output = alloc::vec![0u8; len];
        node.execute_recipe(&mut output, None);
        Ok(output)
    }

    /// Computes the global root `Noun` digest of the `CognitiveOcean`'s root `TrieNode`.
    pub fn root_noun(&self) -> [u8; 32] {
        self.root_node.calculate_noun()
    }
}
