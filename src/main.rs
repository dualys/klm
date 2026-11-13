use crossterm::style::Stylize;
use crossterm::terminal::size;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use justify::{InsertAt, justify_paragraph};
use klm::Kognitara;
use klm::family::MemberId;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
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

fn pack(kognitara: &Kognitara, path: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;

    // Récupération de la taille actuelle pour le monitoring
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    ok(format!(
        "Compression Kpack et sauvegarde append-only (Taille du registre : {} octets)...",
        file_size
    )
    .as_str());

    match serde_json::to_vec(kognitara) {
        Ok(json_bytes) => {
            let recipe = klm::storage::KpackRecipe::encode(&json_bytes);
            match serde_json::to_string(&recipe) {
                Ok(recipe_json) => {
                    // Ouverture en mode "append" (ajout à la fin).
                    // Le fichier grandira indéfiniment sur le SSD.
                    match OpenOptions::new().create(true).append(true).open(path) {
                        Ok(mut file) => {
                            if let Err(e) = writeln!(file, "{}", recipe_json) {
                                ko(&format!("Erreur d'écriture disque : {}", e));
                            } else {
                                ok(&format!(
                                    "Kōgnitara ajoutée au registre ! (Merkle Root BLAKE3: {:02x?})",
                                    &recipe.blake3_hash[..8]
                                ));
                            }
                        }
                        Err(e) => ko(&format!("Impossible d'ouvrir le volume en append : {}", e)),
                    }
                }
                Err(e) => ko(&format!("Erreur de conversion recette : {}", e)),
            }
        }
        Err(e) => ko(&format!("Erreur de sérialisation globale : {}", e)),
    }
}

fn unpack(kognitara: &mut Kognitara, path: &str) -> bool {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    ok("Restauration depuis le registre append-only...");

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            ko(&format!("Impossible de lire le registre : {}", e));
            return false;
        }
    };

    let reader = BufReader::new(file);
    let mut last_valid_recipe: Option<klm::storage::KpackRecipe> = None;
    let mut lines_count = 0;

    // Balayage du registre complet : on ne conserve en RAM que la dernière ligne valide
    for line in reader.lines().flatten() {
        if let Ok(recipe) = serde_json::from_str::<klm::storage::KpackRecipe>(&line) {
            last_valid_recipe = Some(recipe);
            lines_count += 1;
        }
    }

    if let Some(recipe) = last_valid_recipe {
        match recipe.decode() {
            Ok(json_bytes) => match serde_json::from_slice::<Kognitara>(&json_bytes) {
                Ok(awakened) => {
                    *kognitara = awakened;
                    ok(&format!(
                        "Résurrection réussie depuis l'itération {} du registre !",
                        lines_count
                    ));
                    ok(&format!(
                        "Sanctuary : {}",
                        kognitara.sanctuary.sanctuary_name
                    ));
                    ok(&format!(
                        "Époque globale : {}",
                        kognitara.swarm.global_epoch
                    ));
                    true
                }
                Err(e) => {
                    ko(&format!("Échec de reconstruction de la structure : {}", e));
                    false
                }
            },
            Err(e) => {
                ko(&format!(
                    "Échec de décompression Kpack (données corrompues) : {}",
                    e
                ));
                false
            }
        }
    } else {
        ko("Aucune recette Kpack valide trouvée dans le registre.");
        false
    }
}
fn main() {
    let (rows, _) = size().expect("terminal");
    // Ajuste ton chemin vers le SSD 8 To si nécessaire

    let mut kognitara = Kognitara::new("Sanctuary of House Ki");
    let ki_id = MemberId::from("ki");
    unpack(&mut kognitara, DATA);
    let commands = "dream, council, unpack, pack, socratic, oracles, questions, jobs, status, quit";
    let x = rows
        .saturating_sub(commands.chars().count() as u16)
        .saturating_div(2);
    println!("{}", " ".repeat(rows as usize));
    println!("{}{commands}", " ".repeat(x as usize));
    println!("{}", " ".repeat(rows as usize));

    loop {
        let input = Text::new("").prompt_skippable();
        let trimmed = input.unwrap_or_default().unwrap_or_default();
        let tr = trimmed.trim();
        if tr.trim().is_empty() {
            ok("Bye");
            exit(0);
        } else if tr.eq_ignore_ascii_case("quit") || tr.eq_ignore_ascii_case("exit") {
            pack(&kognitara, DATA);
            ok("Farewell in the Sanctuary. Take care.");
            break;
        } else if tr.eq_ignore_ascii_case("dream") {
            ok("Transitioning swarm into 20-60-20 dream cycle...");

            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.white} {msg} [{bar:40.white}] {pos}%")
                    .unwrap()
                    .progress_chars("██ "),
            );

            // On lance le VRAI cycle de rêve, en lui passant ce qu'il doit faire à chaque étape
            let records = kognitara.dream_cycle(|pos, msg| {
                pb.set_message(msg.to_string());
                pb.set_position(pos as u64);
                sleep(Duration::from_millis(800));
            });

            pb.finish_with_message("Éveil de Kōgnitara.");

            // Affichage des oracles une fois le rêve terminé
            for rec in records {
                println!("[Dream] {}", rec.narrative);
                for oracle in rec.oracles {
                    ok(format!("\tOracle: {}", oracle.revelation).as_str());
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
                ko("No dream oracles recorded yet.");
            } else {
                for o in oracles {
                    println!(
                        "  [{}] {} (Resonance: {:.2})",
                        o.oracle_id, o.revelation, o.resonance_strength
                    );
                }
            }
        } else if trimmed.eq_ignore_ascii_case("pack") {
            pack(&kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("unpack") {
            unpack(&mut kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("questions") {
            let questions = kognitara.pending_questions();
            if questions.is_empty() {
                ok("Kōgnitara is currently at peace with her understanding.");
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
            let (resp, concepts) = kognitara.converse(&ki_id, trimmed.as_str());
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
