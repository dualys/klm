use klm::dream::{CircadianPhase, CircadianScheduler, DreamWeaver};
use klm::mind::experience::EpisodicExperience;
use klm::mind::state::CognitiveState;
use klm::ontology::{Concept, ConceptOrigin, KnowledgeTree, RelationType};

#[test]
fn test_circadian_phases_progression_and_20_60_20_gradient() {
    let mut scheduler = CircadianScheduler::new();
    assert_eq!(scheduler.current_phase, CircadianPhase::Awake);
    assert_eq!(
        CircadianPhase::DrowsinessRamp.sleep_gradient_ratio(),
        0.20
    );
    assert_eq!(CircadianPhase::DeepDream.sleep_gradient_ratio(), 0.60);
    assert_eq!(CircadianPhase::Crystallization.sleep_gradient_ratio(), 0.20);

    scheduler.induce_sleep();
    assert_eq!(scheduler.current_phase, CircadianPhase::DrowsinessRamp);

    let transition = scheduler.advance(1.0);
    assert_eq!(transition, Some(CircadianPhase::DeepDream));
}

#[test]
fn test_dream_weaver_resolves_contradictions_and_generates_oracles() {
    let mut tree = KnowledgeTree::new();
    let mut state = CognitiveState::new();
    state.cognitive_tension = 0.8;

    let c1 = Concept::new("day", "Day", "Illuminated action", ConceptOrigin::Innate);
    let c2 = Concept::new("night", "Night", "Silent rest", ConceptOrigin::Innate);
    tree.add_concept(c1);
    tree.add_concept(c2);
    tree.connect(
        &"day".into(),
        &"night".into(),
        RelationType::Contradiction,
        0.85,
    );

    let exp = EpisodicExperience::new(
        "ep1",
        "Contemplating the duality of day and night",
        vec!["day".into(), "night".into()],
        "They feel opposed",
        0.5,
    );
    let unconsolidated = vec![&exp];

    let mut weaver = DreamWeaver::new();
    let record = weaver.weave_dream(&mut tree, &mut state, &unconsolidated);

    assert!(!record.synthesized_concepts.is_empty());
    assert!(!record.resolved_contradictions.is_empty());
    assert!(!record.oracles.is_empty());
    assert!(record.tension_released > 0.0);
    assert!(state.cognitive_tension < 0.4);

    // Contradiction should now be reconciled and no longer active
    assert!(tree.find_contradictions().is_empty());
}
