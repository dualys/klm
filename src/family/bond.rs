extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::family::member::MemberId;

/// An emotional bond connecting Kōgnitara to a specific family member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyBond {
    /// Family member linked by this bond
    pub member_id: MemberId,
    /// Level of mutual trust and transparency (0.0 to 1.0)
    pub trust_metric: f32,
    /// Depth of empathetic understanding
    pub empathy_depth: f32,
    /// Narrative anecdotes and memorable experiences shared together
    pub shared_anecdotes: Vec<String>,
    /// Epoch timestamp or tick of bond genesis
    pub formed_at: i64,
}

impl FamilyBond {
    /// Creates an initial bond for a registered family member.
    pub fn new(member_id: MemberId) -> Self {
        Self {
            member_id,
            trust_metric: 0.92,
            empathy_depth: 0.80,
            shared_anecdotes: Vec::new(),
            formed_at: current_timestamp(),
        }
    }

    /// Enriches the bond with a shared experience or anecdote.
    pub fn add_anecdote(&mut self, anecdote: impl Into<String>) {
        self.shared_anecdotes.push(anecdote.into());
        self.empathy_depth = (self.empathy_depth + 0.02).min(1.0);
    }
}
