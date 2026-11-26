use klm::{BrowsingTrace, Kognitara};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) const TOPICS: &[(&str, &str)] = &[
    ("tree", "Arbre"),
    ("forest", "Forêt"),
    ("earth", "Terre"),
    ("botany", "Botanique"),
    ("water", "Eau"),
    ("ecology", "Écologie"),
    ("agriculture", "Agriculture"),
    ("medicine", "Médecine"),
    ("mathematics", "Mathématiques"),
    ("physics", "Physique"),
    ("philosophy", "Philosophie"),
    ("rust", "Rust_(langage)"),
];

#[derive(Debug, Serialize, Deserialize)]
struct Observation {
    topic: String,
    trace: BrowsingTrace,
}

const MAX_PENDING: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingTopic {
    topic: String,
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

fn isolated_number(title: &str) -> bool {
    let title = title.trim();
    !title.is_empty() && title.chars().all(|c| c.is_ascii_digit())
}

fn learning_priority(mind: &Kognitara, questions: &[klm::OpenQuestion], title: &str) -> f32 {
    if isolated_number(title) {
        return -1.0;
    }
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
        let (topic, seed_title) = TOPICS[topic_index];
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
            let queue: ResearchQueue = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid research queue: {e}"))?;
            if queue.pending.len() > MAX_PENDING
                || queue.pending.iter().any(|item| {
                    !public_title(&item.title)
                        || !TOPICS.iter().any(|(topic, _)| *topic == item.topic)
                })
            {
                return Err("Invalid topics in research queue".into());
            }
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

fn enqueue(queue: &mut ResearchQueue, topic: &str, title: &str) {
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

fn parse_summary(topic: &str, body: &[u8]) -> Result<Observation, String> {
    #[derive(Deserialize)]
    struct Summary {
        #[serde(rename = "type")]
        kind: String,
        title: String,
        extract: String,
        content_urls: Urls,
    }
    #[derive(Deserialize)]
    struct Urls {
        desktop: Page,
    }
    #[derive(Deserialize)]
    struct Page {
        page: String,
    }
    let summary: Summary =
        serde_json::from_slice(body).map_err(|e| format!("Invalid Wikipedia response: {e}"))?;
    if summary.kind != "standard"
        || summary.title.trim().is_empty()
        || summary.extract.trim().is_empty()
        || !summary
            .content_urls
            .desktop
            .page
            .starts_with("https://fr.wikipedia.org/wiki/")
    {
        return Err("Wikipedia returned an unsupported or empty article.".into());
    }
    let excerpt: String = summary.extract.chars().take(4000).collect();
    Ok(Observation {
        topic: topic.to_string(),
        trace: BrowsingTrace::new(
            summary.content_urls.desktop.page,
            None,
            summary.title,
            excerpt,
            vec![topic.to_string()],
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

fn fetch_summary(title: &str) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://fr.wikipedia.org/api/rest_v1/page/summary/{}",
        encode_path(title)
    );
    fetch_url(&url)
}

fn fetch_links(title: &str) -> Result<Vec<String>, String> {
    let url = format!(
        "https://fr.wikipedia.org/w/api.php?action=query&prop=links&plnamespace=0&pllimit=100&format=json&formatversion=2&titles={}",
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

pub fn cycle(registry: &Path, directory: &Path) -> Result<(), String> {
    cycle_with(registry, directory, fetch_summary, fetch_links)
}

pub(crate) fn review_summary(
    mind: &Kognitara,
    topic: &str,
    title: &str,
) -> Result<(String, String), String> {
    let observation = parse_summary(topic, &fetch_summary(title)?)?;
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
    fetch: impl FnOnce(&str) -> Result<Vec<u8>, String>,
    links: impl FnOnce(&str) -> Result<Vec<String>, String>,
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
                let (topic, title) = TOPICS[index];
                if topic_enabled(&mind, topic, title) {
                    enqueue(&mut queue, topic, title);
                }
            }
            save_queue(directory, &queue)?;
            queue
        }
    };
    let pending_before = queue.pending.len();
    for (topic, title) in TOPICS {
        if topic_enabled(&mind, topic, title) {
            enqueue(&mut queue, topic, title);
        }
    }
    if queue.pending.len() != pending_before {
        save_queue(directory, &queue)?;
    }
    if queue.pending.is_empty() {
        println!("Research queue is empty.");
        return Ok(());
    }
    let (selected, topic_index) =
        select_pending(&queue, &mind).ok_or("No pending research topic is currently enabled")?;
    let item = queue.pending[selected].clone();
    let result = (|| {
        let observation = parse_summary(&item.topic, &fetch(&item.title)?)?;
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
        let related = links(&item.title)?;
        Ok(related)
    })();
    let related = match result {
        Ok(related) => related,
        Err(error) => {
            let mut failed = queue
                .pending
                .remove(selected)
                .ok_or("Selected topic disappeared")?;
            failed.failed = true;
            queue.pending.push_back(failed);
            queue.next_topic = (topic_index + 1) % TOPICS.len();
            save_queue(directory, &queue)?;
            return Err(error);
        }
    };
    queue
        .pending
        .remove(selected)
        .ok_or("Selected topic disappeared")?;
    queue.next_topic = (topic_index + 1) % TOPICS.len();
    queue.visited.insert(queue_key(&item.title));
    let enabled_topics = TOPICS
        .iter()
        .filter(|(topic, title)| topic_enabled(&mind, topic, title))
        .count();
    let per_topic_limit = MAX_PENDING.div_ceil(enabled_topics);
    for title in related {
        if !isolated_number(&title)
            && queue
                .pending
                .iter()
                .filter(|pending| pending.topic == item.topic)
                .count()
                < per_topic_limit
        {
            enqueue(&mut queue, &item.topic, &title);
        }
    }
    save_queue(directory, &queue)?;
    println!(
        "Recorded an unverified Wikipedia observation for {}; {} topics pending.",
        item.title,
        queue.pending.len()
    );
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
        if !TOPICS.iter().any(|(topic, _)| *topic == observation.topic)
            || !observation
                .trace
                .source_url
                .starts_with("https://fr.wikipedia.org/wiki/")
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

    fn summary() -> Vec<u8> {
        br#"{"type":"standard","title":"Arbre","extract":"Un arbre est une plante.","content_urls":{"desktop":{"page":"https://fr.wikipedia.org/wiki/Arbre"}}}"#.to_vec()
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
        assert!(
            learning_priority(&mind, &questions, "Known")
                > learning_priority(&mind, &questions, "1787")
        );

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
    fn failed_topics_do_not_block_other_items_and_remain_retryable() {
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.whitelisted_topics.clear();
        mind.harvester.filter.allow_topic("tree");
        let mut queue = ResearchQueue::default();
        enqueue(&mut queue, "tree", "Unknown topic");
        enqueue(&mut queue, "tree", "1787");
        queue.pending[0].failed = true;
        assert_eq!(select_pending(&queue, &mind).unwrap().0, 1);
        queue.pending[1].failed = true;
        assert_eq!(select_pending(&queue, &mind).unwrap().0, 0);
    }

    #[test]
    fn scheduler_preserves_old_queue_and_balances_domains() {
        let dir = test_directory();
        let registry = dir.join("registry");
        let journal = dir.join("journal");
        fs::create_dir(&journal).unwrap();
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.whitelisted_topics.clear();
        for (topic, _) in TOPICS {
            mind.harvester.filter.allow_topic(*topic);
        }
        let json = serde_json::to_vec(&mind).unwrap();
        let mut snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(&json)).unwrap();
        snapshot.push(b'\n');
        fs::write(&registry, snapshot).unwrap();

        let mut pending = Vec::new();
        for year in 1800..1889 {
            pending.push(serde_json::json!({"topic":"medicine", "title":year.to_string()}));
        }
        for (topic, title) in TOPICS.iter().filter(|(topic, _)| *topic != "medicine") {
            pending.push(serde_json::json!({"topic":topic, "title":title}));
        }
        fs::write(
            journal.join("queue.state"),
            serde_json::to_vec(&serde_json::json!({"pending":pending, "visited":[]})).unwrap(),
        )
        .unwrap();

        cycle_with(
            &registry,
            &journal,
            |title| {
                assert_eq!(title, "Arbre");
                Ok(summary())
            },
            |_| Ok(vec!["1787".into(), "Feuille".into()]),
        )
        .unwrap();
        let after = read_queue(&journal).unwrap().unwrap();
        assert_eq!(after.pending.len(), 100);
        assert_eq!(
            after
                .pending
                .iter()
                .filter(|item| item.topic == "medicine")
                .count(),
            89
        );
        assert!(!after.pending.iter().any(|item| item.title == "1787"));
        assert!(after.pending.iter().any(|item| item.title == "Feuille"));
        assert_eq!(after.next_topic, 1);
        cycle_with(
            &registry,
            &journal,
            |title| {
                assert_eq!(title, "Forêt");
                Err("offline".into())
            },
            |_| unreachable!(),
        )
        .unwrap_err();
        let mut after_failure = read_queue(&journal).unwrap().unwrap();
        assert!(
            after_failure
                .pending
                .iter()
                .any(|item| item.title == "Forêt")
        );
        assert_eq!(after_failure.pending.len(), 100);
        assert_eq!(after_failure.next_topic, 2);
        after_failure.next_topic = 7;
        save_queue(&journal, &after_failure).unwrap();
        cycle_with(
            &registry,
            &journal,
            |title| {
                assert_eq!(title, "1800");
                Ok(summary())
            },
            |_| Ok(vec!["Another medical article".into()]),
        )
        .unwrap();
        let after_medicine = read_queue(&journal).unwrap().unwrap();
        assert_eq!(
            after_medicine
                .pending
                .iter()
                .filter(|item| item.topic == "medicine")
                .count(),
            88
        );
        assert!(
            !after_medicine
                .pending
                .iter()
                .any(|item| item.title == "Another medical article")
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_bad_sources_and_disambiguation_pages() {
        assert!(parse_summary("tree", &summary()).is_ok());
        let wrong_source = br#"{"type":"standard","title":"x","extract":"x","content_urls":{"desktop":{"page":"https://example.org/"}}}"#;
        assert!(parse_summary("tree", wrong_source).is_err());
        let ambiguous = br#"{"type":"disambiguation","title":"x","extract":"x","content_urls":{"desktop":{"page":"https://fr.wikipedia.org/wiki/X"}}}"#;
        assert!(parse_summary("tree", ambiguous).is_err());
    }

    #[test]
    fn imports_once_and_preserves_unverified_source() {
        let dir = test_directory();
        let path = dir.join("tree-test.json");
        let observation = parse_summary("tree", &summary()).unwrap();
        atomic_write(&path, &serde_json::to_vec(&observation).unwrap()).unwrap();
        let mut mind = Kognitara::new("Test");
        assert_eq!(import(&mut mind, &dir).unwrap(), 1);
        assert_eq!(import(&mut mind, &dir).unwrap(), 0);
        let serialized = serde_json::to_vec(&mind).unwrap();
        let mut restored: Kognitara = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(import(&mut restored, &dir).unwrap(), 0);
        assert_eq!(
            restored.hypotheses.hypotheses[0].evidence[0].source,
            "https://fr.wikipedia.org/wiki/Arbre"
        );
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
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

    #[test]
    fn cycle_writes_a_public_observation_and_imports_only_once() {
        let dir = test_directory();
        let registry = dir.join("registry");
        let journal = dir.join("journal");
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.whitelisted_topics.clear();
        mind.harvester.filter.allow_topic("tree");
        mind.harvester.filter.allow_topic("a private note");
        let json = serde_json::to_vec(&mind).unwrap();
        let mut snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(&json)).unwrap();
        snapshot.push(b'\n');
        fs::write(&registry, snapshot).unwrap();
        for _ in 0..2 {
            cycle_with(
                &registry,
                &journal,
                |title| {
                    assert_eq!(title, "Arbre");
                    Ok(summary())
                },
                |_| Ok(Vec::new()),
            )
            .unwrap();
        }
        assert_eq!(import(&mut mind, &journal).unwrap(), 1);
        assert_eq!(import(&mut mind, &journal).unwrap(), 0);
        let mut restored: Kognitara =
            serde_json::from_slice(&serde_json::to_vec(&mind).unwrap()).unwrap();
        assert_eq!(import(&mut restored, &journal).unwrap(), 0);
        for concept in ["arbre", "tree"] {
            let page = restored.render_markdown_page(concept).unwrap();
            assert!(page.contains("Unverified hypothesis #"));
            assert!(page.contains("https://fr.wikipedia.org/wiki/Arbre"));
            assert!(page.contains("not fact verification"));
        }
        let (response, _) = restored.converse(&klm::MemberId::from("ki"), "Tell me about arbre");
        assert!(response.contains("Unverified hypothesis #"));
        assert!(response.contains("https://fr.wikipedia.org/wiki/Arbre"));
        for entry in fs::read_dir(&journal).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        fs::remove_dir(&journal).unwrap();
        fs::remove_file(registry).unwrap();
        fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn public_links_are_bounded_deduplicated_and_failures_are_retried() {
        let links = r#"{"query":{"pages":[{"links":[{"ns":0,"title":"Forêt"},{"ns":0,"title":"forêt"},{"ns":4,"title":"Projet:privé"},{"ns":0,"title":"Arbre"}]}]}}"#.as_bytes();
        let mut queue = ResearchQueue::default();
        enqueue(&mut queue, "tree", "Arbre");
        queue.visited.insert(queue_key("Arbre"));
        queue.pending.clear();
        for title in parse_links(links).unwrap() {
            enqueue(&mut queue, "tree", &title);
        }
        assert_eq!(queue.pending.len(), 1);
        assert_eq!(queue.pending[0].title, "Forêt");
        for i in 0..150 {
            enqueue(&mut queue, "tree", &format!("Public {i}"));
        }
        assert_eq!(queue.pending.len(), MAX_PENDING);

        let dir = test_directory();
        let registry = dir.join("registry");
        let journal = dir.join("journal");
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.whitelisted_topics.clear();
        mind.harvester.filter.allow_topic("tree");
        let json = serde_json::to_vec(&mind).unwrap();
        let mut snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(&json)).unwrap();
        snapshot.push(b'\n');
        fs::write(&registry, snapshot).unwrap();
        assert!(
            cycle_with(
                &registry,
                &journal,
                |_| Err("offline".into()),
                |_| Ok(vec![])
            )
            .is_err()
        );
        assert_eq!(
            read_queue(&journal).unwrap().unwrap().pending[0].title,
            "Arbre"
        );
        cycle_with(
            &registry,
            &journal,
            |_| Ok(summary()),
            |_| Ok(vec!["Forêt".into()]),
        )
        .unwrap();
        let restored = read_queue(&journal).unwrap().unwrap();
        assert_eq!(restored.pending[0].title, "Forêt");
        assert!(restored.visited.contains(&queue_key("Arbre")));
        fs::remove_dir_all(dir).unwrap();
    }
}
