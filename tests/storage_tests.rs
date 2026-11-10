use jinshu::storage::StorageEngine;
use klm::mind::state::CognitiveState;
use klm::ontology::{ConceptId, KnowledgeTree};
use klm::storage::{
    CognitiveOcean, CognitiveSnapshot, CubeFsVault, KpackRecipe, MemoryBlockDevice,
};

#[test]
fn test_kpack_native_opcodes_and_blake3_integrity() {
    // 1. Hybrid Lit + RLE encoding
    let original = b"Kognitara Sovereign Mind AAAAAAAAAAAAAAAA Open Hardware";
    let recipe = KpackRecipe::encode(original);
    let decoded = recipe.decode().expect("Decoding should succeed");
    assert_eq!(&decoded[..], original);

    // 2. Native XOR recipe
    let xor_recipe = KpackRecipe::encode_xor(original, 0x5A);
    let xor_decoded = xor_recipe.decode().expect("XOR decoding should succeed");
    assert_eq!(&xor_decoded[..], original);

    // 3. Native Dict recipe
    let dict = b"KOGNITARAMENTYS!";
    let dict_data = b"AMENTYS!KOGNITAR";
    let dict_recipe =
        KpackRecipe::encode_dict(dict_data, 8, dict).expect("Dict encoding should succeed");
    let dict_decoded = dict_recipe
        .decode_with_reference(Some(dict))
        .expect("Dict decoding should succeed");
    assert_eq!(&dict_decoded[..], dict_data);

    // 4. Native Seed + Patch recipe
    let seed_data = b"Short sovereign seed test";
    let seed_recipe = KpackRecipe::encode_seed(seed_data, 1337);
    let seed_decoded = seed_recipe.decode().expect("Seed decoding should succeed");
    assert_eq!(&seed_decoded[..], seed_data);
}

#[test]
fn test_cognitive_snapshot_full_and_delta_roundtrip() {
    let mut tree = KnowledgeTree::with_innate_seed();
    let mut state = CognitiveState::new();

    let base_snapshot =
        CognitiveSnapshot::create(&tree, &state, 1).expect("Base snapshot should succeed");
    let (restored_tree, restored_state) =
        base_snapshot.restore().expect("Restoration should succeed");

    assert_eq!(restored_tree.concepts.len(), tree.concepts.len());
    assert_eq!(
        restored_state.working_memory.len(),
        state.working_memory.len()
    );

    // Modify state and create a Delta snapshot
    state.focus_on(ConceptId::from("family"), 7);
    if let Some(c) = tree.get_concept_mut(&"family".into()) {
        c.apply_experience(0.95, 0.9);
    }

    let delta_snapshot = CognitiveSnapshot::create_delta(&base_snapshot, &tree, &state, 2)
        .expect("Delta snapshot creation should succeed");
    assert_eq!(
        delta_snapshot.parent_merkle_root,
        Some(base_snapshot.merkle_root)
    );

    let (delta_tree, delta_state) = delta_snapshot
        .restore_with_parent(&base_snapshot)
        .expect("Delta snapshot restoration should succeed");
    assert_eq!(delta_tree.concepts.len(), tree.concepts.len());
    assert_eq!(delta_state.working_memory.len(), 1);
}

#[test]
fn test_jinshu_cognitive_ocean_and_cubefs_vault() {
    let tree = KnowledgeTree::with_innate_seed();
    let family = tree.get_concept(&"family".into()).unwrap();

    // 1. Jinshu 4096-byte TrieNode & NounIndex persistence
    let nvme = MemoryBlockDevice::new();
    let mut storage_engine = StorageEngine::new(128, &nvme);
    let mut ocean = CognitiveOcean::new();

    let summary = ocean
        .crystallize_concept(family, &mut storage_engine)
        .expect("Jinshu crystallization should succeed");
    assert_eq!(summary.lba, 8);
    assert_eq!(nvme.allocated_pages(), 1);

    let recalled = ocean
        .recall_bytes(&summary.noun_bytes)
        .expect("Recall from Jinshu TrieNode should succeed");
    let decoded_concept: klm::ontology::Concept =
        serde_json::from_slice(&recalled).expect("Valid concept JSON");
    assert_eq!(decoded_concept.id, family.id);

    // 2. CubeFS Vault Checkpoint, KpackReader & CoW Nucleator
    let mut vault = CubeFsVault::new();
    let cp = vault.commit_kpack_extent(kpack::Opcode::Rle, 64, &[b'A'], 64, summary.noun_bytes);
    assert_eq!(cp.seq, 1);

    let section = vault
        .read_section(0, 64)
        .expect("CubeFS KpackReader should reconstruct section");
    assert_eq!(&section[..], &[b'A'; 64]);

    let page = [0x11u8; 4096];
    let (mutated_page, ejected) =
        CubeFsVault::cow_mutate_page(&page, 0, 0x99).expect("CoW mutation should succeed");
    assert_eq!(mutated_page[0], 0x99);
    assert_eq!(mutated_page[1], 0x11);
    assert_eq!(ejected, 0x11);
}
