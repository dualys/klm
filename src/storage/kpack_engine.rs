extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{KlmError, Result};

/// Recipe opcodes for Kpack formulaic reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KpackOpcode {
    /// Raw uncompressed literal byte sequence
    Lit(Vec<u8>),
    /// Run-Length Encoding repetition: repeats a byte `n` times
    Rle { byte: u8, count: usize },
    /// Differential patch against a baseline offset
    Delta { offset: usize, diff: Vec<u8> },
    /// Pseudo-random generative formula from a deterministic seed
    Seed { seed: u64, length: usize },
}

/// A structured Kpack formula recipe reconstructing a data chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpackRecipe {
    /// Ordered sequence of formula opcodes
    pub opcodes: Vec<KpackOpcode>,
    /// Expected final uncompressed byte length
    pub uncompressed_length: usize,
    /// BLAKE3 256-bit cryptographic digest of the reconstructed block
    pub blake3_hash: [u8; 32],
}

impl KpackRecipe {
    /// Compresses a raw byte buffer into a Kpack recipe.
    pub fn encode(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        let mut opcodes = Vec::new();

        if data.is_empty() {
            return Self {
                opcodes,
                uncompressed_length: 0,
                blake3_hash: *hash.as_bytes(),
            };
        }

        // Check for uniform repetitive runs (RLE optimization)
        let first = data[0];
        let all_same = data.iter().all(|&b| b == first);

        if all_same && data.len() > 16 {
            opcodes.push(KpackOpcode::Rle {
                byte: first,
                count: data.len(),
            });
        } else {
            opcodes.push(KpackOpcode::Lit(data.to_vec()));
        }

        Self {
            opcodes,
            uncompressed_length: data.len(),
            blake3_hash: *hash.as_bytes(),
        }
    }

    /// Evaluates the recipe and reconstructs the original byte buffer, verifying BLAKE3 integrity.
    pub fn decode(&self) -> Result<Vec<u8>> {
        let mut reconstructed = Vec::with_capacity(self.uncompressed_length);

        for op in &self.opcodes {
            match op {
                KpackOpcode::Lit(bytes) => {
                    reconstructed.extend_from_slice(bytes);
                }
                KpackOpcode::Rle { byte, count } => {
                    reconstructed.extend(vec![*byte; *count]);
                }
                KpackOpcode::Delta { offset, diff } => {
                    if *offset < reconstructed.len() {
                        let end = (offset + diff.len()).min(reconstructed.len());
                        for (i, &b) in diff.iter().enumerate() {
                            if offset + i < end {
                                reconstructed[offset + i] ^= b;
                            }
                        }
                    }
                }
                KpackOpcode::Seed { seed, length } => {
                    let mut state = *seed;
                    for _ in 0..*length {
                        // Fast LCG generator
                        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                        reconstructed.push((state >> 33) as u8);
                    }
                }
            }
        }

        // Cryptographic verification
        let hash = blake3::hash(&reconstructed);
        if hash.as_bytes() != &self.blake3_hash {
            return Err(KlmError::StorageError(
                "BLAKE3 integrity check failed during Kpack decoding".into(),
            ));
        }

        Ok(reconstructed)
    }
}
