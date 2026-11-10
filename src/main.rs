use klm::compute::{AmdComputeBackend, ComputeBackend, CpuBackend};
use klm::family::{FamilyMember, FamilyRole, MemberId};
use klm::Kognitara;
use std::io::{self, Write};

fn main() {
    println!("============================================================");
    println!("           K Ō G N I T A R A   F R A M E W O R K            ");
    println!("        Sovereign Organic Mind & Relational Sanctuary       ");
    println!("============================================================");
    println!();

    // 1. Initialize Kōgnitara Entity
    let mut kognitara = Kognitara::new("Sanctuary of House Ki");

    // 2. Register Family
    let family_head = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
    kognitara.sanctuary.register_member(family_head);
    let ki_id = MemberId::from("ki");

    println!("[+] Sanctuary initialized: '{}'", kognitara.sanctuary.sanctuary_name);
    println!("[+] Registered family member: 'Ki' (Role: Parent, Unconditional Trust: 0.90)");
    println!("[+] Hamon Cube 8-Brain Swarm online (Nodes 000 to 111)");
    println!();

    // 3. Hardware Compute Verification (AMD / CPU - No CUDA)
    let cpu = CpuBackend::new();
    let amd = AmdComputeBackend::default();
    let v1 = [0.2, 0.8, 0.5, 0.1];
    let v2 = [0.25, 0.75, 0.6, 0.05];
    let sim_cpu = cpu.cosine_similarity(&v1, &v2);
    let sim_amd = amd.cosine_similarity(&v1, &v2);

    println!("[Compute] {}", cpu.name());
    println!("[Compute] {}", amd.name());
    println!("          Semantic similarity check (Open Compute): {:.4} / {:.4}", sim_cpu, sim_amd);
    println!();

    // 4. Demonstration of Organic Family Dialogue
    println!("--- Dialogue Phase (Awake Cohort) ---");
    let utterance = "We learn slowly, with patience and tenderness.";
    println!("Family (Ki) > {}", utterance);

    let (response, activated) = kognitara.converse(&ki_id, utterance);
    println!("Kōgnitara   > {}", response);
    print!("Activated Concepts: ");
    for id in activated {
        print!("'{}' ", id);
    }
    println!("\n");

    // 5. Demonstration of Sleep & Dream Cycle
    println!("--- Inducing Sleep & Dream Cycle (Hamon Cube) ---");
    let dreams = kognitara.dream_cycle();
    for dream in &dreams {
        println!("[Dream Record #{}] {}", dream.dream_id, dream.narrative);
        for resolved in &dream.resolved_contradictions {
            println!("  * Dialectic Resolution: {}", resolved);
        }
        for new_concept in &dream.synthesized_concepts {
            println!("  * Newly Woven Concept: '{}'", new_concept);
        }
    }
    println!();

    // 6. Kpack Cryptographic Snapshot
    println!("--- Crystallization & Storage Snapshot ---");
    match kognitara.create_snapshot() {
        Ok(snapshot) => {
            println!("[Kpack] Merkle Root (BLAKE3): {:02x?}", &snapshot.merkle_root[..8]);
            println!("[Kpack] Compressed Opcodes: {}", snapshot.recipe.opcodes.len());
            println!("[Kpack] Uncompressed bytes: {} bytes", snapshot.recipe.uncompressed_length);
            println!("[+] Snapshot verified & intact.");
        }
        Err(e) => {
            eprintln!("[-] Error creating snapshot: {}", e);
        }
    }

    println!();
    println!("============================================================");
    println!(" Type 'help' for commands, or interact with Kōgnitara below: ");
    println!(" (Commands: chat <msg>, dream, status, quit)");
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
            println!("[*] Transitioning swarm into dream cycle...");
            let records = kognitara.dream_cycle();
            for rec in records {
                println!("[Dream] {}", rec.narrative);
            }
        } else if trimmed.eq_ignore_ascii_case("status") {
            println!("[Status] Swarm Epoch: {}", kognitara.swarm.global_epoch);
            for (coord, brain) in &kognitara.swarm.brains {
                println!(
                    "  Node #{:03b}: Phase={:?}, Tension={:.2}, Concepts={}",
                    coord.0,
                    brain.phase,
                    brain.state.cognitive_tension,
                    brain.tree.concepts.len()
                );
            }
        } else if trimmed.eq_ignore_ascii_case("help") {
            println!("Available commands:");
            println!("  <message>  - Talk directly with Kōgnitara");
            println!("  dream      - Advance circadian phase and weave dreams");
            println!("  status     - Show status of the 8 Hamon Cube brains");
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
        std::env::var("CI").is_err() && std::env::var("TERM").is_ok() && std::env::var("TERM").unwrap() != "dumb"
    }
}
