use crate::compute::backend::{ComputeBackend, DeviceType};

/// Native open CPU compute implementation (compatible with bare-metal Amentys OS `no_std`).
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
        "Open-Source Native CPU Backend (Amentys OS / SIMD)"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::CpuNative
    }
}
