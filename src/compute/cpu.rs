use crate::compute::backend::{ComputeBackend, DeviceType};

/// Native open CPU compute implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuBackend;

impl CpuBackend {
    /// Creates a new CPU compute backend.
    pub fn new() -> Self {
        Self
    }
}

impl ComputeBackend for CpuBackend {
    fn name(&self) -> &'static str {
        "Open-Source Native CPU Backend"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::CpuNative
    }

    fn vector_dot(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot = self.vector_dot(a, b);
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            // Using software sqrt or sqrtf
            dot / (norm_a.sqrt() * norm_b.sqrt())
        }
    }

    fn vector_add_scaled(&self, dest: &mut [f32], src: &[f32], scale: f32) {
        for (d, s) in dest.iter_mut().zip(src.iter()) {
            *d += s * scale;
        }
    }

    fn normalize(&self, vec: &mut [f32]) {
        let sum_sq: f32 = vec.iter().map(|x| x * x).sum();
        if sum_sq > 0.0 {
            let inv_norm = 1.0 / sum_sq.sqrt();
            for x in vec.iter_mut() {
                *x *= inv_norm;
            }
        }
    }
}
