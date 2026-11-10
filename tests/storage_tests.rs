use klm::mind::state::CognitiveState;
use klm::ontology::KnowledgeTree;
use klm::storage::{CognitiveSnapshot, KpackRecipe};

#[test]
fn test_kpack_recipe_encoding_and_blake3_integrity() {
    let original = b"Kognitara Sovereign Learning Mind without lock-in";
    let recipe = KpackRecipe::encode(original);

    let decoded = recipe.decode().expect("Decoding should succeed");
    assert_eq!(&decoded[..], original);
}

#[test]
fn test_cognitive_snapshot_roundtrip() {
    let tree = KnowledgeTree::with_innate_seed();
    let state = CognitiveState::new();

    let snapshot = CognitiveSnapshot::create(&tree, &state, 42).expect("Snapshot creation should succeed");
    let (restored_tree, restored_state) = snapshot.restore().expect("Restoration should succeed");

    assert_eq!(restored_tree.concepts.len(), tree.concepts.len());
    assert_eq!(restored_state.working_memory.len(), state.working_memory.len());
}
