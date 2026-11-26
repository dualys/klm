use clap::Arg;
use clap::{ArgMatches, Command};
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType, size};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Editor, Select, Text};
use klm::family::MemberId;
use klm::{ConceptId, EvidenceRelation, KnowledgeTree, Kognitara, NodeCoord};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{Write, stdout};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;
mod command_history;
mod reflection_log;
mod research;
mod web_ui;
use inquire::validator::Validation;
use klm::{PULSAR_PRIMES, current_timestamp};

struct StoragePaths {
    data: PathBuf,
    research: PathBuf,
    tags: PathBuf,
    lock: PathBuf,
}

impl StoragePaths {
    fn new(root: Option<OsString>) -> Result<Self, &'static str> {
        let root = root.unwrap_or_else(|| OsString::from("/mnt/kognitara"));
        if root.is_empty() {
            return Err("KOGNITARA_PATH cannot be empty");
        }
        let root = PathBuf::from(root);
        Ok(Self {
            data: root.join("data"),
            research: root.join("research"),
            tags: root.join("tags.json"),
            lock: root.join("registry.lock"),
        })
    }
}

static STORAGE_PATHS: OnceLock<StoragePaths> = OnceLock::new();

fn storage_paths() -> &'static StoragePaths {
    STORAGE_PATHS
        .get()
        .expect("storage paths initialized in main")
}

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
        .subcommand(Command::new("ui").about("Start the local web interface"))
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
                    Command::new("delete")
                        .about("Delete a tag")
                        .arg(Arg::new("tag").required(true).help("Tag name to delete")),
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
            Command::new("compact")
                .about("Keep the latest and tagged snapshots, removing untagged history"),
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

    match klm::storage::registry::export_tag(
        &storage_paths().data,
        &storage_paths().tags,
        tag_name,
        Path::new(dest_path),
    ) {
        Ok(records) => ok(&format!(
            "Exported {records} retained snapshot(s) through tag '{tag_name}' to '{dest_path}'."
        )),
        Err(error) => ko(&format!("Cannot export tag '{tag_name}': {error}")),
    }
}

fn tag_state(kognitara: &Kognitara, tag_name: &str) {
    let tag_name = tag_name.trim();
    if tag_name.is_empty() {
        ko("You must provide a tag name (e.g., 'tag v1.0').");
        return;
    }

    if let Err(error) = persist_import(kognitara, &storage_paths().data) {
        ko(&format!("Cannot save state before tagging: {error}"));
        return;
    }
    match klm::storage::registry::create_tag(&storage_paths().data, &storage_paths().tags, tag_name)
    {
        Ok(id) => ok(&format!(
            "Tag '{}' now points to snapshot {}.",
            tag_name, id
        )),
        Err(error) => ko(&format!("Cannot create tag '{tag_name}': {error}")),
    }
}

fn list_tags() {
    match klm::storage::registry::read_tags(&storage_paths().data, &storage_paths().tags) {
        Ok(tags) if tags.is_empty() => ko("No tags found."),
        Ok(tags) => {
            ok("Here are the saved snapshot tags:");
            for (name, id) in tags {
                println!("  * [{name}] -> {id}");
            }
        }
        Err(error) => ko(&format!("Cannot read tags: {error}")),
    }
}

fn delete_tag(tag_name: &str) {
    match klm::storage::registry::delete_tag(
        &storage_paths().data,
        &storage_paths().tags,
        tag_name.trim(),
    ) {
        Ok(true) => ok(&format!(
            "Tag '{tag_name}' deleted. Run 'compact' to reclaim unreferenced snapshots."
        )),
        Ok(false) => ko(&format!("Tag '{tag_name}' does not exist.")),
        Err(error) => ko(&format!("Cannot delete tag '{tag_name}': {error}")),
    }
}

fn compact_registry() -> std::io::Result<()> {
    let report =
        klm::storage::registry::compact_registry(&storage_paths().data, &storage_paths().tags)?;
    let freed = report.bytes_before.saturating_sub(report.bytes_after);
    println!(
        "Compacted {} snapshots to {}; kept {} tag(s), reclaimed {freed} bytes.",
        report.records_before, report.records_after, report.tags_kept
    );
    Ok(())
}

fn tag_delete_argument(input: &str) -> Option<&str> {
    let (command, rest) = input.trim_start().split_once(char::is_whitespace)?;
    if !command.eq_ignore_ascii_case("tag") {
        return None;
    }
    let (action, name) = rest.trim_start().split_once(char::is_whitespace)?;
    if action.eq_ignore_ascii_case("delete") {
        Some(name.trim())
    } else {
        None
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
        pack(kognitara, &storage_paths().data);
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
    pack(kognitara, &storage_paths().data);
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
    pack(kognitara, &storage_paths().data);
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

fn equivalence_tree<'a>(
    mind: &'a mut Kognitara,
    brain: &str,
) -> Result<&'a mut KnowledgeTree, String> {
    let node = brain
        .trim()
        .parse::<u8>()
        .map_err(|_| "Brain must be a number from 0 to 7.")?;
    if node > 7 {
        return Err("Brain must be a number from 0 to 7.".into());
    }
    mind.swarm
        .brains
        .get_mut(&NodeCoord::new(node))
        .map(|brain| &mut brain.tree)
        .ok_or_else(|| format!("Brain {node} not found."))
}

fn equivalence_command(mind: &mut Kognitara, input: &str) -> Option<Result<bool, String>> {
    let (command, args) = input.split_once(' ').unwrap_or((input, ""));
    match command {
        "equiv-list" => Some((|| {
            let tree = equivalence_tree(mind, args)?;
            if tree.possible_equivalences.is_empty() {
                println!("No possible equivalences in this brain.");
            }
            for link in &tree.possible_equivalences {
                println!("{}", link.display());
                if !link.endpoints_match(tree) {
                    println!("  A concept is absent or has changed; review this proposal.");
                }
                for change in &link.changes {
                    println!(
                        "  {}: {} | timestamp {}",
                        if change.active {
                            "reopened"
                        } else {
                            "withdrawn"
                        },
                        change.reason,
                        change.changed_at
                    );
                }
            }
            Ok(false)
        })()),
        "equiv-suggest" => Some((|| {
            let (brain, concept) = args
                .split_once('|')
                .ok_or("Usage: equiv-suggest <brain> | <concept ID>")?;
            let tree = equivalence_tree(mind, brain)?;
            let count = tree
                .suggest_equivalences(&ConceptId::from(concept.trim()))
                .map_err(|e| e.to_string())?;
            println!("{count} unverified same-label proposal(s); no concepts were merged.");
            Ok(count > 0)
        })()),
        "equiv-propose" => Some((|| {
            let parts: Vec<&str> = args.splitn(5, '|').map(str::trim).collect();
            if parts.len() != 5 {
                return Err(
                    "Usage: equiv-propose <brain> | <ID> | <ID> | <source> | <reason>".into(),
                );
            }
            let tree = equivalence_tree(mind, parts[0])?;
            let id = tree
                .propose_equivalence(
                    &ConceptId::from(parts[1]),
                    &ConceptId::from(parts[2]),
                    parts[3],
                    parts[4],
                )
                .map_err(|e| e.to_string())?;
            println!(
                "Possible equivalence #{id} recorded as unverified; concepts remain separate."
            );
            Ok(true)
        })()),
        "equiv-withdraw" | "equiv-reopen" => Some((|| {
            let parts: Vec<&str> = args.splitn(3, '|').map(str::trim).collect();
            if parts.len() != 3 {
                return Err(format!(
                    "Usage: {command} <brain> | <proposal ID> | <reason>"
                ));
            }
            let id = parts[1]
                .parse::<u64>()
                .map_err(|_| "Invalid proposal ID.")?;
            let tree = equivalence_tree(mind, parts[0])?;
            tree.set_equivalence_active(id, command == "equiv-reopen", parts[2])
                .map_err(|e| e.to_string())?;
            println!("Proposal status updated; history retained.");
            Ok(true)
        })()),
        _ => None,
    }
}

fn interact(kognitara: &mut Kognitara) {
    let history = match command_history::state_dir().and_then(command_history::CommandHistory::new)
    {
        Ok(history) => Some(history),
        Err(error) => {
            eprintln!("Command history unavailable: {error}");
            None
        }
    };
    let mut editor = match rustyline::Config::builder()
        .max_history_size(command_history::HISTORY_LIMIT)
        .and_then(|config| rustyline::DefaultEditor::with_config(config.build()))
    {
        Ok(editor) => editor,
        Err(error) => {
            eprintln!("Cannot start command editor: {error}");
            return;
        }
    };
    let history = history.and_then(|history| match history.load() {
        Ok(entries) => {
            for entry in entries {
                if let Err(error) = editor.add_history_entry(entry) {
                    eprintln!("Cannot load command history entry: {error}");
                }
            }
            Some(history)
        }
        Err(error) => {
            eprintln!("Cannot read command history; history disabled: {error}");
            None
        }
    });
    loop {
        let todo = match editor.readline("> ") {
            Ok(input) => input,
            Err(
                rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof,
            ) => {
                pack(kognitara, &storage_paths().data);
                break;
            }
            Err(error) => {
                eprintln!("Command input failed: {error}");
                break;
            }
        };
        let trimmed = todo.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(history) = &history {
            match history.record(trimmed) {
                Ok(true) => {
                    if let Err(error) = editor.add_history_entry(trimmed) {
                        eprintln!("Cannot update command history: {error}");
                    }
                }
                Ok(false) => {}
                Err(error) => eprintln!("Cannot save command history: {error}"),
            }
        }
        if let Some(result) = hypothesis_command(kognitara, trimmed)
            .or_else(|| equivalence_command(kognitara, trimmed))
        {
            match result {
                Ok(true) => pack(kognitara, &storage_paths().data),
                Ok(false) => {}
                Err(error) => ko(&error),
            }
        } else if trimmed.eq_ignore_ascii_case("pack") {
            pack(kognitara, &storage_paths().data);
        } else if trimmed.eq_ignore_ascii_case("nap") || trimmed.eq_ignore_ascii_case("sieste") {
            for msg in kognitara.nap_cycle() {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
        } else if trimmed.eq_ignore_ascii_case("rest") || trimmed.eq_ignore_ascii_case("repos") {
            for msg in kognitara.rest_cycle() {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
        } else if trimmed.to_lowercase().starts_with("contemplate") {
            let steps = trimmed[11..].trim().parse::<usize>().unwrap_or(5);
            for msg in kognitara.contemplate_cycle(steps) {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
        } else if trimmed.eq_ignore_ascii_case("hibernate") {
            for msg in kognitara.hibernate_cycle() {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
        } else if trimmed.eq_ignore_ascii_case("compact") {
            if let Err(error) = compact_registry() {
                ko(&format!("Registry compaction failed: {error}"));
            }
        } else if let Some(tag_name) = tag_delete_argument(trimmed) {
            if tag_name.is_empty() {
                ko("Usage: tag delete <name>");
            } else {
                delete_tag(tag_name);
            }
        } else if trimmed.eq_ignore_ascii_case("tags") {
            list_tags();
        } else if trimmed.eq_ignore_ascii_case("tag") {
            tag_state(
                kognitara,
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
            pack(kognitara, &storage_paths().data);
        } else if trimmed.to_lowercase().starts_with("man ") {
            let concept = trimmed[4..].trim();
            if let Some(page) = kognitara.render_markdown_page(concept) {
                print!("{page}");
            } else {
                ko(&format!("Concept '{}' not found.", concept));
            }
        } else if trimmed.eq_ignore_ascii_case("unpack") {
            unpack(kognitara, &storage_paths().data);
        } else if trimmed.eq_ignore_ascii_case("teach") {
            teach_kognitara(kognitara);
        } else if trimmed.eq_ignore_ascii_case("status") {
            show_status(kognitara);
        } else if trimmed.eq_ignore_ascii_case("oracles") {
            show_oracles(kognitara);
        } else if trimmed.eq_ignore_ascii_case("reflections 24h") {
            if let Err(error) = reflection_log::display(kognitara, current_timestamp(), stdout()) {
                eprintln!("Cannot display reflections: {error}");
            }
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
            tag_state(kognitara, tag_name);
        } else if trimmed.eq_ignore_ascii_case("tags") {
            list_tags();
        } else if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
            pack(kognitara, &storage_paths().data);
            break;
        } else if trimmed.eq_ignore_ascii_case("dream") {
            dream(kognitara);
        } else {
            let current_user = ki();
            let (response, _activated) = kognitara.converse(&current_user, trimmed);
            println!("\r\n{response}\n");
            pack(kognitara, &storage_paths().data);
        }
    }
}

fn ok(message: &str) {
    let (cols, _) = size().unwrap_or((80, 24));
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
    let (cols, _) = size().unwrap_or((80, 24));
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

fn pack(kognitara: &Kognitara, path: &Path) {
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
                            } else if let Err(error) = file.sync_all().and_then(|_| {
                                reflection_log::write(
                                    kognitara,
                                    path.parent().unwrap_or(Path::new(".")),
                                    current_timestamp(),
                                )
                            }) {
                                ko(&format!(
                                    "Snapshot written, but sync or reflection log failed: {error}"
                                ));
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
    reflection_log::write(
        kognitara,
        path.parent().unwrap_or(Path::new(".")),
        current_timestamp(),
    )
    .map_err(|e| format!("Snapshot saved, but cannot update reflection log: {e}"))?;
    Ok(())
}

fn unpack(kognitara: &mut Kognitara, path: &Path) -> bool {
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
                    if let Err(error) = reflection_log::write(
                        kognitara,
                        path.parent().unwrap_or(Path::new(".")),
                        current_timestamp(),
                    ) {
                        eprintln!("Registry restored, but cannot update reflection log: {error}");
                    }
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
        Some(("ui", _)) => {
            if let Err(error) = web_ui::serve(kognitara, &storage_paths().data, &ki()) {
                eprintln!("Web interface failed: {error}");
                std::process::exit(1);
            }
        }
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
            pack(kognitara, &storage_paths().data);
        }
        Some(("rest", _)) => {
            for msg in kognitara.rest_cycle() {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
        }
        Some(("contemplate", sub)) => {
            let steps = sub
                .get_one::<String>("steps")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(5);
            for msg in kognitara.contemplate_cycle(steps) {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
        }
        Some(("hibernate", _)) => {
            for msg in kognitara.hibernate_cycle() {
                ok(&msg);
            }
            pack(kognitara, &storage_paths().data);
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
            pack(kognitara, &storage_paths().data);
        }
        Some(("tag", sub)) => match sub.subcommand() {
            Some(("list", _)) => {
                list_tags();
            }
            Some(("create", _)) => {
                tag_state(
                    kognitara,
                    Text::new("Tag version:").prompt().expect("").as_str(),
                );
            }
            Some(("delete", sub)) => {
                let tag = sub.get_one::<String>("tag").expect("tag required");
                delete_tag(tag);
            }
            Some(("export", sub)) => {
                let tag = sub.get_one::<String>("tag").expect("tag it's required");
                let destination = sub
                    .get_one::<String>("destination")
                    .expect("destination it's required");
                export_state(tag, destination);
            }
            _ => {
                ko("tag commands are: list, create, delete, or export");
            }
        },
        Some(("compact", _)) => {
            if let Err(error) = compact_registry() {
                ko(&format!("Registry compaction failed: {error}"));
            }
        }
        Some(("pack", _)) => {
            pack(kognitara, &storage_paths().data);
        }
        Some(("dream", _)) => {
            dream(kognitara);
        }
        Some(("unpack", _)) => {
            unpack(kognitara, &storage_paths().data);
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
            pack(kognitara, &storage_paths().data);
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
        pack(kognitara, &storage_paths().data);
    } else {
        ko("failed to learn");
    }
}

fn main() {
    let matches = cli().get_matches();
    let paths = match StoragePaths::new(env::var_os("KOGNITARA_PATH")) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    if STORAGE_PATHS.set(paths).is_err() {
        eprintln!("Storage paths are already initialized");
        std::process::exit(1);
    }
    if matches.subcommand_name() == Some("research-cycle") {
        if let Err(error) = research::cycle(&storage_paths().data, &storage_paths().research) {
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
        .open(&storage_paths().lock)
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
    if matches.subcommand_name() == Some("compact") {
        if let Err(error) = compact_registry() {
            eprintln!("Registry compaction failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    let mut kognitara = Kognitara::new(
        env::var("KOGNITARA_SANCTUARY_NAME")
            .unwrap_or_else(|_| "Sanctuary of House Ki".to_string()),
    );
    if storage_paths().data.exists() {
        if !unpack(&mut kognitara, &storage_paths().data) {
            eprintln!("Cannot open KLM memory; refusing to import research.");
            std::process::exit(1);
        }
    } else {
        pack(&kognitara, &storage_paths().data);
        if !unpack(&mut kognitara, &storage_paths().data) {
            eprintln!("Cannot create KLM memory.");
            std::process::exit(1);
        }
    }
    match research::import(&mut kognitara, &storage_paths().research) {
        Ok(0) => {}
        Ok(count) => {
            if let Err(error) = persist_import(&kognitara, &storage_paths().data) {
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
    if let Err(error) = reflection_log::write(
        &kognitara,
        storage_paths().data.parent().unwrap_or(Path::new(".")),
        current_timestamp(),
    ) {
        eprintln!("Cannot update reflection log: {error}");
    }

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
    fn storage_root_defaults_and_rejects_empty_values() {
        let paths = StoragePaths::new(None).unwrap();
        assert_eq!(paths.data, Path::new("/mnt/kognitara/data"));
        assert_eq!(paths.research, Path::new("/mnt/kognitara/research"));
        assert_eq!(paths.tags, Path::new("/mnt/kognitara/tags.json"));
        assert_eq!(paths.lock, Path::new("/mnt/kognitara/registry.lock"));
        assert!(StoragePaths::new(Some(OsString::new())).is_err());
    }

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

    #[test]
    fn equivalence_commands_require_exact_ids_and_preserve_history() {
        let mut mind = Kognitara::new("Test");
        assert_eq!(
            equivalence_command(
                &mut mind,
                "equiv-propose 0 | tree | forest | local note | check meanings"
            ),
            Some(Ok(true))
        );
        assert!(
            equivalence_command(&mut mind, "equiv-propose 8 | tree | forest | s | r")
                .unwrap()
                .is_err()
        );
        assert!(
            equivalence_command(&mut mind, "equiv-propose 0 | tree | missing | s | r")
                .unwrap()
                .is_err()
        );
        assert_eq!(
            equivalence_command(&mut mind, "equiv-withdraw 0 | 1 | different meanings"),
            Some(Ok(true))
        );
        assert_eq!(
            equivalence_command(&mut mind, "equiv-list 0"),
            Some(Ok(false))
        );
        assert_eq!(
            equivalence_command(&mut mind, "equiv-reopen 0 | 1 | reconsider"),
            Some(Ok(true))
        );
        assert_eq!(
            mind.swarm.brains[&NodeCoord::new(0)]
                .tree
                .possible_equivalences[0]
                .changes
                .len(),
            2
        );
    }
}
