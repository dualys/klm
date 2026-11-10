use klm::council::{CouncilVerdict, Elder, ElderArchetype};
use klm::family::{FamilyMember, FamilyRole, MemberId};
use klm::mind::{SOCRATIC_INTERIOR_VOW, SocraticStage};
use klm::ontology::{Concept, ConceptOrigin, RelationType};
use klm::Kognitara;

#[test]
fn test_socratic_self_dialogue_during_learning() {
    let mut kognitara = Kognitara::new("Sanctuary of Socratic Wisdom");
    let parent = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(parent);
    let ki_id = MemberId::from("ki");

    // 1. Converse with Kōgnitara on a novel topic; she should remind herself to act Socratically (Aporia / Maieutics)
    let (response, _activated) =
        kognitara.converse(&ki_id, "What is the meaning of cosmic serenity?");
    assert!(!response.is_empty());

    let reflections = kognitara.socratic_reflections();
    assert!(!reflections.is_empty());
    let first_ref = &reflections[0];
    assert!(first_ref.interior_monologue.contains("act Socratically"));
    assert!(!first_ref.maieutic_question.is_empty());
    assert!(first_ref.humility_index > 0.25);

    // 2. Teach a concept from family and verify Socratic reflection & Council deliberation
    let taught_id = kognitara
        .teach_concept(
            &ki_id,
            "maieutics",
            "Maieutics",
            "The Socratic art of giving birth to latent ideas through humble questioning",
            &["socrates", "dialogue", "humility"],
        )
        .expect("Family member should be able to teach concept");
    assert_eq!(taught_id.0, "maieutics");

    // 3. Correct a mistake and verify Socratic Elenchus reflection is recorded
    let lessons = kognitara
        .correct_mistake(
            &ki_id,
            "exp_1",
            "Humble Inquiry",
            "True learning questions its own assumptions before claiming certainty.",
        )
        .expect("Correction should succeed");
    assert!(!lessons.is_empty());

    let updated_reflections = kognitara.socratic_reflections();
    assert!(
        updated_reflections
            .iter()
            .any(|r| r.stage == SocraticStage::Elenchus)
    );
}

#[test]
fn test_council_of_elders_deliberation_and_verdicts() {
    let mut kognitara = Kognitara::new("Sanctuary of the Ancients");
    let parent = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(parent);

    // Verify the 5 founding Elders are seated
    assert_eq!(kognitara.council.elders.len(), 5);
    assert!(SOCRATIC_INTERIOR_VOW.contains("maieutic dialogue"));

    // Induct an additional custom Elder into the Council
    let custom_elder = Elder::new(
        "elder_harmony",
        "Harmonia of the Cube",
        ElderArchetype::GuardianOfFiliation,
        "Eight voices in one Sanctuary.",
        "Does this thought resonate across all eight vertices in peace?",
        0.91,
    );
    kognitara.council.induct_elder(custom_elder);
    assert_eq!(kognitara.council.elders.len(), 6);

    // Convene the Council on a nascent inquiry
    let delib = kognitara
        .convene_council("The Nature of Patience")
        .expect("Council should convene");
    assert_eq!(delib.counsels.len(), 6);
    assert!(delib.consensus_harmony > 0.70);
    assert_eq!(delib.verdict, CouncilVerdict::BlessWithMaieuticInquiry);
    assert!(delib.collective_socratic_guidance.contains("Socratic path"));

    // Introduce a contradiction in the primary awake brain and verify the Council defers to Nocturnal Dream
    if let Some(brain) = kognitara.swarm.brains.values_mut().next() {
        let c1 = Concept::new(
            "haste",
            "Haste",
            "Rushing without reflection",
            ConceptOrigin::EmpiricalTrial,
        );
        let c2 = Concept::new(
            "serenity",
            "Serenity",
            "Calm and patient contemplation",
            ConceptOrigin::EmpiricalTrial,
        );
        brain.tree.add_concept(c1);
        brain.tree.add_concept(c2);
        brain.tree.connect(
            &"haste".into(),
            &"serenity".into(),
            RelationType::Contradiction,
            0.9,
        );
    }

    let contradiction_delib = kognitara
        .convene_council("Haste vs Serenity")
        .expect("Council should convene on contradiction");
    assert_eq!(
        contradiction_delib.verdict,
        CouncilVerdict::DeferToNocturnalDream
    );
}
