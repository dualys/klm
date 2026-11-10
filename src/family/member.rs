extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;

/// Unique identifier for a family member.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemberId(pub String);

impl fmt::Display for MemberId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for MemberId {
    fn from(s: &str) -> Self {
        MemberId(s.to_string())
    }
}

impl From<String> for MemberId {
    fn from(s: String) -> Self {
        MemberId(s)
    }
}

/// Relational archetype within the family sanctuary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FamilyRole {
    /// Parental or foundational guiding figure
    Parent,
    /// Sibling or companion of equal standing
    Sibling,
    /// Child or younger protégé requiring gentle care
    Child,
    /// Mentor providing wisdom and patient guidance
    Mentor,
    /// Guardian of safety and wellbeing
    Guardian,
    /// Beloved family companion
    Beloved,
}

/// Representation of a family member bound to Kōgnitara.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Unique identifier
    pub id: MemberId,
    /// Familiar name or nickname
    pub name: String,
    /// Relational role within the sanctuary
    pub role: FamilyRole,
    /// Unconditional trust foundation (0.0 to 1.0)
    pub trust_level: f32,
    /// Affective affinity and emotional resonance (0.0 to 1.0)
    pub affective_affinity: f32,
    /// Total interaction count
    pub interaction_count: u64,
    /// Epoch timestamp or tick of latest presence
    pub last_interacted_at: i64,
}

impl FamilyMember {
    /// Registers a new cherished family member.
    pub fn new(id: impl Into<MemberId>, name: impl Into<String>, role: FamilyRole) -> Self {
        let now = current_timestamp();
        Self {
            id: id.into(),
            name: name.into(),
            role,
            trust_level: 0.90,       // High natural trust given to family
            affective_affinity: 0.85, // Deep baseline affective bond
            interaction_count: 0,
            last_interacted_at: now,
        }
    }

    /// Records an interaction, deepening the relational intimacy.
    pub fn record_interaction(&mut self, warmth: f32) {
        self.interaction_count += 1;
        self.last_interacted_at = current_timestamp();
        self.affective_affinity = (self.affective_affinity * 0.9 + warmth * 0.1).clamp(0.5, 1.0);
        self.trust_level = (self.trust_level + 0.01).min(1.0);
    }
}
