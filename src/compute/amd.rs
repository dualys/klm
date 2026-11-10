use crate::compute::backend::{ComputeBackend, DeviceType};

/// Sovereign open compute backend for AMD Radeon / Instinct hardware (ROCm / HIP).
/// Strictly avoids proprietary CUDA stacks in favor of open AMD RDNA/CDNA wavefront execution.
#[derive(Debug, Clone)]
pub struct AmdComputeBackend {
    /// Target GPU architecture identifier (e.g., "gfx1100", "gfx90a", "gfx1030")
    pub architecture: &'static str,
    /// Wavefront workgroup size (64 for CDNA/GCN, 32/64 for RDNA)
    pub workgroup_size: usize,
    /// Number of logical compute units simulated or bound
    pub compute_units: usize,
}

impl Default for AmdComputeBackend {
    fn default() -> Self {
        Self::new("AMD RDNA3/CDNA3 (gfx1100 / ROCm Open Stack)")
    }
}

impl AmdComputeBackend {
    /// Initializes an AMD open compute profile.
    pub fn new(arch: &'static str) -> Self {
        Self {
            architecture: arch,
            workgroup_size: 64,
            compute_units: 60,
        }
    }
}

impl ComputeBackend for AmdComputeBackend {
    fn name(&self) -> &'static str {
        "Sovereign AMD ROCm / HIP Wavefront Compute Engine"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::AmdRocm
    }
}
