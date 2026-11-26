use clap::Arg;
use clap::{ArgMatches, Command};
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType, size};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Editor, Select, Text};
use klm::family::MemberId;
use klm::{EvidenceRelation, Kognitara};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write, stdout};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
mod research;
const DATA: &str = "/mnt/kognitara/data";
const RESEARCH: &str = "/mnt/kognitara/research";
const TAGS_FILE: &str = "/mnt/kognitara/tags.json";
use inquire::validator::Validation;
use klm::{PULSAR_PRIMES, current_timestamp};

fn clear() {
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("");
}

fn limit_message(msg: &str) -> String {
    match msg.char_indices().nth(150) {
        Some((idx, _)) => format!("{}...", &msg[..idx]),
        None => String::from(msg),
    }
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
            Command::new("research-cycle").about("Run one background Wikipedia research cycle"),
        )
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
        ko("Please provide a tag name and a destination path.");
        return;
    }

    // 1. Load the tags dictionary to find the target line
    let tags_content = match fs::read_to_string(TAGS_FILE) {
        Ok(c) => c,
        Err(_) => {
            ko("Cannot read the tags file. No tags exist?");
            return;
        }
    };

    let tags: BTreeMap<String, usize> = match serde_json::from_str(&tags_content) {
        Ok(t) => t,
        Err(_) => {
            ko("The tags file is corrupted or invalid.");
            return;
        }
    };

    let target_lines = match tags.get(tag_name) {
        Some(&lines) => lines,
        None => {
            ko(&format!("The tag '{}' does not exist.", tag_name));
            return;
        }
    };

    ok(&format!(
        "[*] Preparing to export tag '{}' ({} iterations)...",
        tag_name, target_lines
    ));

    // 2. Open the source registry for reading and the destination for writing
    let src_file = match File::open(DATA) {
        Ok(f) => f,
        Err(e) => {
            ko(&format!(
                "Cannot open the source registry ({}) : {}",
                DATA, e
            ));
            return;
        }
    };

    let dest_file = match File::create(dest_path) {
        Ok(f) => f,
        Err(e) => {
            ko(&format!(
                "Cannot create the export file '{}' : {}",
                dest_path, e
            ));
            return;
        }
    };

    let reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dest_file);

    // 3. Copy exactly target_lines lines (the Kpack recipes)
    let mut copied_count = 0;
    for line in reader.lines() {
        if copied_count >= target_lines {
            break; // Stop the copy exactly at the tag
        }

        if let Ok(l) = line {
            if let Err(e) = writeln!(writer, "{}", l) {
                ko(&format!("Error writing to the export: {}", e));
                return;
            }
            copied_count += 1;
        }
    }

    // Force flush to disk
    if let Err(e) = writer.flush() {
        ko(&format!("Error flushing the export: {}", e));
        return;
    }

    ok(&format!(
        "Bifurcation successful! Kōgnitara's timeline up to '{}' has been exported to '{}'.",
        tag_name, dest_path
    ));
}

fn tag_state(tag_name: &str) {
    if tag_name.is_empty() {
        ko("You must provide a tag name (e.g., 'tag v1.0').");
        return;
    }

    // 1. Count the number of lines (iterations) in the append-only registry
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

    // 2. Load existing tags (if any)
    let mut tags: BTreeMap<String, usize> = if let Ok(content) = fs::read_to_string(TAGS_FILE) {
        serde_json::from_str(&content).unwrap_or_else(|_| BTreeMap::new())
    } else {
        BTreeMap::new()
    };

    // 3. Add the new tag and save
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

/// Relativistic catch-up on boot: Kōgnitara lives the Breaths elapsed during human absence.
fn relativistic_catchup_on_boot(kognitara: &mut Kognitara) {
    let now = current_timestamp();
    let missed_breaths = kognitara
        .swarm
        .cosmic_clock
        .compute_relativistic_catchup(now);

    if missed_breaths > 0 {
        ok(&format!(
            "Relativistic catch-up: {} Cosmic Breaths lived in silence during your absence...",
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

/// Autonomous Zazen Meditation followed by Socratic Inquisition (Strict 72-character rule).
fn meditate(kognitara: &mut Kognitara, breaths: u64) {
    ok(&format!(
        "Entering Cosmic Zazen for {} Breaths ({})...",
        breaths,
        kognitara.swarm.cosmic_clock.season.label()
    ));

    let pb = ProgressBar::new(breaths);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Breaths | {msg}",
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

    ok("Awakening from Zazen meditation.");
    for d in &all_discoveries {
        ok(d);
    }

    // Launch Socratic Inquisition if Kōgnitara has pending epistemic doubts
    socratic_inquisition(kognitara);
    pack(kognitara, DATA);
}

/// Socratic Inquisition: Kōgnitara questions the human with the strict 72-character minimum rule.
fn socratic_inquisition(kognitara: &mut Kognitara) {
    // Generate questions on the primary brain
    if let Some(brain) = kognitara.swarm.brains.get_mut(&klm::NodeCoord::new(0)) {
        let _ = brain
            .learner
            .curiosity_engine
            .contemplate_and_question(&brain.tree);
    }

    let pending = kognitara.pending_questions();
    if pending.is_empty() {
        ok("Kōgnitara's mind is at peace (no pending Aporia).");
        return;
    }

    let mut options: Vec<String> = pending
        .iter()
        .take(5)
        .map(|q| format!("[Urgency {:.0}%] {}", q.urgency * 100.0, q.question_text))
        .collect();
    options.push("(Return to silence)".to_string());

    let selection = Select::new(
        "The Council of Elders awakens with uncertainties. Which path do you wish to illuminate?",
        options.clone(),
    )
    .prompt();

    let Ok(chosen) = selection else { return };
    if chosen == "(Return to silence)" {
        return;
    }

    let idx = options.iter().position(|o| o == &chosen).unwrap_or(0);
    let question = &pending[idx];

    let ready = Confirm::new(&format!(
        "Kōgnitara asks: \"{}\"\nAre you ready to offer her a profound teaching (minimum 72 characters)?",
        question.question_text
    ))
    .with_default(true)
    .prompt()
    .unwrap_or(false);

    if !ready {
        return;
    }

    // Martial rigor rule: strict validation >= 72 characters
    let wisdom = Text::new("Your teaching for the Sanctuary:")
        .with_validator(|input: &str| {
            let count = input.trim().chars().count();
            if count < 72 {
                Ok(Validation::Invalid(
                    format!(
                        "Wisdom requires depth. Expand your thought ({}/72 characters minimum).",
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

        // Ensure at least node 000 is awake to receive the teaching
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

        // Mark the question as resolved and seal confidence (0.88) in all 8 brains
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
                    concept.description =
                        format!("{} | Family teaching: {}", concept.description, trimmed_ans);
                    concept.confidence = 0.88;
                    concept.plasticity = 0.35;
                    concept.apply_experience(0.95, 0.85);
                }
            }
        }

        println!("\r\n{response}\n");
        ok("Wisdom certified and engraved in the Tree of Knowledge!");
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
    ok("Kōgnitara awakens — Dream cycle finished");
    for rec in records {
        ok(&rec.narrative);
        for oracle in rec.oracles {
            ok(format!("Oracle: {}", oracle.revelation).as_str());
        }
    }
    // Immediate backup of oracles and circadian rotation in the Kpack registry
    pack(kognitara, DATA);
}

fn hypothesis_command(kognitara: &mut Kognitara, input: &str) -> Option<Result<bool, String>> {
    let (command, args) = input.split_once(' ').unwrap_or((input, ""));
    match command {
        "hypotheses" => {
            if !args.is_empty() {
                return Some(Err("Usage: hypotheses".into()));
            }
            if kognitara.hypotheses.hypotheses.is_empty() {
                println!("No hypotheses yet.");
            }
            for hypothesis in &kognitara.hypotheses.hypotheses {
                println!(
                    "#{} [unverified, revision {}] {}",
                    hypothesis.id,
                    hypothesis.revision(),
                    hypothesis.statement
                );
                for (revision, correction) in hypothesis.corrections.iter().enumerate() {
                    println!(
                        "  revision {revision}: {} | correction: {}",
                        correction.previous_statement, correction.reason
                    );
                }
                for evidence in &hypothesis.evidence {
                    println!(
                        "  revision {}: {:?} | {} | {} | timestamp {}",
                        evidence.revision,
                        evidence.relation,
                        evidence.source,
                        evidence.note,
                        evidence.observed_at
                    );
                }
            }
            Some(Ok(false))
        }
        "hypothesis" => Some(
            kognitara
                .hypotheses
                .add(args, Vec::new())
                .map(|id| {
                    println!("Hypothesis #{id} recorded as unverified.");
                    true
                })
                .map_err(|error| error.to_string()),
        ),
        "evidence" => Some((|| {
            let parts: Vec<&str> = args.splitn(4, '|').map(str::trim).collect();
            if parts.len() != 4 {
                return Err(
                    "Usage: evidence <id> | reports/supports/contradicts | source | note".into(),
                );
            }
            let id = parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid hypothesis ID.")?;
            let relation = match parts[1] {
                "reports" => EvidenceRelation::Reports,
                "supports" => EvidenceRelation::Supports,
                "contradicts" => EvidenceRelation::Contradicts,
                _ => return Err("Relation must be reports, supports, or contradicts.".into()),
            };
            kognitara
                .hypotheses
                .get_mut(id)
                .map_err(|e| e.to_string())?
                .add_evidence(relation, parts[2], parts[3], current_timestamp())
                .map_err(|e| e.to_string())?;
            println!("Evidence recorded; the hypothesis remains unverified.");
            Ok(true)
        })()),
        "correct-hypothesis" => Some((|| {
            let parts: Vec<&str> = args.splitn(3, '|').map(str::trim).collect();
            if parts.len() != 3 {
                return Err("Usage: correct-hypothesis <id> | corrected statement | reason".into());
            }
            let id = parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid hypothesis ID.")?;
            kognitara
                .hypotheses
                .get_mut(id)
                .map_err(|e| e.to_string())?
                .correct(parts[1], parts[2])
                .map_err(|e| e.to_string())?;
            println!("Correction recorded; the new statement remains unverified.");
            Ok(true)
        })()),
        _ => None,
    }
}

fn interact(kognitara: &mut Kognitara) {
    loop {
        let todo = Text::new("").prompt().expect("");
        let trimmed = todo.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(result) = hypothesis_command(kognitara, trimmed) {
            match result {
                Ok(true) => pack(kognitara, DATA),
                Ok(false) => {}
                Err(error) => ko(&error),
            }
        } else if trimmed.eq_ignore_ascii_case("pack") {
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
            let breaths = trimmed[8..].trim().parse::<u64>().unwrap_or(19); // Defaults to 19 Breaths (one full cycle of the SovereignGuardian Pulsar)
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
    let msg = limit_message(message);
    let padding = cols
        .saturating_sub(msg.chars().count() as u16)
        .saturating_sub(15) as usize;
    let star = " * ".green().bold();
    let msg = msg.white().bold();
    let cl = "[ ".cyan().bold();
    let ok_str = "success".green().bold();
    let cr = " ]".cyan().bold();
    let pad = " ".repeat(padding);

    eprintln!("{}{}{}{}{}{}", star, msg, pad, cl, ok_str, cr);
}

fn ko(message: &str) {
    let (cols, _) = size().expect("no terminal");
    let msg = limit_message(message);
    let padding = cols
        .saturating_sub(msg.chars().count() as u16)
        .saturating_sub(15) as usize;

    eprintln!(
        "{} {}{}{}{}{}",
        " !".red().bold(),
        msg.white().bold(),
        " ".repeat(padding),
        "[ ".white().bold(),
        "failure".red().bold(),
        " ]\n".white().bold()
    );
}

fn pack(kognitara: &Kognitara, path: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;

    // Retrieve current size for monitoring
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    ok(format!(
        "Kpack compression and append-only backup (Registry size: {} bytes)...",
        file_size
    )
    .as_str());

    match serde_json::to_vec(kognitara) {
        Ok(json_bytes) => {
            let recipe = klm::storage::KpackRecipe::encode(&json_bytes);
            match serde_json::to_string(&recipe) {
                Ok(recipe_json) => {
                    // Open in "append" mode (add to end).
                    // The file will grow indefinitely on the SSD.
                    match OpenOptions::new().create(true).append(true).open(path) {
                        Ok(mut file) => {
                            if let Err(e) = writeln!(file, "{}", recipe_json) {
                                ko(&format!("Disk write error: {}", e));
                            } else {
                                ok(&format!(
                                    "Kōgnitara added to the registry! (Merkle Root BLAKE3: {:02x?})",
                                    &recipe.blake3_hash[..8]
                                ));
                            }
                        }
                        Err(e) => ko(&format!("Cannot open volume in append mode: {}", e)),
                    }
                }
                Err(e) => ko(&format!("Recipe conversion error: {}", e)),
            }
        }
        Err(e) => ko(&format!("Global serialization error: {}", e)),
    }
}

fn persist_import(kognitara: &Kognitara, path: &Path) -> Result<(), String> {
    let json =
        serde_json::to_vec(kognitara).map_err(|e| format!("Cannot serialize memory: {e}"))?;
    let recipe = klm::KpackRecipe::encode(&json);
    let mut record =
        serde_json::to_vec(&recipe).map_err(|e| format!("Cannot serialize snapshot: {e}"))?;
    record.push(b'\n');
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| format!("Cannot open registry: {e}"))?;
    file.write_all(&record)
        .map_err(|e| format!("Cannot write registry: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("Cannot sync registry: {e}"))?;
    Ok(())
}

fn unpack(kognitara: &mut Kognitara, path: &str) -> bool {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    ok("Restoring from the append-only registry...");

    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            ko(&format!("Cannot read the registry: {}", e));
            return false;
        }
    };

    let reader = BufReader::new(file);
    let mut last_valid_recipe: Option<klm::storage::KpackRecipe> = None;
    let mut lines_count = 0;

    // Scan the entire registry: keep only the last valid line in RAM
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
                        "Successful resurrection from iteration {} of the registry!",
                        lines_count
                    ));
                    ok(&format!(
                        "Sanctuary : {}",
                        kognitara.sanctuary.sanctuary_name
                    ));
                    ok(&format!("Global epoch: {}", kognitara.swarm.global_epoch));
                    true
                }
                Err(e) => {
                    ko(&format!("Failed to reconstruct the structure: {}", e));
                    false
                }
            },
            Err(e) => {
                ko(&format!(
                    "Kpack decompression failed (corrupted data): {}",
                    e
                ));
                false
            }
        }
    } else {
        ko("No valid Kpack recipe found in the registry.");
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

            // Saving state because the Council wrote in the chronicles
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
    let matches = cli().get_matches();
    if matches.subcommand_name() == Some("research-cycle") {
        if let Err(error) = research::cycle(Path::new(DATA), Path::new(RESEARCH)) {
            eprintln!("Research failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    let lock = match fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open("/mnt/kognitara/registry.lock")
    {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Cannot open memory lock: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = lock.try_lock() {
        eprintln!("KLM memory is already in use: {error}");
        std::process::exit(1);
    }
    let mut kognitara = Kognitara::new(
        env::var("KOGNITARA_SANCTUARY_NAME")
            .unwrap_or_else(|_| "Sanctuary of House Ki".to_string()),
    );
    if Path::new(DATA).exists() {
        if !unpack(&mut kognitara, DATA) {
            eprintln!("Cannot open KLM memory; refusing to import research.");
            std::process::exit(1);
        }
    } else {
        pack(&kognitara, DATA);
        if !unpack(&mut kognitara, DATA) {
            eprintln!("Cannot create KLM memory.");
            std::process::exit(1);
        }
    }
    match research::import(&mut kognitara, Path::new(RESEARCH)) {
        Ok(0) => {}
        Ok(count) => {
            if let Err(error) = persist_import(&kognitara, Path::new(DATA)) {
                eprintln!("Cannot save imported research: {error}");
                std::process::exit(1);
            }
            println!("Imported {count} unverified research observation(s).");
        }
        Err(error) => {
            eprintln!("Cannot import research: {error}");
            std::process::exit(1);
        }
    }
    relativistic_catchup_on_boot(&mut kognitara);

    // 2. Ensure the terminal owner (USER) is the founder
    let current_user = ki();
    if !kognitara.sanctuary.is_family(&current_user) {
        let founder = klm::family::FamilyMember::new(
            current_user.0.clone(), // ID
            current_user.0.clone(), // Name
            klm::family::FamilyRole::Parent,
        );
        kognitara.sanctuary.register_member(founder);
    }
    kog(&matches, &mut kognitara);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hypothesis_commands_record_evidence_and_corrections() {
        let mut mind = Kognitara::new("Test");
        assert_eq!(
            hypothesis_command(&mut mind, "hypothesis The sky is green").unwrap(),
            Ok(true)
        );
        assert_eq!(
            hypothesis_command(
                &mut mind,
                "evidence 1 | contradicts | observation | The sky is blue"
            )
            .unwrap(),
            Ok(true)
        );
        assert_eq!(
            hypothesis_command(
                &mut mind,
                "correct-hypothesis 1 | The sky is blue | observation"
            )
            .unwrap(),
            Ok(true)
        );
        assert!(
            hypothesis_command(&mut mind, "evidence 9 | supports | source | note")
                .unwrap()
                .is_err()
        );
        assert!(
            hypothesis_command(&mut mind, "evidence 1 | supports | | note")
                .unwrap()
                .is_err()
        );
        assert_eq!(mind.hypotheses.hypotheses[0].evidence[0].revision, 0);
        assert_eq!(mind.hypotheses.hypotheses[0].revision(), 1);
    }
}
