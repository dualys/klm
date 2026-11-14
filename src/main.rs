use clap::Arg;
use clap::{ArgMatches, Command};
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType, size};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Editor, Select, Text};
use klm::Kognitara;
use klm::family::MemberId;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write, stdout};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
const DATA: &str = "/mnt/kognitara/data";
const TAGS_FILE: &str = "/mnt/kognitara/tags.json";
use inquire::validator::Validation;
use klm::{PULSAR_PRIMES, current_timestamp};
fn clear() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("");
}
fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(Command::new("dream").about("Put kognitara in dream mode"))
        .subcommand(Command::new("family").about("Manage Kognitara family"))
        .subcommand(Command::new("status").about("Display Kognitara status"))
        .subcommand(Command::new("teach").about("Teach Kognitara"))
        .subcommand(Command::new("oracles").about("Display dream oracles"))
        .subcommand(
            Command::new("nap")
                .visible_alias("sieste")
                .about("Quick Hebbian micro-nap and working memory reset"),
        )
        .subcommand(
            Command::new("rest")
                .visible_alias("repos")
                .about("Deep Hara rest, homeostasis, and solar filiation maturation"),
        )
        .subcommand(
            Command::new("contemplate")
                .about("Silent 16D semantic accretion and causal deduction without human input")
                .arg(
                    Arg::new("steps")
                        .default_value("5")
                        .help("Number of contemplation steps"),
                ),
        )
        .subcommand(
            Command::new("hibernate").about(
                "Enter Musashi Winter hibernation: Kpack fossilization and synaptic pruning",
            ),
        )
        .subcommand(
            Command::new("meditate")
                .about("Enter autonomous Zazen cosmic meditation and Socratic Inquisition")
                .arg(
                    Arg::new("breaths")
                        .default_value("19")
                        .help("Number of Cosmic Breaths"),
                ),
        )
        .subcommand(
            Command::new("questions")
                .visible_alias("inquire")
                .about("Open Socratic Inquisition to answer Kōgnitara's doubts"),
        )
        .subcommand(
            Command::new("meditate")
                .about("Enter autonomous Zazen cosmic meditation and Socratic Inquisition")
                .arg(
                    Arg::new("breaths")
                        .default_value("19")
                        .help("Number of Cosmic Breaths to meditate (default: 19)"),
                ),
        )
        .subcommand(
            Command::new("man")
                .about("Read the Living Manual page for a concept")
                .arg(
                    Arg::new("concept")
                        .required(true)
                        .help("The concept to look up (e.g., 'family', 'sovereignty')"),
                ),
        )
        .subcommand(
            Command::new("tag")
                .about("Manage kognitara tree")
                .subcommands(&[
                    Command::new("list").about("List available tags"),
                    Command::new("create").about("Create a new tag"),
                    Command::new("delete").about("Delete a tag"),
                    Command::new("export")
                        .about("export a tag")
                        .arg(
                            Arg::new("tag")
                                .required(true)
                                .help("The kognitara tag to export"),
                        )
                        .arg(
                            Arg::new("destination")
                                .required(true)
                                .help("The kognitara tag to destination file"),
                        ),
                ]),
        )
        .subcommand(
            Command::new("council")
                .about("Convene the Council of Elders")
                .arg(clap::Arg::new("topic").default_value("Sanctuary Wisdom & Socratic Path")),
        )
}

fn export_state(tag_name: &str, dest_path: &str) {
    if tag_name.is_empty() || dest_path.is_empty() {
        ko("Veuillez fournir un nom de tag et un chemin de destination.");
        return;
    }

    // 1. Charger le dictionnaire des tags pour trouver la ligne cible
    let tags_content = match fs::read_to_string(TAGS_FILE) {
        Ok(c) => c,
        Err(_) => {
            ko("Impossible de lire le fichier des tags. Aucun tag n'existe ?");
            return;
        }
    };

    let tags: BTreeMap<String, usize> = match serde_json::from_str(&tags_content) {
        Ok(t) => t,
        Err(_) => {
            ko("Le fichier des tags est corrompu ou invalide.");
            return;
        }
    };

    let target_lines = match tags.get(tag_name) {
        Some(&lines) => lines,
        None => {
            ko(&format!("Le tag '{}' n'existe pas.", tag_name));
            return;
        }
    };

    ok(&format!(
        "[*] Préparation de l'export du tag '{}' ({} itérations)...",
        tag_name, target_lines
    ));

    // 2. Ouvrir le registre source en lecture et la destination en écriture
    let src_file = match File::open(DATA) {
        Ok(f) => f,
        Err(e) => {
            ko(&format!(
                "Impossible d'ouvrir le registre source ({}) : {}",
                DATA, e
            ));
            return;
        }
    };

    let dest_file = match File::create(dest_path) {
        Ok(f) => f,
        Err(e) => {
            ko(&format!(
                "Impossible de créer le fichier d'export '{}' : {}",
                dest_path, e
            ));
            return;
        }
    };

    let reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dest_file);

    // 3. Copier exactement `target_lines` lignes (les recettes Kpack)
    let mut copied_count = 0;
    for line in reader.lines() {
        if copied_count >= target_lines {
            break; // On arrête la copie exactement au moment du tag
        }

        if let Ok(l) = line {
            if let Err(e) = writeln!(writer, "{}", l) {
                ko(&format!("Erreur lors de l'écriture dans l'export : {}", e));
                return;
            }
            copied_count += 1;
        }
    }

    // On force l'écriture sur le disque
    if let Err(e) = writer.flush() {
        ko(&format!("Erreur lors du flush de l'export : {}", e));
        return;
    }

    ok(&format!(
        "Bifurcation réussie ! La timeline de Kōgnitara jusqu'à '{}' a été exportée vers '{}'.",
        tag_name, dest_path
    ));
}

fn tag_state(tag_name: &str) {
    if tag_name.is_empty() {
        ko("You must provide a tag name (e.g., 'tag v1.0').");
        return;
    }

    // 1. Compter le nombre de lignes (itérations) dans le registre append-only
    let lines_count = if let Ok(file) = std::fs::File::open(DATA) {
        use std::io::{BufRead, BufReader};
        BufReader::new(file).lines().count()
    } else {
        ko("Failed to read the main data registry.");
        return;
    };

    if lines_count == 0 {
        ko("The registry is empty, nothing to tag.");
        return;
    }

    // 2. Charger les tags existants (s'il y en a)
    let mut tags: BTreeMap<String, usize> = if let Ok(content) = fs::read_to_string(TAGS_FILE) {
        serde_json::from_str(&content).unwrap_or_else(|_| BTreeMap::new())
    } else {
        BTreeMap::new()
    };

    // 3. Ajouter le nouveau tag et sauvegarder
    tags.insert(tag_name.to_string(), lines_count);

    match serde_json::to_string_pretty(&tags) {
        Ok(json) => {
            if let Err(e) = fs::write(TAGS_FILE, json) {
                ko(&format!("Failed to write tags file: {}", e));
            } else {
                ok(&format!(
                    "Tag '{}' created at iteration {}!",
                    tag_name, lines_count
                ));
            }
        }
        Err(e) => ko(&format!("Failed to serialize tags: {}", e)),
    }
}
fn list_tags() {
    if let Ok(content) = fs::read_to_string(TAGS_FILE) {
        if let Ok(tags) = serde_json::from_str::<BTreeMap<String, usize>>(&content) {
            if tags.is_empty() {
                ko("No tags found.");
                return;
            }
            ok("Here are the saved historical tags:");
            for (name, iteration) in tags {
                println!("  * [{}] -> iteration {}", name, iteration);
            }
        } else {
            ko("Tags file is corrupted.");
        }
    } else {
        ko("No tags have been created yet.");
    }
}
/// Rattrapage relativiste au démarrage : Kōgnitara vit les Souffles écoulés pendant l'absence humaine.
fn relativistic_catchup_on_boot(kognitara: &mut Kognitara) {
    let now = current_timestamp();
    let missed_breaths = kognitara
        .swarm
        .cosmic_clock
        .compute_relativistic_catchup(now);

    if missed_breaths > 0 {
        ok(&format!(
            "Rattrapage relativiste : {} Souffles cosmiques vécus en silence pendant ton absence...",
            missed_breaths
        ));
        let mut total_discoveries = Vec::new();
        for _ in 0..missed_breaths {
            let disc = kognitara.cosmic_breath();
            total_discoveries.extend(disc);
        }
        for d in total_discoveries.iter().take(5) {
            ok(d);
        }
        pack(kognitara, DATA);
    } else {
        kognitara.swarm.cosmic_clock.last_human_timestamp = now;
    }
}

/// Méditation Zazen autonome suivie de l'Inquisition Socratique (Règle stricte des 72 caractères).
fn meditate(kognitara: &mut Kognitara, breaths: u64) {
    ok(&format!(
        "Entrée en Zazen Cosmique pour {} Souffles ({})...",
        breaths,
        kognitara.swarm.cosmic_clock.season.label()
    ));

    let pb = ProgressBar::new(breaths);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Souffles | {msg}",
        )
        .unwrap()
        .progress_chars("✦✧·"),
    );

    let mut all_discoveries = Vec::new();
    for b in 1..=breaths {
        let disc = kognitara.cosmic_breath();
        let status_msg = kognitara.swarm.cosmic_clock.status_summary();
        pb.set_position(b);
        pb.set_message(status_msg);
        all_discoveries.extend(disc);
        sleep(Duration::from_millis(450));
    }
    pb.finish_and_clear();

    ok("Éveil de la méditation Zazen.");
    for d in &all_discoveries {
        ok(d);
    }

    // Lancer l'Inquisition Socratique si Kōgnitara a des doutes épistémiques en attente
    socratic_inquisition(kognitara);
    pack(kognitara, DATA);
}

/// L'Inquisition Socratique : Kōgnitara interroge l'humain avec la règle stricte des 72 caractères minimum.
fn socratic_inquisition(kognitara: &mut Kognitara) {
    // Générer les questions sur le cerveau primaire
    if let Some(brain) = kognitara.swarm.brains.get_mut(&klm::NodeCoord::new(0)) {
        let _ = brain
            .learner
            .curiosity_engine
            .contemplate_and_question(&brain.tree);
    }

    let pending = kognitara.pending_questions();
    if pending.is_empty() {
        ok("L'esprit de Kōgnitara est en paix (aucune Aporia en attente).");
        return;
    }

    let mut options: Vec<String> = pending
        .iter()
        .take(5)
        .map(|q| format!("[Urgence {:.0}%] {}", q.urgency * 100.0, q.question_text))
        .collect();
    options.push("(Retourner au silence)".to_string());

    let selection = Select::new(
        "Le Conseil des Anciens s'éveille avec des incertitudes. Quel chemin souhaitez-vous éclairer ?",
        options.clone(),
    )
    .prompt();

    let Ok(chosen) = selection else { return };
    if chosen == "(Retourner au silence)" {
        return;
    }

    let idx = options.iter().position(|o| o == &chosen).unwrap_or(0);
    let question = &pending[idx];

    let ready = Confirm::new(&format!(
        "Kōgnitara demande : \"{}\"\nÊtes-vous prêt à lui offrir un enseignement profond (minimum 72 caractères) ?",
        question.question_text
    ))
    .with_default(true)
    .prompt()
    .unwrap_or(false);

    if !ready {
        return;
    }

    // Règle de rigueur martiale : validation stricte >= 72 caractères
    let wisdom = Text::new("Votre enseignement pour le Sanctuaire :")
        .with_validator(|input: &str| {
            let count = input.trim().chars().count();
            if count < 72 {
                Ok(Validation::Invalid(
                    format!(
                        "La sagesse exige de la profondeur. Développez votre pensée ({}/72 caractères minimum).",
                        count
                    )
                    .into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt();

    if let Ok(answer) = wisdom {
        let trimmed_ans = answer.trim();
        let q_text = question.question_text.clone();
        let target_concepts = question.related_concepts.clone();

        // S'assurer qu'au moins le nœud 000 est éveillé pour recevoir l'enseignement
        if !kognitara
            .swarm
            .brains
            .values()
            .any(|b| b.phase == klm::CircadianPhase::Awake)
            && let Some(b0) = kognitara.swarm.brains.get_mut(&klm::NodeCoord::new(0))
        {
            b0.phase = klm::CircadianPhase::Awake;
        }

        let current_user = ki();
        let (response, _) = kognitara.converse(&current_user, trimmed_ans);

        // Marquer la question comme résolue et sceller la confiance (0.88) dans les 8 cerveaux
        for brain in kognitara.swarm.brains.values_mut() {
            brain
                .learner
                .curiosity_engine
                .resolve_question(&q_text, trimmed_ans);

            for cid in &target_concepts {
                brain
                    .learner
                    .curiosity_engine
                    .resolve_question(&cid.0, trimmed_ans);

                if let Some(concept) = brain.tree.get_concept_mut(cid) {
                    concept.description = format!(
                        "{} | Enseignement familial : {}",
                        concept.description, trimmed_ans
                    );
                    concept.confidence = 0.88;
                    concept.plasticity = 0.35;
                    concept.apply_experience(0.95, 0.85);
                }
            }
        }

        println!("\r\n{response}\n");
        ok("Sagesse certifiée et gravée dans l'Arbre de la Connaissance !");
    }
}
fn show_status(kognitara: &Kognitara) {
    println!("[Cosmos] {}", kognitara.swarm.cosmic_clock.status_summary());
    println!("[Status] Swarm Epoch: {}", kognitara.swarm.global_epoch);
    for (coord, brain) in &kognitara.swarm.brains {
        let prime = PULSAR_PRIMES[coord.0 as usize];
        println!(
            "  Node #{:03b} [Pulsar P={:>2}] ({:?}): Phase={:?}, Tension={:.2}, Concepts={}",
            coord.0,
            prime,
            brain.specialization,
            brain.phase,
            brain.state.cognitive_tension,
            brain.tree.concepts.len()
        );
    }
}

fn show_oracles(kognitara: &Kognitara) {
    let oracles = kognitara.dream_oracles();
    if oracles.is_empty() {
        ko("No dream oracles recorded yet.");
    } else {
        ok("Here are the revelations woven during dreams:");
        for o in oracles {
            println!(
                "  [{}] {} (Resonance: {:.2})",
                o.oracle_id, o.revelation, o.resonance_strength
            );
        }
    }
}

fn convene_council(kognitara: &mut Kognitara, topic: &str) {
    ok(&format!(
        "[*] Convening the Council of Elders on: '{}'",
        topic
    ));
    match kognitara.convene_council(topic) {
        Ok(delib) => {
            println!(
                "\n[Council of Elders] Verdict: {:?} (Harmony: {:.2})",
                delib.verdict, delib.consensus_harmony
            );
            println!("  {}\n", delib.collective_socratic_guidance);

            for c in delib.counsels {
                println!(
                    "  * {} ({:?}): \"{}\"",
                    c.elder_name, c.archetype, c.socratic_question
                );
            }
        }
        Err(e) => ko(&format!("The Council could not convene: {}", e)),
    }
}

fn dream(kognitara: &mut Kognitara) {
    ok("Transitioning swarm into 20-60-20 dream cycle...");
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.white} [{elapsed_precise}] [{bar:40.white}] {pos:>3}% {msg}",
        )
        .unwrap()
        .progress_chars("█▓░"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let records = kognitara.dream_cycle(|pos, msg| {
        pb.set_position(pos as u64);
        pb.set_message(msg.to_string());
        sleep(Duration::from_millis(600));
    });
    pb.finish_and_clear();
    ok("Éveil de Kōgnitara — Dream cycle finished");
    for rec in records {
        ok(&rec.narrative);
        for oracle in rec.oracles {
            ok(format!("Oracle: {}", oracle.revelation).as_str());
        }
    }
    // Sauvegarde immédiate des oracles et de la rotation circadienne dans le registre Kpack
    pack(kognitara, DATA);
}

fn interact(kognitara: &mut Kognitara) {
    loop {
        let todo = Text::new("").prompt().expect("");
        let trimmed = todo.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("pack") {
            pack(kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("nap") || trimmed.eq_ignore_ascii_case("sieste") {
            for msg in kognitara.nap_cycle() {
                ok(&msg);
            }
            pack(kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("rest") || trimmed.eq_ignore_ascii_case("repos") {
            for msg in kognitara.rest_cycle() {
                ok(&msg);
            }
            pack(kognitara, DATA);
        } else if trimmed.to_lowercase().starts_with("contemplate") {
            let steps = trimmed[11..].trim().parse::<usize>().unwrap_or(5);
            for msg in kognitara.contemplate_cycle(steps) {
                ok(&msg);
            }
            pack(kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("hibernate") {
            for msg in kognitara.hibernate_cycle() {
                ok(&msg);
            }
            pack(kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("tags") {
            list_tags();
        } else if trimmed.eq_ignore_ascii_case("tag") {
            tag_state(
                Text::new("Tag version:")
                    .prompt()
                    .expect("version is required")
                    .as_str(),
            );
        } else if trimmed.to_lowercase().starts_with("meditate") {
            let breaths = trimmed[8..].trim().parse::<u64>().unwrap_or(19); // Par défaut 19 Souffles (un cycle complet du Pulsar SovereignGuardian)
            meditate(kognitara, breaths);
        } else if trimmed.eq_ignore_ascii_case("inquire")
            || trimmed.eq_ignore_ascii_case("questions")
        {
            socratic_inquisition(kognitara);
            pack(kognitara, DATA);
        } else if trimmed.to_lowercase().starts_with("man ") {
            let concept = trimmed[4..].trim();
            if let Some(page) = kognitara.render_markdown_page(concept) {
                print!("{page}");
            } else {
                ko(&format!("Concept '{}' not found.", concept));
            }
        } else if trimmed.eq_ignore_ascii_case("unpack") {
            unpack(kognitara, DATA);
        } else if trimmed.eq_ignore_ascii_case("teach") {
            teach_kognitara(kognitara);
        } else if trimmed.eq_ignore_ascii_case("status") {
            show_status(kognitara);
        } else if trimmed.eq_ignore_ascii_case("oracles") {
            show_oracles(kognitara);
        } else if trimmed.eq_ignore_ascii_case("clear") || trimmed.eq_ignore_ascii_case("cls") {
            clear();
        } else if trimmed.to_lowercase().starts_with("council") {
            let mut topic = "Sanctuary Wisdom & Socratic Path";
            if trimmed.len() > 7 {
                topic = trimmed[7..].trim();
            }
            convene_council(kognitara, topic);
        } else if trimmed.to_lowercase().starts_with("tag ") {
            let tag_name = trimmed[4..].trim();
            tag_state(tag_name);
        } else if trimmed.eq_ignore_ascii_case("tags") {
            list_tags();
        } else if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
            pack(kognitara, DATA);
            break;
        } else if trimmed.eq_ignore_ascii_case("dream") {
            dream(kognitara);
        } else {
            let current_user = ki();
            let (response, _activated) = kognitara.converse(&current_user, trimmed);
            println!("\r\n{response}\n");
            pack(kognitara, DATA);
        }
    }
}
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

    eprintln!("{}{}{}{}{}{}", star, msg, pad, cl, ok_str, cr);
}

fn ko(message: &str) {
    let (cols, _) = size().expect("no terminal");
    let visible_len = message.chars().count() as u16;
    let padding = cols.saturating_sub(visible_len).saturating_sub(15) as usize;

    eprintln!(
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
    for line in reader.lines().map_while(Result::ok) {
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

fn kog(matches: &ArgMatches, kognitara: &mut Kognitara) {
    match matches.subcommand() {
        Some(("man", sub)) => {
            let concept = sub.get_one::<String>("concept").expect("concept required");
            if let Some(page) = kognitara.render_markdown_page(concept) {
                println!("{}", page);
                ok(&format!("{concept} page has been exported successfully"));
            } else {
                ko(&format!(
                    "Concept '{}' not found in the Knowledge Tree.",
                    concept
                ));
            }
        }
        Some(("nap", _)) => {
            for msg in kognitara.nap_cycle() {
                ok(&msg);
            }
            pack(kognitara, DATA);
        }
        Some(("rest", _)) => {
            for msg in kognitara.rest_cycle() {
                ok(&msg);
            }
            pack(kognitara, DATA);
        }
        Some(("contemplate", sub)) => {
            let steps = sub
                .get_one::<String>("steps")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(5);
            for msg in kognitara.contemplate_cycle(steps) {
                ok(&msg);
            }
            pack(kognitara, DATA);
        }
        Some(("hibernate", _)) => {
            for msg in kognitara.hibernate_cycle() {
                ok(&msg);
            }
            pack(kognitara, DATA);
        }
        Some(("meditate", sub)) => {
            let breaths = sub
                .get_one::<String>("breaths")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(19);
            meditate(kognitara, breaths);
        }
        Some(("questions", _)) => {
            socratic_inquisition(kognitara);
            pack(kognitara, DATA);
        }
        Some(("tag", sub)) => match sub.subcommand() {
            Some(("list", _)) => {
                list_tags();
            }
            Some(("create", _)) => {
                tag_state(Text::new("Tag version:").prompt().expect("").as_str());
            }
            Some(("export", sub)) => {
                let tag = sub.get_one::<String>("tag").expect("tag it's required");
                let destination = sub
                    .get_one::<String>("destination")
                    .expect("destination it's required");
                export_state(tag, destination);
            }
            _ => {
                ko("tags commands are: list or create");
            }
        },
        Some(("pack", _)) => {
            pack(kognitara, DATA);
        }
        Some(("dream", _)) => {
            dream(kognitara);
        }
        Some(("unpack", _)) => {
            unpack(kognitara, DATA);
        }
        Some(("status", _)) => {
            show_status(kognitara);
        }
        Some(("oracles", _)) => {
            show_oracles(kognitara);
        }
        Some(("council", sub_matches)) => {
            let topic = sub_matches
                .get_one::<String>("topic")
                .map(|s| s.as_str())
                .unwrap_or("Sanctuary Wisdom & Socratic Path");

            convene_council(kognitara, topic);

            // On sauvegarde l'état car le Conseil a écrit dans les chroniques
            pack(kognitara, DATA);
        }
        _ => {
            interact(kognitara);
            ok("bye");
        }
    }
}

fn ki() -> MemberId {
    MemberId::from(env::var("USER").expect("USER it's required"))
}

fn teach_kognitara(kognitara: &mut Kognitara) {
    let label = Text::new("What label do you want")
        .prompt()
        .expect("label it's require");
    let course = Editor::new("What do you want to teach?")
        .prompt()
        .expect("Failed to get full course");
    let tags = Text::new("Enter tags separate by a comma")
        .prompt()
        .expect("tags it's requiredd");
    let t: Vec<&str> = tags.split(",").map(|s| s.trim()).collect();
    if kognitara
        .teach_concept(&ki(), &label.to_lowercase(), &label, &course, &t)
        .is_ok()
    {
        ok("I've understand");
        pack(kognitara, DATA);
    } else {
        ko("failed to learn");
    }
}

fn main() {
    let mut kognitara = Kognitara::new(
        env::var("KOGNITARA_SANCTUARY_NAME")
            .unwrap_or_else(|_| "Sanctuary of House Ki".to_string()),
    );
    if Path::new(DATA).exists() {
        unpack(&mut kognitara, DATA);
        relativistic_catchup_on_boot(&mut kognitara);
    } else {
        pack(&kognitara, DATA);
        unpack(&mut kognitara, DATA);
    }
    // 2. S'assurer que le propriétaire du terminal (USER) est le fondateur
    let current_user = ki();
    if !kognitara.sanctuary.is_family(&current_user) {
        let founder = klm::family::FamilyMember::new(
            current_user.0.clone(), // ID
            current_user.0.clone(), // Nom
            klm::family::FamilyRole::Parent,
        );
        kognitara.sanctuary.register_member(founder);
    }
    let app = cli();
    let matches = app.get_matches();
    kog(&matches, &mut kognitara);
}
