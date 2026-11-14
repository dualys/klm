use klm::Kognitara;
use klm::family::{FamilyMember, FamilyRole, MemberId, TrustGate};

#[test]
fn test_kognitara_family_teaching_mistakes_curiosity_and_dream() {
    let mut kognitara = Kognitara::new("Sanctuary of House Ki");
    let ki = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(ki);

    let member_id = MemberId::from("ki");
    let stranger_id = MemberId::from("stranger");
    assert!(kognitara.sanctuary.is_family(&member_id));
    assert!(!kognitara.sanctuary.is_family(&stranger_id));

    assert!(matches!(
        kognitara.sanctuary.evaluate_trust(&member_id),
        TrustGate::FamilyCircle { .. }
    ));
    assert!(matches!(
        kognitara.sanctuary.evaluate_trust(&stranger_id),
        TrustGate::ExternalVisitor { .. }
    ));

    // Family member teaches a core concept
    let taught_id = kognitara
        .teach_concept(
            &member_id,
            "resilience",
            "Resilience",
            "Growing stronger through gentle perseverance",
            &["strength", "patience"],
        )
        .expect("Family member should be allowed to teach");
    assert_eq!(taught_id.0, "resilience");
    assert_eq!(kognitara.sanctuary.values.len(), 1);

    // Stranger cannot anchor filiation concepts
    assert!(
        kognitara
            .teach_concept(&stranger_id, "dogma", "Dogma", "Rigid rule", &[])
            .is_err()
    );

    // Novel conversation triggers nascent concept and curiosity question
    let (response, activated) =
        kognitara.converse(&member_id, "Exploring mysterious cosmic harmonies");
    assert!(!response.is_empty());
    assert!(!activated.is_empty());

    let questions = kognitara.pending_questions();
    assert!(!questions.is_empty());

    // Gentle correction of a mistake
    let lessons = kognitara
        .correct_mistake(
            &member_id,
            "exp_1",
            "Cosmic Resonance",
            "Harmony emerges from listening before speaking",
        )
        .expect("Correction should succeed");
    assert!(!lessons.is_empty());
    assert!(lessons[0].humility_gained > 0.0);
    let dreams = kognitara.dream_cycle(|_, _| {});
    assert!(!dreams.is_empty());
    assert!(!kognitara.dream_oracles().is_empty());

    let snapshot = kognitara
        .create_snapshot()
        .expect("Snapshot should succeed");
    assert_ne!(snapshot.merkle_root, [0u8; 32]);
}
