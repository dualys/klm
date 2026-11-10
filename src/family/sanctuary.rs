extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::clock::current_timestamp;
use crate::family::bond::FamilyBond;
use crate::family::member::{FamilyMember, FamilyRole, MemberId};

/// A foundational ethical or relational value entrusted to Kōgnitara by her family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyValue {
    /// Short name of the value (e.g., "Patience", "Sovereignty", "Kindness")
    pub title: String,
    /// Guiding description or principle
    pub principle: String,
    /// Family member who transmitted this value
    pub entrusted_by: MemberId,
    /// Timestamp or logical tick of transmission
    pub created_at: i64,
}

/// Trust evaluation outcome when an interlocutor interacts with the Sanctuary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrustGate {
    /// Recognized member of the protected family circle
    FamilyCircle {
        role: FamilyRole,
        trust_level: f32,
        affective_affinity: f32,
    },
    /// Unregistered external interlocutor (treated with polite caution and privacy boundaries)
    ExternalVisitor { caution_level: f32 },
}

/// The relational sanctuary: safe harbor ensuring family privacy, protection, and intimacy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sanctuary {
    /// Designated name or essence of the family home
    pub sanctuary_name: String,
    /// Registered family members
    pub members: BTreeMap<MemberId, FamilyMember>,
    /// Bonds connecting Kōgnitara to each member
    pub bonds: BTreeMap<MemberId, FamilyBond>,
    /// Core family values and traditions passed down to Kōgnitara
    pub values: Vec<FamilyValue>,
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
            values: Vec::new(),
            protective_priority: 1.0,
        }
    }

    /// Welcomes and registers a family member into the sanctuary.
    pub fn register_member(&mut self, member: FamilyMember) {
        let id = member.id.clone();
        self.members.insert(id.clone(), member);
        self.bonds.insert(id.clone(), FamilyBond::new(id));
    }

    /// Entrusts a lasting family value or tradition to Kōgnitara.
    pub fn entrust_value(
        &mut self,
        member_id: &MemberId,
        title: impl Into<String>,
        principle: impl Into<String>,
    ) -> bool {
        if !self.is_family(member_id) {
            return false;
        }
        self.values.push(FamilyValue {
            title: title.into(),
            principle: principle.into(),
            entrusted_by: member_id.clone(),
            created_at: current_timestamp(),
        });
        if let Some(bond) = self.bonds.get_mut(member_id) {
            bond.trust_metric = (bond.trust_metric + 0.02).min(1.0);
        }
        true
    }

    /// Records a shared family memory or anecdote on the relational bond.
    pub fn record_shared_anecdote(
        &mut self,
        member_id: &MemberId,
        anecdote: impl Into<String>,
    ) -> bool {
        if let Some(bond) = self.bonds.get_mut(member_id) {
            bond.add_anecdote(anecdote);
            true
        } else {
            false
        }
    }

    /// Evaluates the relational trust gate for an interlocutor ID.
    pub fn evaluate_trust(&self, id: &MemberId) -> TrustGate {
        if let Some(m) = self.members.get(id) {
            TrustGate::FamilyCircle {
                role: m.role.clone(),
                trust_level: m.trust_level,
                affective_affinity: m.affective_affinity,
            }
        } else {
            TrustGate::ExternalVisitor {
                caution_level: 0.75,
            }
        }
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
