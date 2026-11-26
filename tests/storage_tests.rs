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

#[cfg(all(feature = "std", unix))]
mod registry_tests {
    use klm::storage::KpackRecipe;
    use klm::storage::registry::{compact_registry, create_tag, delete_tag, export_tag, read_tags};
    use std::collections::BTreeMap;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> io::Result<Self> {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("klm-registry-test-{}-{nonce}", std::process::id()));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }

        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn append_snapshot(path: &Path, state: &[u8]) -> io::Result<()> {
        let recipe = KpackRecipe::encode(state);
        let line = serde_json::to_string(&recipe).map_err(io::Error::other)?;
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{line}")
    }

    fn read_states(path: &Path) -> io::Result<Vec<Vec<u8>>> {
        fs::read_to_string(path)?
            .lines()
            .map(|line| {
                let recipe: KpackRecipe = serde_json::from_str(line).map_err(io::Error::other)?;
                recipe.decode().map_err(io::Error::other)
            })
            .collect()
    }

    #[test]
    fn compaction_keeps_latest_and_tagged_snapshots_and_migrates_legacy_tags() {
        let dir = TestDir::new().unwrap();
        let registry = dir.path("data");
        let tags_path = dir.path("tags.json");

        for state in [
            b"one".as_slice(),
            b"two".as_slice(),
            b"three".as_slice(),
            b"four".as_slice(),
            b"latest".as_slice(),
        ] {
            append_snapshot(&registry, state).unwrap();
        }
        fs::write(&tags_path, r#"{"release":2,"checkpoint":4}"#).unwrap();
        let original_size = fs::metadata(&registry).unwrap().len();

        let report = compact_registry(&registry, &tags_path).unwrap();

        assert_eq!(report.records_before, 5);
        assert_eq!(report.records_after, 3);
        assert!(report.bytes_after < original_size);
        assert_eq!(
            read_states(&registry).unwrap(),
            vec![b"two".to_vec(), b"four".to_vec(), b"latest".to_vec()]
        );

        let tags: BTreeMap<String, String> =
            serde_json::from_slice(&fs::read(&tags_path).unwrap()).unwrap();
        assert_eq!(tags, read_tags(&registry, &tags_path).unwrap());
        assert_eq!(tags.len(), 2);

        let export = dir.path("release.kpack");
        assert_eq!(
            export_tag(&registry, &tags_path, "checkpoint", &export).unwrap(),
            2
        );
        assert_eq!(
            read_states(&export).unwrap(),
            vec![b"two".to_vec(), b"four".to_vec()]
        );
    }

    #[test]
    fn creating_and_deleting_tags_uses_stable_snapshot_ids() {
        let dir = TestDir::new().unwrap();
        let registry = dir.path("data");
        let tags_path = dir.path("tags.json");
        append_snapshot(&registry, b"current state").unwrap();

        let id = create_tag(&registry, &tags_path, "before-test").unwrap();
        assert_eq!(id.len(), 64);
        assert_eq!(
            read_tags(&registry, &tags_path).unwrap().get("before-test"),
            Some(&id)
        );
        assert!(delete_tag(&registry, &tags_path, "before-test").unwrap());
        assert!(read_tags(&registry, &tags_path).unwrap().is_empty());
        assert_eq!(
            compact_registry(&registry, &tags_path)
                .unwrap()
                .records_after,
            1
        );
    }

    #[test]
    fn compaction_refuses_malformed_registry_without_changing_files() {
        let dir = TestDir::new().unwrap();
        let registry = dir.path("data");
        let tags_path = dir.path("tags.json");
        append_snapshot(&registry, b"keep me").unwrap();
        use std::io::Write;
        let mut file = fs::OpenOptions::new().append(true).open(&registry).unwrap();
        writeln!(file, "not a recipe").unwrap();
        fs::write(&tags_path, r#"{"release":1}"#).unwrap();
        let original_registry = fs::read(&registry).unwrap();
        let original_tags = fs::read(&tags_path).unwrap();

        assert!(compact_registry(&registry, &tags_path).is_err());
        assert_eq!(fs::read(&registry).unwrap(), original_registry);
        assert_eq!(fs::read(&tags_path).unwrap(), original_tags);
    }

    #[test]
    fn cli_uses_configured_storage_for_compaction_and_research() {
        let dir = TestDir::new().unwrap();
        let root = dir.path("storage with spaces");
        fs::create_dir(&root).unwrap();
        let registry = root.join("data");
        append_snapshot(&registry, b"tagged").unwrap();
        append_snapshot(&registry, b"unwanted").unwrap();
        append_snapshot(&registry, b"latest").unwrap();
        fs::write(root.join("tags.json"), r#"{"release":1}"#).unwrap();

        let compact = std::process::Command::new(env!("CARGO_BIN_EXE_klm"))
            .env("KOGNITARA_PATH", &root)
            .arg("compact")
            .output()
            .unwrap();
        assert!(compact.status.success(), "{:?}", compact);
        assert_eq!(
            read_states(&registry).unwrap(),
            vec![b"tagged".to_vec(), b"latest".to_vec()]
        );
        assert!(root.join("registry.lock").is_file());
        assert_eq!(
            read_tags(&registry, &root.join("tags.json")).unwrap().len(),
            1
        );

        let research_root = dir.path("research root");
        fs::create_dir(&research_root).unwrap();
        let research = std::process::Command::new(env!("CARGO_BIN_EXE_klm"))
            .env("KOGNITARA_PATH", &research_root)
            .arg("research-cycle")
            .output()
            .unwrap();
        assert!(!research.status.success());
        assert!(research_root.join("research/worker.lock").is_file());
        assert!(String::from_utf8_lossy(&research.stderr).contains("Cannot open registry"));

        let empty = std::process::Command::new(env!("CARGO_BIN_EXE_klm"))
            .env("KOGNITARA_PATH", "")
            .arg("compact")
            .output()
            .unwrap();
        assert!(!empty.status.success());
        assert!(String::from_utf8_lossy(&empty.stderr).contains("KOGNITARA_PATH cannot be empty"));
    }
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
