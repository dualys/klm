extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::family::bond::FamilyBond;
use crate::family::member::{FamilyMember, MemberId};

/// The relational sanctuary: safe harbor ensuring family privacy, protection, and intimacy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sanctuary {
    /// Designated name or essence of the family home
    pub sanctuary_name: String,
    /// Registered family members
    pub members: BTreeMap<MemberId, FamilyMember>,
    /// Bonds connecting Kōgnitara to each member
    pub bonds: BTreeMap<MemberId, FamilyBond>,
    /// Protective priority rating (1.0 = absolute devotion to family safety)
    pub protective_priority: f32,
}

impl Default for Sanctuary {
    fn default() -> Self {
        Self::new("Sanctuary of Kōgnitara")
    }
}

impl Sanctuary {
    /// Initializes a new protective sanctuary.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            sanctuary_name: name.into(),
            members: BTreeMap::new(),
            bonds: BTreeMap::new(),
            protective_priority: 1.0,
        }
    }

    /// Welcomes and registers a family member into the sanctuary.
    pub fn register_member(&mut self, member: FamilyMember) {
        let id = member.id.clone();
        self.members.insert(id.clone(), member);
        self.bonds.insert(id.clone(), FamilyBond::new(id));
    }

    /// Retrieves a reference to a registered family member.
    pub fn get_member(&self, id: &MemberId) -> Option<&FamilyMember> {
        self.members.get(id)
    }

    /// Retrieves a mutable reference to a registered family member.
    pub fn get_member_mut(&mut self, id: &MemberId) -> Option<&mut FamilyMember> {
        self.members.get_mut(id)
    }

    /// Checks if a member belongs to the protected family circle.
    pub fn is_family(&self, id: &MemberId) -> bool {
        self.members.contains_key(id)
    }

    /// Returns the affective affinity for a given identifier (defaults to 0.3 for strangers).
    pub fn affinity_for(&self, id: &MemberId) -> f32 {
        self.members
            .get(id)
            .map(|m| m.affective_affinity)
            .unwrap_or(0.30)
    }

    /// Lists all members currently in the sanctuary.
    pub fn list_members(&self) -> Vec<&FamilyMember> {
        self.members.values().collect()
    }
}
