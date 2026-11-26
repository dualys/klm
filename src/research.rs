use klm::{BrowsingTrace, Kognitara};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const TOPICS: &[(&str, &str)] = &[
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

fn latest_snapshot(path: &Path) -> Result<Kognitara, String> {
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

fn topic_enabled(mind: &Kognitara, topic: &str, title: &str) -> bool {
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

fn next_topic(mind: &Kognitara, cursor: usize) -> Option<(usize, &'static str, &'static str)> {
    (0..TOPICS.len()).find_map(|offset| {
        let index = (cursor % TOPICS.len() + offset) % TOPICS.len();
        let (topic, title) = TOPICS[index];
        topic_enabled(mind, topic, title).then_some((index, topic, title))
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

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
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

fn encode_path(value: &str) -> String {
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
            &url,
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
    cycle_with(registry, directory, fetch_summary)
}

fn cycle_with(
    registry: &Path,
    directory: &Path,
    fetch: impl FnOnce(&str) -> Result<Vec<u8>, String>,
) -> Result<(), String> {
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
    let mind = latest_snapshot(registry)?;
    let cursor = read_cursor(directory)?;
    let Some((index, topic, title)) = next_topic(&mind, cursor) else {
        println!("No eligible public research topics.");
        return Ok(());
    };
    let observation = parse_summary(topic, &fetch(title)?)?;
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
    let path = directory.join(format!("{topic}-{}.json", hash.to_hex()));
    if !path.exists() {
        let data = serde_json::to_vec(&observation)
            .map_err(|e| format!("Cannot encode observation: {e}"))?;
        atomic_write(&path, &data)?;
    }
    atomic_write(
        &directory.join("cursor"),
        ((index + 1) % TOPICS.len()).to_string().as_bytes(),
    )?;
    println!("Recorded an unverified Wikipedia observation for {topic}.");
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
        assert!(next_topic(&mind, 0).is_none());
        mind.harvester.filter.allow_topic("tree");
        assert_eq!(next_topic(&mind, 0), Some((0, "tree", "Arbre")));
        assert_eq!(encode_path("Forêt"), "For%C3%AAt");
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
            cycle_with(&registry, &journal, |title| {
                assert_eq!(title, "Arbre");
                Ok(summary())
            })
            .unwrap();
        }
        assert_eq!(import(&mut mind, &journal).unwrap(), 1);
        assert_eq!(import(&mut mind, &journal).unwrap(), 0);
        for entry in fs::read_dir(&journal).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        fs::remove_dir(&journal).unwrap();
        fs::remove_file(registry).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
