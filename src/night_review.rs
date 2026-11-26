use crate::research;
use chrono::{DateTime, Timelike};
use klm::{EvidenceRelation, Kognitara, RelationType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

const CONCEPTS_PER_CYCLE: usize = 20;
const HYPOTHESES_PER_CYCLE: usize = 10;
const REQUEST_INTERVAL: i64 = 300;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct ReviewState {
    concept_cursor: Option<(u8, String)>,
    hypothesis_cursor: Option<u64>,
    pub concept_examinations: u64,
    pub hypothesis_examinations: u64,
    pub last_run: Option<i64>,
    last_request: Option<i64>,
    pub concepts: Vec<ConceptReview>,
    pub hypotheses: BTreeMap<String, HypothesisReview>,
    #[serde(default)]
    pub summaries: BTreeMap<String, NightSummary>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct NightSummary {
    pub cycles: u64,
    pub concept_examinations: u64,
    pub hypothesis_examinations: u64,
    pub last_run: i64,
    pub subjects: BTreeMap<String, NightSubject>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct NightSubject {
    pub label: String,
    pub kind: String,
    pub contradictions: Vec<String>,
}

fn record_subject(summary: &mut NightSummary, key: String, subject: NightSubject) {
    match summary.subjects.get_mut(&key) {
        Some(existing) => {
            for contradiction in subject.contradictions {
                if !existing.contradictions.contains(&contradiction) {
                    existing.contradictions.push(contradiction);
                }
            }
        }
        None => {
            summary.subjects.insert(key, subject);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ConceptReview {
    pub(crate) brain: u8,
    pub(crate) label: String,
    pub(crate) contradictions: Vec<String>,
    pub(crate) question: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HypothesisReview {
    pub(crate) id: u64,
    pub(crate) revision: usize,
    pub(crate) statement: String,
    pub(crate) examinations: u64,
    pub(crate) examined_at: i64,
    reports: usize,
    supports: usize,
    contradicts: usize,
    pub(crate) contradictions: Vec<String>,
    pub(crate) questions: Vec<String>,
    pub(crate) wikipedia: Option<WikipediaReview>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WikipediaReview {
    pub(crate) source: String,
    pub(crate) excerpt: String,
    examined_at: i64,
}

pub(crate) fn is_night<T: chrono::TimeZone>(time: DateTime<T>) -> bool {
    time.hour() < 8
}

pub(crate) fn read(directory: &Path) -> Result<Option<ReviewState>, String> {
    match fs::read(directory.join("night-review.state")) {
        Ok(data) => serde_json::from_slice(&data)
            .map(Some)
            .map_err(|error| format!("Invalid night review state: {error}")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Cannot read night review state: {error}")),
    }
}

fn save(directory: &Path, state: &ReviewState) -> Result<(), String> {
    let data = serde_json::to_vec(state)
        .map_err(|error| format!("Cannot encode night review state: {error}"))?;
    research::atomic_write(&directory.join("night-review.state"), &data)
}

fn batch<T: Ord + Clone>(keys: &[T], cursor: &Option<T>, limit: usize) -> Vec<T> {
    if keys.is_empty() {
        return Vec::new();
    }
    let start = cursor
        .as_ref()
        .and_then(|last| keys.iter().position(|key| key > last))
        .unwrap_or(0);
    (0..keys.len().min(limit))
        .map(|offset| keys[(start + offset) % keys.len()].clone())
        .collect()
}

fn examine(
    mind: &Kognitara,
    state: &mut ReviewState,
    now: i64,
    date: &str,
) -> Option<(u64, usize, &'static str, &'static str)> {
    let mut concepts = Vec::new();
    for (coord, brain) in &mind.swarm.brains {
        for id in brain.tree.concepts.keys() {
            concepts.push((coord.0, id.0.clone()));
        }
    }
    concepts.sort();
    let selected = batch(&concepts, &state.concept_cursor, CONCEPTS_PER_CYCLE);
    let summary = state.summaries.entry(date.to_string()).or_default();
    summary.cycles = summary.cycles.saturating_add(1);
    summary.last_run = now;
    summary.concept_examinations = summary
        .concept_examinations
        .saturating_add(selected.len() as u64);
    state.concepts.clear();
    for (brain_id, concept_id) in &selected {
        let brain = &mind.swarm.brains[&klm::NodeCoord(*brain_id)];
        let id = klm::ConceptId(concept_id.clone());
        let concept = &brain.tree.concepts[&id];
        let contradictions: Vec<String> = brain
            .tree
            .edges
            .iter()
            .filter(|edge| {
                edge.relation_type == RelationType::Contradiction
                    && (edge.source == id || edge.target == id)
            })
            .filter_map(|edge| {
                let other = if edge.source == id {
                    &edge.target
                } else {
                    &edge.source
                };
                brain
                    .tree
                    .concepts
                    .get(other)
                    .map(|other| other.label.clone())
            })
            .collect();
        record_subject(
            summary,
            format!("concept:{brain_id}:{concept_id}"),
            NightSubject {
                label: concept.label.clone(),
                kind: format!("Cerveau {brain_id}"),
                contradictions: contradictions.clone(),
            },
        );
        state.concepts.push(ConceptReview {
            brain: *brain_id,
            label: concept.label.clone(),
            contradictions,
            question: format!(
                "Que sais-je réellement de « {} » et quels liens restent à examiner ?",
                concept.label
            ),
        });
    }
    state.concept_examinations = state
        .concept_examinations
        .saturating_add(selected.len() as u64);
    if let Some(last) = selected.last() {
        state.concept_cursor = Some(last.clone());
    }

    let mut ids: Vec<_> = mind
        .hypotheses
        .hypotheses
        .iter()
        .map(|hypothesis| hypothesis.id)
        .collect();
    ids.sort_unstable();
    let selected = batch(&ids, &state.hypothesis_cursor, HYPOTHESES_PER_CYCLE);
    summary.hypothesis_examinations = summary
        .hypothesis_examinations
        .saturating_add(selected.len() as u64);
    let mut target = None;
    for id in &selected {
        let hypothesis = mind
            .hypotheses
            .hypotheses
            .iter()
            .find(|h| h.id == *id)
            .expect("selected hypothesis");
        let revision = hypothesis.revision();
        let evidence: Vec<_> = hypothesis
            .evidence
            .iter()
            .filter(|e| e.revision == revision)
            .collect();
        let mut contradictions = Vec::new();
        for brain in mind.swarm.brains.values() {
            for edge in &brain.tree.edges {
                if edge.relation_type == RelationType::Contradiction
                    && (hypothesis.related_concepts.contains(&edge.source)
                        || hypothesis.related_concepts.contains(&edge.target))
                {
                    if let (Some(left), Some(right)) = (
                        brain.tree.concepts.get(&edge.source),
                        brain.tree.concepts.get(&edge.target),
                    ) {
                        contradictions.push(format!("{} / {}", left.label, right.label));
                    }
                }
            }
        }
        let key = format!("{id}:{revision}");
        let previous = state
            .hypotheses
            .remove(&key)
            .filter(|review| review.statement == hypothesis.statement);
        let examinations = previous
            .as_ref()
            .map_or(1, |review| review.examinations.saturating_add(1));
        let wikipedia = previous.and_then(|review| review.wikipedia);
        let contradicts = evidence
            .iter()
            .filter(|e| e.relation == EvidenceRelation::Contradicts)
            .count();
        let mut findings = contradictions.clone();
        if contradicts > 0 {
            findings.push(format!(
                "{contradicts} source(s) enregistrée(s) comme contradictoires"
            ));
        }
        record_subject(
            summary,
            key.clone(),
            NightSubject {
                label: hypothesis.statement.clone(),
                kind: format!("Hypothèse #{id} · révision {revision}"),
                contradictions: findings,
            },
        );
        state.hypotheses.insert(
            key,
            HypothesisReview {
                id: *id,
                revision,
                statement: hypothesis.statement.clone(),
                examinations,
                examined_at: now,
                reports: evidence
                    .iter()
                    .filter(|e| e.relation == EvidenceRelation::Reports)
                    .count(),
                supports: evidence
                    .iter()
                    .filter(|e| e.relation == EvidenceRelation::Supports)
                    .count(),
                contradicts,
                contradictions,
                questions: vec![
                    "Quelles sources rapportent cette proposition et que disent-elles ?".into(),
                    "Quelles raisons la soutiennent ou la contredisent ?".into(),
                    "Quelles informations manquent pour conclure ?".into(),
                ],
                wikipedia,
            },
        );
        if target.is_none() {
            for (topic, title) in research::TOPICS {
                let encoded = format!(
                    "https://fr.wikipedia.org/wiki/{}",
                    research::encode_path(title)
                );
                let plain = format!("https://fr.wikipedia.org/wiki/{title}");
                if research::topic_enabled(mind, topic, title)
                    && evidence
                        .iter()
                        .any(|e| e.source == encoded || e.source == plain)
                {
                    target = Some((*id, revision, *topic, *title));
                    break;
                }
            }
        }
    }
    state.hypothesis_examinations = state
        .hypothesis_examinations
        .saturating_add(selected.len() as u64);
    if let Some(last) = selected.last() {
        state.hypothesis_cursor = Some(*last);
    }
    state.last_run = Some(now);
    target
}

pub(crate) fn cycle(registry: &Path, directory: &Path) -> Result<(), String> {
    cycle_at(
        registry,
        directory,
        chrono::Local::now(),
        |mind, topic, title| {
            if !is_night(chrono::Local::now()) {
                return Err("Night review ended before the Wikipedia request could start".into());
            }
            research::review_summary(mind, topic, title)
        },
    )
}

fn cycle_at<T: chrono::TimeZone>(
    registry: &Path,
    directory: &Path,
    now: DateTime<T>,
    fetch: impl FnOnce(&Kognitara, &str, &str) -> Result<(String, String), String>,
) -> Result<(), String> {
    if !is_night(now.clone()) {
        return Ok(());
    }
    let _lock = research::worker_lock(directory)?;
    let mind = research::latest_snapshot(registry)?;
    let mut state = read(directory)?.unwrap_or_default();
    let timestamp = now.timestamp();
    let date = now.date_naive().to_string();
    let target = examine(&mind, &mut state, timestamp, &date);
    save(directory, &state)?;
    if let Some((id, revision, topic, title)) = target {
        if state
            .last_request
            .is_none_or(|last| timestamp.saturating_sub(last) >= REQUEST_INTERVAL)
        {
            state.last_request = Some(timestamp);
            save(directory, &state)?;
            let (source, excerpt) = fetch(&mind, topic, title)?;
            let expected = format!(
                "https://fr.wikipedia.org/wiki/{}",
                research::encode_path(title)
            );
            let plain = format!("https://fr.wikipedia.org/wiki/{title}");
            if source != expected && source != plain {
                return Err(format!(
                    "Wikipedia redirected to a different article: {source}"
                ));
            }
            state
                .hypotheses
                .get_mut(&format!("{id}:{revision}"))
                .expect("examined hypothesis")
                .wikipedia = Some(WikipediaReview {
                source,
                excerpt,
                examined_at: timestamp,
            });
            save(directory, &state)?;
        }
    }
    println!(
        "Night review: {} concepts and {} hypotheses examined in total; conclusions remain unverified.",
        state.concept_examinations, state.hypothesis_examinations
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn night_window_and_cursor_wrap() {
        let tz = chrono::FixedOffset::east_opt(3600).unwrap();
        assert!(is_night(
            tz.with_ymd_and_hms(2026, 11, 27, 0, 0, 0).unwrap()
        ));
        assert!(is_night(
            tz.with_ymd_and_hms(2026, 11, 27, 7, 59, 59).unwrap()
        ));
        assert!(!is_night(
            tz.with_ymd_and_hms(2026, 11, 27, 8, 0, 0).unwrap()
        ));
        assert_eq!(batch(&[1, 2, 3], &Some(2), 2), vec![3, 1]);
        assert_eq!(batch::<u64>(&[], &None, 2), Vec::<u64>::new());
    }

    #[test]
    fn examination_tracks_evidence_without_correcting_or_verifying() {
        let mut mind = Kognitara::new("Test");
        let id = mind.hypotheses.add("Une affirmation", vec![]).unwrap();
        mind.hypotheses
            .get_mut(id)
            .unwrap()
            .add_evidence(EvidenceRelation::Contradicts, "source", "note", 1)
            .unwrap();
        let mut state = ReviewState::default();
        assert!(examine(&mind, &mut state, 10, "2026-11-27").is_none());
        examine(&mind, &mut state, 20, "2026-11-27");
        let review = &state.hypotheses[&format!("{id}:0")];
        assert_eq!(review.examinations, 2);
        assert_eq!(review.contradicts, 1);
        assert_eq!(mind.hypotheses.hypotheses[0].revision(), 0);
        assert_eq!(mind.hypotheses.hypotheses[0].statement, "Une affirmation");
        assert!(state.concept_examinations > 0);
        let summary = &state.summaries["2026-11-27"];
        assert_eq!(summary.cycles, 2);
        assert_eq!(summary.hypothesis_examinations, 2);
        assert_eq!(summary.subjects[&format!("{id}:0")].contradictions.len(), 1);
        mind.hypotheses.get_mut(id).unwrap().evidence.clear();
        examine(&mind, &mut state, 30, "2026-11-27");
        examine(&mind, &mut state, 40, "2026-11-28");
        assert_eq!(state.summaries["2026-11-27"].cycles, 3);
        assert_eq!(
            state.summaries["2026-11-27"].subjects[&format!("{id}:0")]
                .contradictions
                .len(),
            1
        );
        assert_eq!(state.summaries["2026-11-28"].cycles, 1);
        assert!(
            state.summaries["2026-11-28"].subjects[&format!("{id}:0")]
                .contradictions
                .is_empty()
        );

        let mut old = serde_json::to_value(&state).unwrap();
        old.as_object_mut().unwrap().remove("summaries");
        let restored: ReviewState = serde_json::from_value(old).unwrap();
        assert!(restored.summaries.is_empty());
        assert_eq!(restored.hypothesis_examinations, 4);
    }

    #[test]
    fn only_known_public_source_titles_can_be_requested() {
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.allow_topic("tree");
        let id = mind.hypotheses.add("private text", vec![]).unwrap();
        let hypothesis = mind.hypotheses.get_mut(id).unwrap();
        hypothesis
            .add_evidence(
                EvidenceRelation::Reports,
                "https://fr.wikipedia.org/wiki/private_text",
                "note",
                0,
            )
            .unwrap();
        assert!(examine(&mind, &mut ReviewState::default(), 0, "2026-11-27").is_none());
        mind.hypotheses
            .get_mut(id)
            .unwrap()
            .add_evidence(
                EvidenceRelation::Reports,
                "https://fr.wikipedia.org/wiki/Arbre",
                "note",
                0,
            )
            .unwrap();
        assert_eq!(
            examine(&mind, &mut ReviewState::default(), 0, "2026-11-27"),
            Some((id, 0, "tree", "Arbre"))
        );
    }

    #[test]
    fn cycle_persists_results_and_limits_requests_even_after_failure() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "klm-night-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let registry = dir.join("data");
        let journal = dir.join("research");
        let mut mind = Kognitara::new("Test");
        mind.harvester.filter.allow_topic("tree");
        let id = mind.hypotheses.add("Une affirmation", vec![]).unwrap();
        mind.hypotheses
            .get_mut(id)
            .unwrap()
            .add_evidence(
                EvidenceRelation::Reports,
                "https://fr.wikipedia.org/wiki/Arbre",
                "note",
                0,
            )
            .unwrap();
        let mut snapshot = serde_json::to_vec(&klm::KpackRecipe::encode(
            &serde_json::to_vec(&mind).unwrap(),
        ))
        .unwrap();
        snapshot.push(b'\n');
        fs::write(&registry, snapshot).unwrap();
        let tz = chrono::FixedOffset::east_opt(3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 11, 27, 0, 0, 0).unwrap();
        assert!(
            cycle_at(&registry, &journal, start, |_, _, _| Err(
                "network failed".into()
            ))
            .is_err()
        );
        assert_eq!(
            read(&journal).unwrap().unwrap().hypotheses[&format!("{id}:0")].examinations,
            1
        );
        assert_eq!(
            read(&journal).unwrap().unwrap().summaries["2026-11-27"].cycles,
            1
        );
        cycle_at(
            &registry,
            &journal,
            start + chrono::Duration::seconds(299),
            |_, _, _| panic!("too early"),
        )
        .unwrap();
        cycle_at(
            &registry,
            &journal,
            start + chrono::Duration::seconds(300),
            |_, _, title| {
                assert_eq!(title, "Arbre");
                Ok((
                    "https://fr.wikipedia.org/wiki/Arbre".into(),
                    "Un arbre est une plante.".into(),
                ))
            },
        )
        .unwrap();
        let state = read(&journal).unwrap().unwrap();
        assert_eq!(state.hypotheses[&format!("{id}:0")].examinations, 3);
        assert_eq!(
            state.hypotheses[&format!("{id}:0")]
                .wikipedia
                .as_ref()
                .unwrap()
                .excerpt,
            "Un arbre est une plante."
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(journal.join("night-review.state"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        cycle_at(
            &registry,
            &journal,
            start + chrono::Duration::hours(8),
            |_, _, _| panic!("outside window"),
        )
        .unwrap();
        assert_eq!(
            read(&journal).unwrap().unwrap().hypotheses[&format!("{id}:0")].examinations,
            3
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
