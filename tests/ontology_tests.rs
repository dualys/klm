use klm::ontology::{Concept, ConceptId, ConceptOrigin, KnowledgeTree, RelationType};

#[test]
fn test_concept_organic_plasticity_and_humility() {
    let mut concept = Concept::new("kindness", "Kindness", "Gentle caring presence", ConceptOrigin::Innate);
    assert_eq!(concept.experience_count, 1);
    assert!(concept.confidence <= 0.35); // Initial humility
    assert!(concept.plasticity > 0.90);

    // Apply positive reinforcement experience
    concept.apply_experience(0.9, 0.8);
    assert_eq!(concept.experience_count, 2);
    assert!(concept.confidence > 0.25);
    assert!(concept.affective_valence > 0.0);
}

#[test]
fn test_knowledge_tree_spreading_activation() {
    let mut tree = KnowledgeTree::with_innate_seed();

    let family_id = ConceptId::from("family");
    tree.spread_activation(&family_id, 1.0, 2);

    let warmth = tree.get_concept(&ConceptId::from("warmth")).unwrap();
    assert!(warmth.current_activation > 0.0);
}

#[test]
fn test_tree_contradiction_detection() {
    let mut tree = KnowledgeTree::new();
    let c1 = Concept::new("order", "Order", "Predictable regularity", ConceptOrigin::Innate);
    let c2 = Concept::new("chaos", "Chaos", "Wild unbounded potential", ConceptOrigin::Innate);

    tree.add_concept(c1);
    tree.add_concept(c2);
    tree.connect(&"order".into(), &"chaos".into(), RelationType::Contradiction, 0.9);

    let contradictions = tree.find_contradictions();
    assert_eq!(contradictions.len(), 1);
    assert_eq!(contradictions[0].1, "Order");
    assert_eq!(contradictions[0].3, "Chaos");
}
