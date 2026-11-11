#[doc = "This module provides functionality related to council operations."]
pub mod assembly;
#[doc = "This module provides functionality related to elder operations."]
pub mod elder;

pub use assembly::{CouncilDeliberation, CouncilOfElders, CouncilVerdict};
pub use elder::{Elder, ElderArchetype, ElderCounsel, ElderId};
