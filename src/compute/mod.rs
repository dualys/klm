#[doc = "This module provides functionality related to compute operations."]
pub mod amd;
#[doc = "This module provides functionality related to backend operations."]
pub mod backend;
#[doc = "This module provides functionality related to CPU operations."]
pub mod cpu;
#[doc = "This module provides functionality related to mathematical operations."]
pub mod math;
#[doc = "This module provides functionality related to Vulkan operations."]
pub mod vulkan;

pub use amd::AmdComputeBackend;
pub use backend::{ComputeBackend, DeviceType};
pub use cpu::CpuBackend;
pub use math::{abs_f32, exp_f32, sqrt_f32};
pub use vulkan::VulkanComputeBackend;
