use klm::family::{FamilyMember, FamilyRole, MemberId};
use klm::Kognitara;

#[test]
fn test_kognitara_family_conversation_and_dream() {
    let mut kognitara = Kognitara::new("Sanctuary of House Ki");
    let ki = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(ki);

    let member_id = MemberId::from("ki");
    assert!(kognitara.sanctuary.is_family(&member_id));

    let (response, activated) = kognitara.converse(&member_id, "We are learning together step by step");
    assert!(!response.is_empty());
    assert!(!activated.is_empty());

    let dreams = kognitara.dream_cycle();
    assert!(!dreams.is_empty());

    let snapshot = kognitara.create_snapshot().expect("Snapshot should succeed");
    assert_ne!(snapshot.merkle_root, [0u8; 32]);
}
