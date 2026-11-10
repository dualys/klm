use klm::compute::{AmdComputeBackend, ComputeBackend, VulkanComputeBackend};
use klm::ontology::{Concept, ConceptId, ConceptOrigin, EMBEDDING_DIM, KnowledgeTree, RelationType};

#[test]
fn test_concept_organic_plasticity_humility_and_noun() {
    let mut concept = Concept::new(
        "kindness",
        "Kindness",
        "Gentle caring presence",
        ConceptOrigin::Innate,
    );
    assert_eq!(concept.experience_count, 1);
    assert!(concept.confidence <= 0.35); // Initial humility
    assert!(concept.plasticity > 0.90);
    assert_eq!(concept.embedding.len(), EMBEDDING_DIM);
    assert!(!concept.noun().is_null());
    assert!(concept.epistemic_doubt() > 0.5);

    // Apply positive reinforcement experience
    concept.apply_experience(0.9, 0.8);
    assert_eq!(concept.experience_count, 2);
    assert!(concept.confidence > 0.25);
    assert!(concept.affective_valence > 0.0);
}

#[test]
fn test_knowledge_tree_spreading_activation_and_reasoning_path() {
    let mut tree = KnowledgeTree::with_innate_seed();

    let family_id = ConceptId::from("family");
    tree.spread_activation(&family_id, 1.0, 2);

    let warmth = tree.get_concept(&ConceptId::from("warmth")).unwrap();
    assert!(warmth.current_activation > 0.0);

    // Connect warmth to dream to create a multi-hop path: family -> warmth -> dream -> learning
    tree.connect(
        &"warmth".into(),
        &"dream".into(),
        RelationType::Associative,
        0.8,
    );
    let path = tree
        .find_reasoning_path(&"family".into(), &"learning".into(), 4)
        .expect("Multi-hop reasoning path should exist");
    assert!(!path.is_empty());
    assert_eq!(path.first().unwrap().0, ConceptId::from("family"));
    assert_eq!(path.last().unwrap().2, ConceptId::from("learning"));
}

#[test]
fn test_semantic_search_and_conceptual_fusion_on_open_compute() {
    let mut tree = KnowledgeTree::with_innate_seed();
    let amd = AmdComputeBackend::default();
    let vk = VulkanComputeBackend::default();
    assert!(amd.is_open_source_sovereign());
    assert!(vk.is_open_source_sovereign());

    let results = tree.semantic_search("Family sanctuary trust love", 3, &amd);
    assert!(!results.is_empty());
    assert_eq!(results[0].0, ConceptId::from("family"));

    let fused_id = tree
        .fuse_concepts(&"learning".into(), &"humility".into(), "Humble Wisdom")
        .expect("Fusion should succeed");
    let fused = tree.get_concept(&fused_id).unwrap();
    assert_eq!(fused.label, "Humble Wisdom");
    assert_eq!(fused.embedding.len(), EMBEDDING_DIM);
}

#[test]
fn test_tree_contradiction_detection() {
    let mut tree = KnowledgeTree::new();
    let c1 = Concept::new(
        "order",
        "Order",
        "Predictable regularity",
        ConceptOrigin::Innate,
    );
    let c2 = Concept::new(
        "chaos",
        "Chaos",
        "Wild unbounded potential",
        ConceptOrigin::Innate,
    );

    tree.add_concept(c1);
    tree.add_concept(c2);
    tree.connect(
        &"order".into(),
        &"chaos".into(),
        RelationType::Contradiction,
        0.9,
    );

    let contradictions = tree.find_contradictions();
    assert_eq!(contradictions.len(), 1);
    assert_eq!(contradictions[0].1, "Order");
    assert_eq!(contradictions[0].3, "Chaos");
}
