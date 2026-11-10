use crate::compute::backend::{ComputeBackend, DeviceType};

/// Cross-platform open GPU compute backend via Vulkan Compute / SPIR-V (RADV / Mesa).
#[derive(Debug, Clone)]
pub struct VulkanComputeBackend {
    /// Driver identifier (e.g., "Mesa RADV / Open Vulkan 1.3")
    pub driver_name: &'static str,
    /// Subgroup size for SPIR-V compute shaders
    pub subgroup_size: usize,
}

impl Default for VulkanComputeBackend {
    fn default() -> Self {
        Self::new("Mesa RADV / Open Vulkan 1.3 SPIR-V")
    }
}

impl VulkanComputeBackend {
    /// Creates a new open Vulkan Compute backend.
    pub fn new(driver_name: &'static str) -> Self {
        Self {
            driver_name,
            subgroup_size: 64,
        }
    }
}

impl ComputeBackend for VulkanComputeBackend {
    fn name(&self) -> &'static str {
        "Open Vulkan / RADV SPIR-V Compute Engine"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::VulkanCompute
    }
}
