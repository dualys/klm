use klm::{BrowsingTrace, Kognitara};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const ALLOWED_DOMAINS: &[&str] = &[
    "fr.wikipedia.org",
    "fr.wikisource.org",
    "fr.wikibooks.org",
    "fr.wikiversity.org",
];

pub(crate) const TOPICS: &[(&str, &str, &str)] = &[
    (
        "operating_systems",
        "fr.wikipedia.org",
        "Système_d'exploitation",
    ),
    ("astronomy", "fr.wikipedia.org", "Astronomie"),
    ("nature", "fr.wikipedia.org", "Nature"),
    ("cryptography", "fr.wikipedia.org", "Cryptographie"),
    ("earth", "fr.wikipedia.org", "Terre"),
    ("agriculture", "fr.wikipedia.org", "Agriculture"),
    ("medicine", "fr.wikipedia.org", "Médecine"),
    ("mathematics", "fr.wikipedia.org", "Mathématiques"),
    ("physics", "fr.wikipedia.org", "Physique"),
    ("philosophy", "fr.wikipedia.org", "Philosophie"),
    (
        "programmation",
        "fr.wikipedia.org",
        "Programmation_informatique",
    ),
    // --- Robotique, Électronique & Embarqué ---
    ("robotics", "fr.wikipedia.org", "Robotique"),
    ("electronics", "fr.wikipedia.org", "Électronique"),
    ("microcontrollers", "fr.wikipedia.org", "Microcontrôleur"),
    ("sensors", "fr.wikipedia.org", "Capteur"),
    ("actuators", "fr.wikipedia.org", "Actionneur"),
    ("control_theory", "fr.wikipedia.org", "Automatique"),
    ("embedded_systems", "fr.wikipedia.org", "Système_embarqué"),
    ("mechanics", "fr.wikipedia.org", "Mécanique_(science)"),
    ("electronics_book", "fr.wikibooks.org", "Électronique"),
    (
        "robotics_course",
        "fr.wikiversity.org",
        "Introduction_à_la_robotique",
    ),
    ("music", "fr.wikipedia.org", "Musique"),
    ("humans", "fr.wikipedia.org", "Homo_sapiens"),
    ("rust", "fr.wikipedia.org", "Rust_(langage)"),
    // --- Nouvelles bibliothèques Wikimedia ---
    ("fables", "fr.wikisource.org", "Fables_de_La_Fontaine"),
    (
        "philosophy_texts",
        "fr.wikisource.org",
        "Apologie_de_Socrate",
    ),
    ("rust_book", "fr.wikibooks.org", "Programmation_Rust"),
    ("logic_course", "fr.wikiversity.org", "Logique_mathématique"),
];

fn default_domain() -> String {
    String::from("fr.wikipedia.org")
}

#[derive(Debug, Serialize, Deserialize)]
struct Observation {
    topic: String,
    trace: BrowsingTrace,
}

const MAX_PENDING: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingTopic {
    topic: String,
    #[serde(default = "default_domain")]
    domain: String,
    title: String,
    #[serde(default)]
    failed: bool,
}
#[derive(Default, Serialize, Deserialize)]
struct ResearchQueue {
    pending: VecDeque<PendingTopic>,
    visited: BTreeSet<String>,
    #[serde(default)]
    next_topic: usize,
}

fn queue_key(title: &str) -> String {
    title.replace('_', " ").to_lowercase()
}

fn public_title(title: &str) -> bool {
    !title.trim().is_empty()
        && title.len() <= 255
        && !title.contains(':')
        && !title.chars().any(char::is_control)
}

fn chronological_noise(title: &str) -> bool {
    let t = title.trim().to_lowercase();
    if t.is_empty() {
        return true;
    }
    // Astéroïdes numérotés "(371) Bohemia" ou nombres purs/négatifs ("1571", "-399")
    if t.starts_with('(')
        || t.strip_prefix('-')
            .unwrap_or(&t)
            .chars()
            .all(|c| c.is_ascii_digit())
    {
        return true;
    }
    // Dates ("10 août", "12 juillet 2011", "1er février")
    let starts_with_day = t
        .split_whitespace()
        .next()
        .is_some_and(|w| w == "1er" || (w.len() <= 2 && w.chars().all(|c| c.is_ascii_digit())));

    starts_with_day
        || t.ends_with(" av. j.-c.")
        || t.contains(" en science")
        || t.contains(" en philosophie")
        || t.contains(" en littérature")
        || t.contains(" en informatique")
        || t.contains(" en jeu vidéo")
        || t.contains(" en sport")
        || t.contains(" dans les chemins de fer")
        || t.starts_with("années ")
}

fn learning_priority(mind: &Kognitara, questions: &[klm::OpenQuestion], title: &str) -> f32 {
    let title = title.replace('_', " ");
    let mut doubt: Option<f32> = None;
    for brain in mind.swarm.brains.values() {
        if let Some(concept) = brain.tree.resolve_exact(&title) {
            if questions
                .iter()
                .any(|question| question.related_concepts.contains(&concept.id))
            {
                return 2.0;
            }
            let value = concept.epistemic_doubt();
            doubt = Some(doubt.map_or(value, |current| current.max(value)));
        }
    }
    doubt.unwrap_or(1.0)
}

fn select_pending(queue: &ResearchQueue, mind: &Kognitara) -> Option<(usize, usize)> {
    let questions = mind.pending_questions();
    for offset in 0..TOPICS.len() {
        let topic_index = (queue.next_topic % TOPICS.len() + offset) % TOPICS.len();
        let (topic, _, seed_title) = TOPICS[topic_index];
        if !topic_enabled(mind, topic, seed_title) {
            continue;
        }
        let mut selected: Option<(usize, bool, f32)> = None;
        for (index, item) in queue.pending.iter().enumerate() {
            if item.topic != topic {
                continue;
            }
            let priority = learning_priority(mind, &questions, &item.title);
            if selected.is_none_or(|(_, failed, best)| {
                (failed && !item.failed) || (!failed && !item.failed && priority > best)
            }) {
                selected = Some((index, item.failed, priority));
            }
        }
        if let Some((index, _, _)) = selected {
            return Some((index, topic_index));
        }
    }
    None
}
fn read_queue(directory: &Path) -> Result<Option<ResearchQueue>, String> {
    match fs::read(directory.join("queue.state")) {
        Ok(bytes) => {
            let mut queue: ResearchQueue = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid research queue: {e}"))?;
            // On filtre proprement les anciens topics retirés de TOPICS ou devenus invalides
            // au lieu de bloquer tout le service avec une erreur.
            queue.pending.retain(|item| {
                public_title(&item.title)
                    && !chronological_noise(&item.title)
                    && TOPICS.iter().any(|(topic, _, _)| *topic == item.topic)
            });
            // On borne à MAX_PENDING au cas où la limite aurait changé
            queue.pending.truncate(MAX_PENDING);

            Ok(Some(queue))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Cannot read research queue: {e}")),
    }
}

fn save_queue(directory: &Path, queue: &ResearchQueue) -> Result<(), String> {
    let data = serde_json::to_vec(queue).map_err(|e| format!("Cannot encode queue: {e}"))?;
    atomic_write(&directory.join("queue.state"), &data)
}

fn enqueue(queue: &mut ResearchQueue, topic: &str, domain: &str, title: &str) {
    let key = queue_key(title);
    if queue.pending.len() < MAX_PENDING
        && public_title(title)
        && !queue.visited.contains(&key)
        && !queue
            .pending
            .iter()
            .any(|item| queue_key(&item.title) == key)
    {
        queue.pending.push_back(PendingTopic {
            domain: domain.to_owned(),
            topic: topic.to_owned(),
            title: title.to_owned(),
            failed: false,
        });
    }
}

fn parse_links(body: &[u8]) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct Response {
        query: Query,
    }
    #[derive(Deserialize)]
    struct Query {
        pages: Vec<Page>,
    }
    #[derive(Deserialize)]
    struct Page {
        #[serde(default)]
        links: Vec<Link>,
    }
    #[derive(Deserialize)]
    struct Link {
        ns: i32,
        title: String,
    }
    let response: Response =
        serde_json::from_slice(body).map_err(|e| format!("Invalid Wikipedia links: {e}"))?;
    if response.query.pages.len() != 1 {
        return Err("Wikipedia links response must contain one page".into());
    }
    Ok(response
        .query
        .pages
        .into_iter()
        .flat_map(|page| page.links)
        .filter(|link| link.ns == 0 && public_title(&link.title))
        .map(|link| link.title)
        .collect())
}

pub(crate) fn latest_snapshot(path: &Path) -> Result<Kognitara, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open registry: {e}"))?;
    let mut reader = BufReader::new(file);
    let mut latest = Vec::new();
    let mut line = Vec::new();
    loop {
        line.clear();
        let count = reader
            .read_until(b'\n', &mut line)
            .map_err(|e| format!("Cannot read registry: {e}"))?;
        if count == 0 {
            break;
        }
        if line.last() == Some(&b'\n') {
            std::mem::swap(&mut latest, &mut line);
        }
    }
    if latest.is_empty() {
        return Err("Registry has no complete snapshot.".into());
    }
    let recipe: klm::KpackRecipe =
        serde_json::from_slice(&latest).map_err(|e| format!("Invalid last snapshot: {e}"))?;
    let json = recipe
        .decode()
        .map_err(|e| format!("Cannot decode snapshot: {e}"))?;
    serde_json::from_slice(&json).map_err(|e| format!("Cannot restore snapshot: {e}"))
}

pub(crate) fn topic_enabled(mind: &Kognitara, topic: &str, title: &str) -> bool {
    if mind.harvester.filter.whitelisted_topics.contains(topic) {
        return true;
    }
    mind.pending_questions().iter().any(|question| {
        question.related_concepts.iter().any(|id| {
            mind.swarm.brains.values().any(|brain| {
                brain.tree.get_concept(id).is_some_and(|concept| {
                    concept.label.eq_ignore_ascii_case(topic)
                        || concept.label.eq_ignore_ascii_case(title)
                })
            })
        })
    })
}
fn parse_summary(domain: &str, topic: &str, body: &[u8]) -> Result<Observation, String> {
    #[derive(Deserialize)]
    struct Response {
        query: Query,
    }
    #[derive(Deserialize)]
    struct Query {
        pages: Vec<Page>,
    }
    #[derive(Deserialize)]
    struct Page {
        #[serde(default)]
        missing: bool,
        #[serde(default)]
        title: String,
        #[serde(default)]
        extract: String,
        #[serde(default)]
        fullurl: String,
        #[serde(default)]
        pageprops: Option<serde_json::Value>,
    }

    if !ALLOWED_DOMAINS.contains(&domain) {
        return Err(format!("Unsupported Wikimedia domain: {domain}"));
    }

    let response: Response =
        serde_json::from_slice(body).map_err(|e| format!("Invalid MediaWiki response: {e}"))?;
    let page = response
        .query
        .pages
        .into_iter()
        .next()
        .ok_or("Empty MediaWiki response")?;

    let is_disambig = page
        .pageprops
        .as_ref()
        .and_then(|p| p.get("disambiguation"))
        .is_some();

    if page.missing
        || is_disambig
        || page.title.trim().is_empty()
        || page.extract.trim().is_empty()
        || !page.fullurl.starts_with(&format!("https://{domain}/wiki/"))
    {
        return Err("Wikipedia returned an unsupported or empty article.".into());
    }

    let excerpt: String = page.extract.chars().take(4000).collect();
    Ok(Observation {
        topic: topic.to_string(),
        trace: BrowsingTrace::new(
            page.fullurl,
            None,
            page.title,
            excerpt,
            vec![topic.to_string(), domain.to_string()],
        ),
    })
}

pub(crate) fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temp)
        .map_err(|e| format!("Cannot create {}: {e}", temp.display()))?;
    let result = (|| -> io::Result<()> {
        file.write_all(contents)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if let Err(error) = result {
        if let Err(cleanup) = fs::remove_file(&temp) {
            return Err(format!(
                "Cannot write {}: {error}; cleanup failed: {cleanup}",
                path.display()
            ));
        }
        return Err(format!("Cannot write {}: {error}", path.display()));
    }
    Ok(())
}

fn read_cursor(directory: &Path) -> Result<usize, String> {
    match fs::read_to_string(directory.join("cursor")) {
        Ok(value) => value
            .trim()
            .parse()
            .map_err(|e| format!("Invalid research cursor: {e}")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(format!("Cannot read research cursor: {e}")),
    }
}

pub(crate) fn encode_path(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn fetch_summary(domain: &str, title: &str) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://{domain}/w/api.php?action=query&prop=extracts|info|pageprops&exintro=1&explaintext=1&exchars=4000&inprop=url&redirects=1&format=json&formatversion=2&titles={}",
        encode_path(title)
    );
    fetch_url(&url)
}

fn fetch_links(domain: &str, title: &str) -> Result<Vec<String>, String> {
    let url = format!(
        "https://{domain}/w/api.php?action=query&prop=links&plnamespace=0&pllimit=100&format=json&formatversion=2&titles={}",
        encode_path(title)
    );
    parse_links(&fetch_url(&url)?)
}

fn fetch_url(url: &str) -> Result<Vec<u8>, String> {
    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--globoff",
            "--proto",
            "=https",
            "--max-time",
            "30",
            "--connect-timeout",
            "10",
            "--max-filesize",
            "1048576",
            "--user-agent",
            "klm/0.2 (https://github.com/dualys/klm)",
            "--url",
            url,
        ])
        .output()
        .map_err(|e| format!("Cannot start curl: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Wikipedia request failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}
const ARTICLES_PER_CYCLE: usize = 3;

pub fn cycle(registry: &Path, directory: &Path) -> Result<(), String> {
    cycle_with(registry, directory, fetch_summary, fetch_links)
}

pub(crate) fn review_summary(
    mind: &Kognitara,
    topic: &str,
    domain: &str,
    title: &str,
) -> Result<(String, String), String> {
    let observation = parse_summary(domain, topic, &fetch_summary(domain, title)?)?;
    let verdict = mind
        .harvester
        .filter
        .evaluate(&observation.trace.source_url, &observation.trace.snippet);
    if !verdict.is_allowed() {
        return Err(format!("Research source rejected by filter: {verdict:?}"));
    }
    Ok((observation.trace.source_url, observation.trace.snippet))
}

pub(crate) fn worker_lock(directory: &Path) -> Result<File, String> {
    fs::create_dir_all(directory).map_err(|e| format!("Cannot create research directory: {e}"))?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("worker.lock"))
        .map_err(|e| format!("Cannot open research lock: {e}"))?;
    lock.try_lock()
        .map_err(|e| format!("Another research cycle is running: {e}"))?;
    Ok(lock)
}

fn cycle_with(
    registry: &Path,
    directory: &Path,
    fetch: impl Fn(&str, &str) -> Result<Vec<u8>, String>,
    links: impl Fn(&str, &str) -> Result<Vec<String>, String>,
) -> Result<(), String> {
    let _lock = worker_lock(directory)?;
    let mind = latest_snapshot(registry)?;
    let mut queue = match read_queue(directory)? {
        Some(queue) => queue,
        None => {
            let mut queue = ResearchQueue::default();
            let cursor = read_cursor(directory)? % TOPICS.len();
            for offset in 0..TOPICS.len() {
                let index = (cursor + offset) % TOPICS.len();
                let (topic, domain, title) = TOPICS[index];
                if topic_enabled(&mind, topic, title) {
                    enqueue(&mut queue, topic, domain, title);
                }
            }
            save_queue(directory, &queue)?;
            queue
        }
    };
    let pending_before = queue.pending.len();
    for (topic, domain, title) in TOPICS {
        if topic_enabled(&mind, topic, title) {
            enqueue(&mut queue, topic, domain, title);
        }
    }
    if queue.pending.len() != pending_before {
        save_queue(directory, &queue)?;
    }
    if queue.pending.is_empty() {
        println!("Research queue is empty.");
        return Ok(());
    }
    let mut recorded = 0;
    for step in 0..ARTICLES_PER_CYCLE {
        let Some((selected, topic_index)) = select_pending(&queue, &mind) else {
            break;
        };
        if step > 0 {
            std::thread::sleep(std::time::Duration::from_millis(1200));
        }
        let item = queue.pending[selected].clone();
        let result = (|| {
            let observation = parse_summary(
                &item.domain,
                &item.topic,
                &fetch(&item.domain, &item.title)?,
            )?;
            let verdict = mind
                .harvester
                .filter
                .evaluate(&observation.trace.source_url, &observation.trace.snippet);
            if !verdict.is_allowed() {
                return Err(format!("Research source rejected by filter: {verdict:?}"));
            }
            let hash = blake3::hash(
                format!(
                    "{}\n{}",
                    observation.trace.source_url, observation.trace.snippet
                )
                .as_bytes(),
            );
            let path = directory.join(format!("{}-{}.json", item.topic, hash.to_hex()));
            if !path.exists() {
                let data = serde_json::to_vec(&observation)
                    .map_err(|e| format!("Cannot encode observation: {e}"))?;
                atomic_write(&path, &data)?;
            }
            let related = links(&item.domain, &item.title)?;
            Ok(related)
        })();

        match result {
            Ok(related) => {
                queue
                    .pending
                    .remove(selected)
                    .ok_or("Selected topic disappeared")?;
                queue.next_topic = (topic_index + 1) % TOPICS.len();
                queue.visited.insert(queue_key(&item.title));
                let enabled_topics = TOPICS
                    .iter()
                    .filter(|(topic, _, title)| topic_enabled(&mind, topic, title))
                    .count()
                    .max(1);
                let per_topic_limit = MAX_PENDING.div_ceil(enabled_topics);
                for title in related {
                    if !chronological_noise(&title)
                        && queue
                            .pending
                            .iter()
                            .filter(|pending| pending.topic == item.topic)
                            .count()
                            < per_topic_limit
                    {
                        enqueue(&mut queue, &item.topic, &item.domain, &title);
                    }
                }
                recorded += 1;
                println!(
                    "Recorded an unverified Wikipedia observation for {}; {} topics pending.",
                    item.title,
                    queue.pending.len()
                );
            }
            Err(error) => {
                let mut failed = queue
                    .pending
                    .remove(selected)
                    .ok_or("Selected topic disappeared")?;
                queue.next_topic = (topic_index + 1) % TOPICS.len();
                // Si la page n'existe pas (404) ou est une page d'homonymie/vide, on l'écarte définitivement
                if error.contains("404") || error.contains("unsupported or empty article") {
                    queue.visited.insert(queue_key(&failed.title));
                } else {
                    failed.failed = true;
                    queue.pending.push_back(failed);
                }
                save_queue(directory, &queue)?;
                if recorded == 0 {
                    return Err(error);
                }
                break;
            }
        }
    }
    save_queue(directory, &queue)?;
    Ok(())
}

pub fn import(mind: &mut Kognitara, directory: &Path) -> Result<usize, String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(format!("Cannot read research journal: {e}")),
    };
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Cannot list research journal: {e}"))?;
    paths.sort();
    let mut count = 0;
    for path in paths {
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid research filename")?
            .to_string();
        if mind.imported_research.contains(&id) {
            continue;
        }
        let data = fs::read(&path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        let observation: Observation = serde_json::from_slice(&data)
            .map_err(|e| format!("Invalid {}: {e}", path.display()))?;
        if observation.topic.trim().is_empty()
            || !ALLOWED_DOMAINS.iter().any(|d| {
                observation
                    .trace
                    .source_url
                    .starts_with(&format!("https://{d}/wiki/"))
            })
        {
            return Err(format!("Invalid research source in {}", path.display()));
        }
        let report = mind
            .harvest_browsing_trace(&observation.trace)
            .map_err(|e| e.to_string())?;
        if !report.verdict.is_allowed() || report.woven_concepts.is_empty() {
            return Err(format!(
                "Cannot import {}: source rejected or no active brain",
                path.display()
            ));
        }
        mind.imported_research.insert(id);
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn test_directory() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "klm-research-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn only_public_predefined_topics_can_be_sent() {
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.whitelisted_topics.clear();
        mind.harvester
            .filter
            .allow_topic("a private note about trees");
        assert!(!topic_enabled(&mind, "tree", "Arbre"));
        mind.harvester.filter.allow_topic("tree");
        assert!(topic_enabled(&mind, "tree", "Arbre"));
        assert_eq!(encode_path("Forêt"), "For%C3%AAt");
    }

    #[test]
    fn learning_priority_uses_questions_and_local_knowledge() {
        let mut mind = Kognitara::new("Test");
        let brain = mind.swarm.brains.get_mut(&klm::NodeCoord::new(0)).unwrap();
        let mut known = klm::Concept::new("known", "Known", "", klm::ConceptOrigin::Innate);
        known.confidence = 0.9;
        brain.tree.add_concept(known);
        let questions = mind.pending_questions();
        assert!(
            learning_priority(&mind, &questions, "Unseen")
                > learning_priority(&mind, &questions, "Known")
        );
        assert!(chronological_noise("1787"));
        let brain = mind.swarm.brains.get_mut(&klm::NodeCoord::new(0)).unwrap();

        brain
            .learner
            .curiosity_engine
            .questions
            .push(klm::OpenQuestion {
                id: "q-test".into(),
                related_concepts: vec![klm::ConceptId::from("known")],
                question_text: "Unanswered".into(),
                urgency: 0.9,
                is_resolved: false,
                resolution_insight: None,
                created_at: 0,
            });
        let questions = mind.pending_questions();
        assert!(
            learning_priority(&mind, &questions, "Known")
                > learning_priority(&mind, &questions, "Unseen")
        );
    }

    #[test]
    fn reads_last_complete_snapshot_while_append_is_in_progress() {
        let dir = test_directory();
        let path = dir.join("registry");
        let mind = Kognitara::new("Test");
        let json = serde_json::to_vec(&mind).unwrap();
        let snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(&json)).unwrap();
        let mut contents = snapshot;
        contents.extend_from_slice(b"\n{unfinished");
        fs::write(&path, contents).unwrap();
        assert_eq!(
            latest_snapshot(&path).unwrap().sanctuary.sanctuary_name,
            "Test"
        );
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
