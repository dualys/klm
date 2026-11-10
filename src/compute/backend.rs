extern crate alloc;

use serde::{Deserialize, Serialize};

/// Supported open hardware device profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Portable open CPU backend (SIMD / Multithreaded / bare-metal kernel)
    CpuNative,
    /// Open-source AMD Radeon / Instinct acceleration via ROCm / HIP
    AmdRocm,
    /// Cross-platform open GPU compute via Vulkan Compute / WGPU
    VulkanCompute,
}

/// Abstract compute backend trait for open hardware tensor/vector operations.
pub trait ComputeBackend {
    /// Returns the descriptive name of the compute backend.
    fn name(&self) -> &'static str;

    /// Returns the active device type profile.
    fn device_type(&self) -> DeviceType;

    /// Computes the dot product between two floating-point vectors.
    fn vector_dot(&self, a: &[f32], b: &[f32]) -> f32;

    /// Computes the cosine similarity between two embedding vectors.
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32;

    /// Accumulates a scaled vector: `dest += src * scale`.
    fn vector_add_scaled(&self, dest: &mut [f32], src: &[f32], scale: f32);

    /// Normalizes a vector in-place to unit length.
    fn normalize(&self, vec: &mut [f32]);
}
