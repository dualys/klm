use crossterm::style::Stylize;
use crossterm::terminal::size;
use inquire::Text;
use justify::{InsertAt, justify_paragraph};
use klm::Kognitara;
use klm::family::{FamilyMember, FamilyRole, MemberId};
use std::path::Path;
use std::process::exit;
const DATA: &str = "/mnt/kognitara/data";
fn ok(message: &str) {
    let (cols, _) = size().expect("no terminal");

    // Calcul du padding basé sur les caractères visibles
    let visible_len = message.chars().count() as u16;
    let padding = cols.saturating_sub(visible_len).saturating_sub(15) as usize;

    let star = " * ".green().bold();
    let msg = message.white().bold();
    let cl = "[ ".cyan().bold();
    let ok_str = "success".green().bold();
    let cr = " ]".cyan().bold();
    let pad = " ".repeat(padding);

    println!("{}{}{}{}{}{}", star, msg, pad, cl, ok_str, cr);
}

fn ko(message: &str) {
    let (cols, _) = size().expect("no terminal");
    let visible_len = message.chars().count() as u16;
    let padding = cols.saturating_sub(visible_len).saturating_sub(15) as usize;

    println!(
        "{} {}{}{}{}{}",
        " !".red().bold(),
        message.white().bold(),
        " ".repeat(padding),
        "[ ".white().bold(),
        "failure".red().bold(),
        " ]\n".white().bold()
    );
}

fn main() {
    let (rows, _) = size().expect("terminal");
    // Ajuste ton chemin vers le SSD 8 To si nécessaire
    let path = "/mnt/kognitara/data";

    let mut kognitara = Kognitara::new("Sanctuary of House Ki");
    let ki_id = MemberId::from("ki");

    // LOGIQUE DE DÉMARRAGE BARE-METAL
    if Path::new(path).exists() {
        ok("Lecture et décompression Kpack depuis le volume physique...");
        match std::fs::read_to_string(path) {
            Ok(raw_data) => match serde_json::from_str::<klm::storage::KpackRecipe>(&raw_data) {
                Ok(recipe) => match recipe.decode() {
                    Ok(json_bytes) => match serde_json::from_slice::<Kognitara>(&json_bytes) {
                        Ok(awakened) => {
                            kognitara = awakened;
                            ok("Résurrection réussie ! Intégrité BLAKE3 vérifiée.");
                            ok(&format!(
                                "Sanctuary : {}",
                                kognitara.sanctuary.sanctuary_name
                            ));
                            ok(&format!(
                                "Époque globale : {}",
                                kognitara.swarm.global_epoch
                            ));
                        }
                        Err(e) => {
                            ko(&format!("Échec de reconstruction de la structure : {e}"));
                            exit(1);
                        }
                    },
                    Err(e) => {
                        ko(&format!(
                            "Échec de décompression Kpack (données corrompues) : {e}"
                        ));
                        exit(1);
                    }
                },
                Err(e) => {
                    ko(&format!("Recette Kpack invalide : {e}"));
                    exit(1);
                }
            },
            Err(e) => {
                ko(&format!("Impossible de lire le disque : {e}"));
                exit(1);
            }
        }
    } else {
        // Premier démarrage (Genesis)
        ok("Aucun fichier data trouvé. Initialisation d'une nouvelle matrice cognitive.");

        let family_head = FamilyMember::new("ki", "Ki", FamilyRole::Parent);
        kognitara.sanctuary.register_member(family_head);
        kognitara
            .sanctuary
            .record_shared_anecdote(&ki_id, "First awakening on Amentys OS architecture");

        ok(&format!(
            "Sanctuary initialized: '{}'",
            kognitara.sanctuary.sanctuary_name
        ));
        ok("Registered family member: 'Ki' (Role: Parent, Unconditional Trust: 0.90)");
        ok("Hamon Cube 8-Brain Swarm online (Nodes 000 to 111 with specializations)");
    }

    let commands = "dream, council, unpack, pack, socratic, oracles, questions, jobs, status, quit";
    let x = rows
        .saturating_sub(commands.chars().count() as u16)
        .saturating_div(2);
    println!("{}", " ".repeat(rows as usize));
    println!("{}{commands}", " ".repeat(x as usize));
    println!("{}", " ".repeat(rows as usize));

    loop {
        let input = Text::new("\x1b[1;32mki\x1b[0m>")
            .prompt_skippable()
            .unwrap_or_default()
            .unwrap();
        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        } else if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
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
        } else if trimmed.eq_ignore_ascii_case("council") {
            if let Ok(delib) = kognitara.convene_council("Sanctuary Wisdom & Socratic Path") {
                println!(
                    "[Council of Elders] Verdict: {:?} (Harmony: {:.2})",
                    delib.verdict, delib.consensus_harmony
                );
                println!("  {}", delib.collective_socratic_guidance);
                for c in delib.counsels {
                    println!(
                        "  * {} ({:?}): \"{}\"",
                        c.elder_name, c.archetype, c.socratic_question
                    );
                }
            }
        } else if trimmed.eq_ignore_ascii_case("socratic") {
            let reflections = kognitara.socratic_reflections();
            if reflections.is_empty() {
                println!("No Socratic reflections recorded yet.");
            } else {
                for r in reflections {
                    println!(
                        "  [{}] ({:?}, Humility {:.2}) {}",
                        r.id, r.stage, r.humility_index, r.interior_monologue
                    );
                    println!("       Maieutic Question: {}", r.maieutic_question);
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
        } else if trimmed.eq_ignore_ascii_case("pack") {
            println!("[*] Compression Kpack de l'entité globale...");

            match serde_json::to_vec(&kognitara) {
                Ok(json_bytes) => {
                    let recipe = klm::storage::KpackRecipe::encode(&json_bytes);
                    match serde_json::to_string(&recipe) {
                        Ok(recipe_json) => {
                            if let Err(e) = std::fs::write(path, recipe_json) {
                                println!("[-] Erreur d'écriture disque : {}", e);
                            } else {
                                println!(
                                    "[+] Kōgnitara compressée et sauvegardée ! (Merkle Root BLAKE3: {:02x?})",
                                    &recipe.blake3_hash[..8]
                                );
                            }
                        }
                        Err(e) => println!("[-] Erreur de conversion recette : {}", e),
                    }
                }
                Err(e) => println!("[-] Erreur de sérialisation globale : {}", e),
            }
        } else if trimmed.eq_ignore_ascii_case("unpack") {
            println!("[*] Lecture et décompression Kpack depuis le volume physique...");

            match std::fs::read_to_string(DATA) {
                Ok(raw_data) => {
                    match serde_json::from_str::<klm::storage::KpackRecipe>(&raw_data) {
                        Ok(recipe) => match recipe.decode() {
                            Ok(json_bytes) => {
                                match serde_json::from_slice::<Kognitara>(&json_bytes) {
                                    Ok(awakened) => {
                                        kognitara = awakened;
                                        println!(
                                            "[+] Résurrection réussie ! Intégrité BLAKE3 vérifiée."
                                        );
                                        println!(
                                            "[+] Sanctuary : {}",
                                            kognitara.sanctuary.sanctuary_name
                                        );
                                        println!(
                                            "[+] Époque globale : {}",
                                            kognitara.swarm.global_epoch
                                        );
                                    }
                                    Err(e) => println!(
                                        "[-] Échec de reconstruction de la structure : {}",
                                        e
                                    ),
                                }
                            }
                            Err(e) => println!(
                                "[-] Échec de décompression Kpack (données corrompues) : {}",
                                e
                            ),
                        },
                        Err(e) => {
                            println!("[-] Fichier illisible ou recette Kpack invalide : {}", e)
                        }
                    }
                }
                Err(e) => println!("[-] Impossible de lire le disque : {}", e),
            }
        } else if trimmed.eq_ignore_ascii_case("questions") {
            let questions = kognitara.pending_questions();
            if questions.is_empty() {
                println!("Kōgnitara is currently at peace with her understanding.");
            } else {
                for q in questions {
                    println!(
                        "  [{}] (Urgency {:.2}) {}",
                        q.id, q.urgency, q.question_text
                    );
                }
            }
        } else if trimmed.eq_ignore_ascii_case("jobs") {
            if let Ok(snap) = kognitara.create_snapshot() {
                if let Ok(bytes) = snap.raw_bytes() {
                    let job = kognitara.execute_cube_job(&bytes);
                    println!(
                        "[Hamon Cube] Executed 8-node job over {} bytes -> 3D Merkle Root: {:02x?}",
                        job.total_uncompressed_bytes,
                        &job.cube_merkle_root[..8]
                    );
                }
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
            println!("  <message>    - Talk directly with Kōgnitara");
            println!("      dream    - Advance circadian phase and weave dreams");
            println!("       pack    - Backup knowledge tree on disk");
            println!("     unpack    - Restore disk from backup");
            println!("    council    - Convene the Council of Elders for Socratic deliberation");
            println!("   socratic    - View Kōgnitara's interior Socratic reflections");
            println!("    oracles    - View revelations woven during dreams");
            println!("  questions    - View humble curiosity questions from Kōgnitara");
            println!("   cube-job    - Dispatch a Kpack job across the 8 Hamon Cube nodes");
            println!("     status    - Show status & specialization of the 8 brains");
            println!("       quit    - Exit the Sanctuary Shell");
        } else {
            let (resp, concepts) = kognitara.converse(&ki_id, trimmed);
            println!(
                "\r\n{}",
                justify_paragraph(
                    &resp,
                    &justify::Settings {
                        justify_last_line: true,
                        hyphenate_overflow: true,
                        width: 100,
                        insert_at: InsertAt::Left,
                        wcwidth: true,
                        ignore_spaces: true,
                        newline: "\n",
                        hyphen: "\n",
                        separator: "\n"
                    }
                )
            );
            if !concepts.is_empty() {
                println!("  [Mind resonating with: {} concepts]", concepts.len());
            }
        }
    }
}
