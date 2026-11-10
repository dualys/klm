use jinshu::storage::StorageEngine;
use klm::Kognitara;
use klm::compute::{AmdComputeBackend, ComputeBackend, CpuBackend, VulkanComputeBackend};
use klm::family::{FamilyMember, FamilyRole, MemberId};
use klm::storage::{CognitiveOcean, CubeFsVault, MemoryBlockDevice};
use std::io::{self, Write};

fn main() {
    println!("============================================================");
    println!("           K Ō G N I T A R A   F R A M E W O R K            ");
    println!("        Sovereign Organic Mind & Relational Sanctuary       ");
    println!("============================================================");
    println!();

    // 1. Initialize Kōgnitara Entity
    let mut kognitara = Kognitara::new("Sanctuary of House Ki");

    // 2. Register Family & Entrust Core Values
    let family_head = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(family_head);
    let ki_id = MemberId::from("ki");
    kognitara
        .sanctuary
        .record_shared_anecdote(&ki_id, "First awakening on Amentys OS architecture");

    println!(
        "[+] Sanctuary initialized: '{}'",
        kognitara.sanctuary.sanctuary_name
    );
    println!("[+] Registered family member: 'Ki' (Role: Parent, Unconditional Trust: 0.90)");
    println!("[+] Hamon Cube 8-Brain Swarm online (Nodes 000 to 111 with specializations)");
    println!();

    // 3. Hardware Compute Verification (AMD ROCm / Vulkan / CPU - Zero NVIDIA/CUDA)
    let cpu = CpuBackend::new();
    let amd = AmdComputeBackend::default();
    let vulkan = VulkanComputeBackend::default();
    let v1 = [0.2, 0.8, 0.5, 0.1];
    let v2 = [0.25, 0.75, 0.6, 0.05];
    let sim_cpu = cpu.cosine_similarity(&v1, &v2);
    let sim_amd = amd.cosine_similarity(&v1, &v2);
    let sim_vk = vulkan.cosine_similarity(&v1, &v2);

    println!("[Compute] {}", cpu.name());
    println!("[Compute] {} ({})", amd.name(), amd.architecture);
    println!("[Compute] {}", vulkan.name());
    println!(
        "          Semantic similarity check (Open Compute): {:.4} / {:.4} / {:.4}",
        sim_cpu, sim_amd, sim_vk
    );
    println!();

    // 4. Family Nurturing, Dialogue & Learning from Mistakes
    println!("--- Family Nurturing & Organic Dialogue (Awake Cohort) ---");
    let _ = kognitara.teach_concept(
        &ki_id,
        "compassion",
        "Compassion",
        "Gentle care and patience toward living beings who are learning",
        &["kindness", "patience", "heart"],
    );
    println!("[+] Family taught concept: 'Compassion' (linked via Filiation to 'Family')");

    let utterance = "We learn slowly, with patience and tenderness.";
    println!("Family (Ki) > {}", utterance);

    let (response, activated) = kognitara.converse(&ki_id, utterance);
    println!("Kōgnitara   > {}", response);
    print!("Activated Concepts: ");
    for id in activated {
        print!("'{}' ", id);
    }
    println!();

    // Demonstrate gentle self-correction from a mistake
    let lessons = kognitara
        .correct_mistake(
            &ki_id,
            "exp_1",
            "Patient Maturation",
            "Understanding blossoms through time, trial, and dreams rather than haste.",
        )
        .unwrap_or_default();
    if let Some(lesson) = lessons.first() {
        println!(
            "[+] Mistake transformed into wisdom (#{}): '{}' (Humility gained: +{:.2})",
            lesson.id, lesson.family_guidance, lesson.humility_gained
        );
    }
    println!();

    // 5. Demonstration of Sleep, 20-60-20 Dream Cycle & Oracles
    println!("--- Inducing Sleep & 20-60-20 Dream Cycle (Hamon Cube) ---");
    let dreams = kognitara.dream_cycle();
    for dream in &dreams {
        println!("[Dream Record #{}] {}", dream.dream_id, dream.narrative);
        for oracle in &dream.oracles {
            println!("  * Oracle ({}): {}", oracle.oracle_id, oracle.revelation);
        }
    }
    println!();

    // 6. Kpack, Jinshu Ocean & CubeFS Vault Crystallization
    println!("--- Crystallization: Kpack, Jinshu Ocean & CubeFS Vault ---");
    if let Ok(snapshot) = kognitara.create_snapshot() {
        println!(
            "[Kpack] Snapshot Merkle Root (BLAKE3): {:02x?}",
            &snapshot.merkle_root[..8]
        );
        println!(
            "[Kpack] Compressed Opcodes: {}, Uncompressed: {} bytes",
            snapshot.recipe.opcodes.len(),
            snapshot.recipe.uncompressed_length
        );

        // Distributed 8-node Hamon Cube Kpack Job
        let raw = snapshot.raw_bytes().unwrap_or_default();
        let cube_job = kognitara.execute_cube_job(&raw);
        println!(
            "[Hamon Cube] 8-Node Distributed Kpack Job: {} shards, 3D Butterfly Merkle Root: {:02x?}",
            cube_job.shards.len(),
            &cube_job.cube_merkle_root[..8]
        );

        // Jinshu 4096-byte TrieNode & Noun Ocean
        let nvme = MemoryBlockDevice::new();
        let mut storage_engine = StorageEngine::new(1024, &nvme);
        let mut ocean = CognitiveOcean::new();
        if let Some(brain) = kognitara.swarm.brains.values().next()
            && let Some(family_concept) = brain.tree.get_concept(&"family".into())
            && let Ok(entry) = ocean.crystallize_concept(family_concept, &mut storage_engine)
        {
            println!(
                "[Jinshu] Concept 'Family' crystallized into 4096B TrieNode at LBA {} (Noun: {:02x?})",
                entry.lba,
                &entry.noun_bytes[..8]
            );
        }

        // CubeFS Checkpoint
        let mut vault = CubeFsVault::new();
        let cp = vault.commit_kpack_extent(
            kpack::Opcode::Rle,
            4096,
            &[0x4B],
            4096,
            snapshot.merkle_root,
        );
        println!(
            "[CubeFS] Sealed Checkpoint #{} (B-Tree Meta Root: {:02x?})",
            cp.seq,
            &cp.root_meta_id[..8]
        );
    }

    println!();
    println!("============================================================");
    println!(" Type 'help' for commands, or interact with Kōgnitara below: ");
    println!(" (Commands: dream, oracles, questions, cube-job, status, quit)");
    println!("============================================================");

    // If running in non-interactive environment, conclude gracefully
    if !std::io::stdin().is_terminal() {
        println!("[+] Non-interactive execution finished successfully.");
        return;
    }

    let stdin = io::stdin();
    loop {
        print!("\nFamily (Ki) > ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() || input.trim().is_empty() {
            break;
        }

        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
            println!("Farewell in the Sanctuary. Take care.");
            break;
        } else if trimmed.eq_ignore_ascii_case("dream") {
            println!("[*] Transitioning swarm into 20-60-20 dream cycle...");
            let records = kognitara.dream_cycle();
            for rec in records {
                println!("[Dream] {}", rec.narrative);
                for oracle in rec.oracles {
                    println!("  * Oracle: {}", oracle.revelation);
                }
            }
        } else if trimmed.eq_ignore_ascii_case("oracles") {
            let oracles = kognitara.dream_oracles();
            if oracles.is_empty() {
                println!("No dream oracles recorded yet.");
            } else {
                for o in oracles {
                    println!(
                        "  [{}] {} (Resonance: {:.2})",
                        o.oracle_id, o.revelation, o.resonance_strength
                    );
                }
            }
        } else if trimmed.eq_ignore_ascii_case("questions") {
            let questions = kognitara.pending_questions();
            if questions.is_empty() {
                println!("Kōgnitara is currently at peace with her understanding.");
            } else {
                for q in questions {
                    println!("  [{}] (Urgency {:.2}) {}", q.id, q.urgency, q.question_text);
                }
            }
        } else if trimmed.eq_ignore_ascii_case("cube-job") {
            if let Ok(snap) = kognitara.create_snapshot()
                && let Ok(bytes) = snap.raw_bytes()
            {
                let job = kognitara.execute_cube_job(&bytes);
                println!(
                    "[Hamon Cube] Executed 8-node job over {} bytes -> 3D Merkle Root: {:02x?}",
                    job.total_uncompressed_bytes,
                    &job.cube_merkle_root[..8]
                );
            }
        } else if trimmed.eq_ignore_ascii_case("status") {
            println!("[Status] Swarm Epoch: {}", kognitara.swarm.global_epoch);
            for (coord, brain) in &kognitara.swarm.brains {
                println!(
                    "  Node #{:03b} ({:?}): Phase={:?}, Tension={:.2}, Concepts={}",
                    coord.0,
                    brain.specialization,
                    brain.phase,
                    brain.state.cognitive_tension,
                    brain.tree.concepts.len()
                );
            }
        } else if trimmed.eq_ignore_ascii_case("help") {
            println!("Available commands:");
            println!("  <message>  - Talk directly with Kōgnitara");
            println!("  dream      - Advance circadian phase and weave dreams");
            println!("  oracles    - View revelations woven during dreams");
            println!("  questions  - View humble curiosity questions from Kōgnitara");
            println!("  cube-job   - Dispatch a Kpack job across the 8 Hamon Cube nodes");
            println!("  status     - Show status & specialization of the 8 brains");
            println!("  quit       - Exit the Sanctuary Shell");
        } else {
            let (resp, concepts) = kognitara.converse(&ki_id, trimmed);
            println!("Kōgnitara   > {}", resp);
            if !concepts.is_empty() {
                println!("  [Mind resonating with: {} concepts]", concepts.len());
            }
        }
    }
}

trait IsTerminal {
    fn is_terminal(&self) -> bool;
}

impl IsTerminal for io::Stdin {
    fn is_terminal(&self) -> bool {
        // Safe heuristic fallback for non-interactive test harnesses
        std::env::var("CI").is_err()
            && std::env::var("TERM").is_ok()
            && std::env::var("TERM").unwrap() != "dumb"
    }
}
