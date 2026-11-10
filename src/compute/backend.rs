extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::compute::math::{exp_f32, sqrt_f32};

/// Supported open hardware device profiles (strictly sovereign and open-source; no NVIDIA/CUDA).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Portable open CPU backend (SIMD / Multithreaded / bare-metal Amentys OS kernel)
    CpuNative,
    /// Open-source AMD Radeon / Instinct acceleration via ROCm / HIP
    AmdRocm,
    /// Cross-platform open GPU compute via Vulkan Compute / WGPU
    VulkanCompute,
}

/// Abstract compute backend trait for open hardware tensor and neuro-symbolic vector operations.
pub trait ComputeBackend {
    /// Returns the descriptive name of the compute backend.
    fn name(&self) -> &'static str;

    /// Returns the active device type profile.
    fn device_type(&self) -> DeviceType;

    /// Confirms that the backend adheres to Kōgnitara's open-hardware sovereignty policy.
    fn is_open_source_sovereign(&self) -> bool {
        true
    }

    /// Computes the dot product between two floating-point vectors.
    fn vector_dot(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Computes the cosine similarity between two embedding vectors in `[-1.0, 1.0]`.
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot = self.vector_dot(a, b);
        let norm_a: f32 = a.iter().map(|x| x * x).sum();
        let norm_b: f32 = b.iter().map(|x| x * x).sum();

        if norm_a <= 1e-12 || norm_b <= 1e-12 {
            0.0
        } else {
            dot / (sqrt_f32(norm_a) * sqrt_f32(norm_b))
        }
    }

    /// Computes the Euclidean (L2) distance between two embedding vectors.
    fn l2_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let sum_sq: f32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let d = x - y;
                d * d
            })
            .sum();
        sqrt_f32(sum_sq)
    }

    /// Accumulates a scaled vector: `dest += src * scale`.
    fn vector_add_scaled(&self, dest: &mut [f32], src: &[f32], scale: f32) {
        for (d, s) in dest.iter_mut().zip(src.iter()) {
            *d += s * scale;
        }
    }

    /// Normalizes a vector in-place to unit L2 length.
    fn normalize(&self, vec: &mut [f32]) {
        let sum_sq: f32 = vec.iter().map(|x| x * x).sum();
        if sum_sq > 1e-12 {
            let inv_norm = 1.0 / sqrt_f32(sum_sq);
            for x in vec.iter_mut() {
                *x *= inv_norm;
            }
        }
    }

    /// Blends two conceptual embeddings by linear interpolation (`(1 - ratio_b)*a + ratio_b*b`)
    /// and normalizes the resulting synthesized vector.
    fn blend_embeddings(&self, a: &[f32], b: &[f32], ratio_b: f32) -> Vec<f32> {
        let len = a.len().min(b.len());
        let ratio = ratio_b.clamp(0.0, 1.0);
        let inv = 1.0 - ratio;
        let mut blended = Vec::with_capacity(len);
        for i in 0..len {
            blended.push(a[i] * inv + b[i] * ratio);
        }
        self.normalize(&mut blended);
        blended
    }

    /// Computes a numerically stable softmax attention distribution over a slice of scores.
    fn softmax_attention(&self, scores: &[f32]) -> Vec<f32> {
        if scores.is_empty() {
            return Vec::new();
        }
        let max_score = scores
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, |a, b| if a > b { a } else { b });
        let mut exps: Vec<f32> = scores.iter().map(|&s| exp_f32(s - max_score)).collect();
        let sum: f32 = exps.iter().sum();
        if sum > 1e-12 {
            for e in &mut exps {
                *e /= sum;
            }
        } else {
            let uniform = 1.0 / (scores.len() as f32);
            exps = vec![uniform; scores.len()];
        }
        exps
    }
}
