extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use kpack::{Opcode, execute};
use serde::{Deserialize, Serialize};

use crate::error::{KlmError, Result};

/// High-level serializable representation of all 7 native `kpack::Opcode` primitives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KpackOpcode {
    /// Raw uncompressed literal byte sequence (`kpack::Opcode::Lit`)
    Lit(Vec<u8>),
    /// Run-Length Encoding repetition: repeats a byte `count` times (`kpack::Opcode::Rle`)
    Rle { byte: u8, count: usize },
    /// Differential copy/add patch instructions against a reference buffer (`kpack::Opcode::Delta`)
    Delta {
        instructions: Vec<u8>,
        output_len: usize,
    },
    /// Pseudo-random generative xorshift sequence with sparse byte patches (`kpack::Opcode::Seed`)
    Seed {
        seed: u32,
        patches: Vec<u8>,
        output_len: usize,
    },
    /// Dictionary-based word indexing (`kpack::Opcode::Dict`)
    Dict {
        word_size: u32,
        indices: Vec<u8>,
        output_len: usize,
    },
    /// Bitwise XOR transformation against a constant key or reference buffer (`kpack::Opcode::Xor`)
    Xor { key: u8, payload: Vec<u8> },
    /// Bitwise right shift operation on each byte (`kpack::Opcode::Rs`)
    Rs { shift: u8, payload: Vec<u8> },
}

impl KpackOpcode {
    /// Returns the underlying native `kpack::Opcode` discriminant.
    pub fn to_native_opcode(&self) -> Opcode {
        match self {
            KpackOpcode::Lit(_) => Opcode::Lit,
            KpackOpcode::Rle { .. } => Opcode::Rle,
            KpackOpcode::Delta { .. } => Opcode::Delta,
            KpackOpcode::Seed { .. } => Opcode::Seed,
            KpackOpcode::Dict { .. } => Opcode::Dict,
            KpackOpcode::Xor { .. } => Opcode::Xor,
            KpackOpcode::Rs { .. } => Opcode::Rs,
        }
    }

    /// Executes this instruction directly through the native `kpack::execute` virtual machine.
    pub fn execute_native(&self, reference_buffer: Option<&[u8]>) -> Vec<u8> {
        match self {
            KpackOpcode::Lit(bytes) => {
                let mut out = vec![0u8; bytes.len()];
                execute(&Opcode::Lit, bytes.len() as u32, bytes, &mut out, None);
                out
            }
            KpackOpcode::Rle { byte, count } => {
                let mut out = vec![0u8; *count];
                let payload = [*byte];
                execute(&Opcode::Rle, *count as u32, &payload, &mut out, None);
                out
            }
            KpackOpcode::Delta {
                instructions,
                output_len,
            } => {
                let mut out = vec![0u8; *output_len];
                execute(
                    &Opcode::Delta,
                    0,
                    instructions,
                    &mut out,
                    reference_buffer,
                );
                out
            }
            KpackOpcode::Seed {
                seed,
                patches,
                output_len,
            } => {
                let mut out = vec![0u8; *output_len];
                execute(&Opcode::Seed, *seed, patches, &mut out, None);
                out
            }
            KpackOpcode::Dict {
                word_size,
                indices,
                output_len,
            } => {
                let mut out = vec![0u8; *output_len];
                execute(
                    &Opcode::Dict,
                    *word_size,
                    indices,
                    &mut out,
                    reference_buffer,
                );
                out
            }
            KpackOpcode::Xor { key, payload } => {
                let mut out = vec![0u8; payload.len()];
                execute(
                    &Opcode::Xor,
                    u32::from(*key),
                    payload,
                    &mut out,
                    reference_buffer,
                );
                out
            }
            KpackOpcode::Rs { shift, payload } => {
                let mut out = vec![0u8; payload.len()];
                execute(&Opcode::Rs, u32::from(*shift), payload, &mut out, None);
                out
            }
        }
    }
}

/// A structured Kpack formula recipe reconstructing a data chunk with BLAKE3 cryptographic verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpackRecipe {
    /// Ordered sequence of formula opcodes executed sequentially
    pub opcodes: Vec<KpackOpcode>,
    /// Expected final uncompressed byte length
    pub uncompressed_length: usize,
    /// BLAKE3 256-bit cryptographic digest of the reconstructed block
    pub blake3_hash: [u8; 32],
}

impl KpackRecipe {
    /// Compresses a raw byte buffer into a hybrid RLE + Literal Kpack recipe.
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

        // Hybrid scanner: segments repetitive runs (>= 6 identical bytes) as RLE and remaining spans as Lit
        let mut i = 0;
        let mut lit_start = 0;

        while i < data.len() {
            let b = data[i];
            let mut run_len = 1;
            while i + run_len < data.len() && data[i + run_len] == b {
                run_len += 1;
            }

            if run_len >= 6 {
                if lit_start < i {
                    opcodes.push(KpackOpcode::Lit(data[lit_start..i].to_vec()));
                }
                opcodes.push(KpackOpcode::Rle {
                    byte: b,
                    count: run_len,
                });
                i += run_len;
                lit_start = i;
            } else {
                i += 1;
            }
        }

        if lit_start < data.len() {
            opcodes.push(KpackOpcode::Lit(data[lit_start..].to_vec()));
        }

        Self {
            opcodes,
            uncompressed_length: data.len(),
            blake3_hash: *hash.as_bytes(),
        }
    }

    /// Encodes a buffer using the native `kpack::Opcode::Xor` transformation with a key.
    pub fn encode_xor(data: &[u8], key: u8) -> Self {
        let hash = blake3::hash(data);
        let transformed: Vec<u8> = data.iter().map(|b| b ^ key).collect();
        Self {
            opcodes: vec![KpackOpcode::Xor {
                key,
                payload: transformed,
            }],
            uncompressed_length: data.len(),
            blake3_hash: *hash.as_bytes(),
        }
    }

    /// Encodes `data` as a native `kpack::Opcode::Delta` stream relative to a `reference` slice.
    pub fn encode_delta(data: &[u8], reference: &[u8]) -> Self {
        let hash = blake3::hash(data);
        let mut instructions = Vec::new();

        // Find common prefix with reference buffer (up to u16::MAX)
        let max_prefix = data.len().min(reference.len()).min(u16::MAX as usize);
        let mut prefix_len = 0;
        while prefix_len < max_prefix && data[prefix_len] == reference[prefix_len] {
            prefix_len += 1;
        }

        if prefix_len > 0 {
            // 0x01: Copy from reference [offset: u16 LE, size: u16 LE]
            instructions.push(0x01);
            instructions.extend_from_slice(&0u16.to_le_bytes());
            instructions.extend_from_slice(&(prefix_len as u16).to_le_bytes());
        }

        let remaining = &data[prefix_len..];
        if !remaining.is_empty() {
            let chunk_size = remaining.len().min(u16::MAX as usize);
            // 0x02: Add literal bytes [size: u16 LE, bytes...]
            instructions.push(0x02);
            instructions.extend_from_slice(&(chunk_size as u16).to_le_bytes());
            instructions.extend_from_slice(&remaining[..chunk_size]);
        }

        // 0x00: End of delta stream
        instructions.push(0x00);

        Self {
            opcodes: vec![KpackOpcode::Delta {
                instructions,
                output_len: data.len(),
            }],
            uncompressed_length: data.len(),
            blake3_hash: *hash.as_bytes(),
        }
    }

    /// Encodes a buffer using `kpack::Opcode::Dict` against a fixed-width word dictionary.
    pub fn encode_dict(data: &[u8], word_size: usize, dictionary: &[u8]) -> Option<Self> {
        if word_size == 0 || !data.len().is_multiple_of(word_size) {
            return None;
        }
        let num_dict_words = dictionary.len() / word_size;
        if num_dict_words == 0 || num_dict_words > 256 {
            return None;
        }

        let mut indices = Vec::with_capacity(data.len() / word_size);
        for chunk in data.chunks_exact(word_size) {
            let found = (0..num_dict_words).find(|&idx| {
                let offset = idx * word_size;
                &dictionary[offset..offset + word_size] == chunk
            })?;
            indices.push(found as u8);
        }

        let hash = blake3::hash(data);
        Some(Self {
            opcodes: vec![KpackOpcode::Dict {
                word_size: word_size as u32,
                indices,
                output_len: data.len(),
            }],
            uncompressed_length: data.len(),
            blake3_hash: *hash.as_bytes(),
        })
    }

    /// Encodes a small buffer using `kpack::Opcode::Seed` (PRNG stream + exact byte patches).
    pub fn encode_seed(data: &[u8], seed: u32) -> Self {
        let hash = blake3::hash(data);
        let mut baseline = vec![0u8; data.len()];
        execute(&Opcode::Seed, seed, &[], &mut baseline, None);

        let mut patch_entries: Vec<(u16, u8)> = Vec::new();
        for (idx, (&target_b, &base_b)) in data.iter().zip(baseline.iter()).enumerate() {
            if target_b != base_b && idx <= u16::MAX as usize {
                patch_entries.push((idx as u16, target_b));
            }
        }

        let mut patches = Vec::with_capacity(2 + patch_entries.len() * 3);
        let count = patch_entries.len().min(u16::MAX as usize) as u16;
        patches.extend_from_slice(&count.to_le_bytes());
        for &(pos, byte) in patch_entries.iter().take(count as usize) {
            patches.extend_from_slice(&pos.to_le_bytes());
            patches.push(byte);
        }

        Self {
            opcodes: vec![KpackOpcode::Seed {
                seed,
                patches,
                output_len: data.len(),
            }],
            uncompressed_length: data.len(),
            blake3_hash: *hash.as_bytes(),
        }
    }

    /// Evaluates the recipe without a reference buffer and verifies BLAKE3 integrity.
    pub fn decode(&self) -> Result<Vec<u8>> {
        self.decode_with_reference(None)
    }

    /// Evaluates the recipe through `kpack::execute` with an optional reference buffer and verifies BLAKE3 integrity.
    pub fn decode_with_reference(&self, reference_buffer: Option<&[u8]>) -> Result<Vec<u8>> {
        let mut reconstructed = Vec::with_capacity(self.uncompressed_length);

        for op in &self.opcodes {
            let segment = op.execute_native(reference_buffer);
            reconstructed.extend_from_slice(&segment);
        }

        let hash = blake3::hash(&reconstructed);
        if hash.as_bytes() != &self.blake3_hash {
            return Err(KlmError::StorageError(
                "BLAKE3 integrity check failed during Kpack recipe decoding".into(),
            ));
        }

        Ok(reconstructed)
    }
}
