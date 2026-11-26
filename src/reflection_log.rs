use crate::command_history::write_private_atomic;
use klm::{Kognitara, SocraticReflection};
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

const DAY_SECONDS: i64 = 24 * 60 * 60;

#[derive(Serialize)]
pub(crate) struct Entry<'a> {
    pub(crate) brain: u8,
    #[serde(flatten)]
    pub(crate) reflection: &'a SocraticReflection,
}

pub(crate) fn recent(mind: &Kognitara, now: i64) -> Vec<Entry<'_>> {
    let start = now.saturating_sub(DAY_SECONDS);
    let mut entries = Vec::new();
    for (coord, brain) in &mind.swarm.brains {
        for reflection in &brain.learner.socratic_engine.reflections {
            if (start..=now).contains(&reflection.created_at) {
                entries.push(Entry {
                    brain: coord.0,
                    reflection,
                });
            }
        }
    }
    entries.sort_by_key(|entry| (entry.reflection.created_at, entry.brain));
    entries
}

pub fn write(mind: &Kognitara, directory: &Path, now: i64) -> io::Result<()> {
    let mut bytes = Vec::new();
    for entry in recent(mind, now) {
        serde_json::to_writer(&mut bytes, &entry)?;
        bytes.push(b'\n');
    }
    write_private_atomic(&directory.join("reflections.jsonl"), &bytes)
}

pub fn display(mind: &Kognitara, now: i64, mut output: impl Write) -> io::Result<()> {
    let entries = recent(mind, now);
    if entries.is_empty() {
        writeln!(output, "No reflections in the last 24 hours.")?;
    }
    for entry in entries {
        let reflection = entry.reflection;
        let time = chrono::DateTime::from_timestamp(reflection.created_at, 0)
            .map(|time| time.with_timezone(&chrono::Local).to_rfc3339())
            .unwrap_or_else(|| reflection.created_at.to_string());
        writeln!(
            output,
            "[{time}] brain {} | {:?}\n  {}\n  Question: {}",
            entry.brain,
            reflection.stage,
            reflection.interior_monologue,
            reflection.maieutic_question
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klm::{NodeCoord, SocraticStage};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn journal_filters_and_sorts_last_day_and_is_private() {
        let mut mind = Kognitara::new("Test");
        let brain = mind.swarm.brains.get_mut(&NodeCoord::new(0)).unwrap();
        for (id, created_at) in [
            ("new", 100_000),
            ("old", 13_599),
            ("boundary", 13_600),
            ("future", 100_001),
        ] {
            brain
                .learner
                .socratic_engine
                .reflections
                .push(SocraticReflection {
                    id: id.into(),
                    stage: SocraticStage::Aporia,
                    interior_monologue: id.into(),
                    examined_hypothesis: "example".into(),
                    maieutic_question: "Why?".into(),
                    concepts: Vec::new(),
                    humility_index: 0.5,
                    created_at,
                });
        }
        let reflection = brain.learner.socratic_engine.reflections[0].clone();
        mind.swarm
            .brains
            .get_mut(&NodeCoord::new(1))
            .unwrap()
            .learner
            .socratic_engine
            .reflections
            .push(reflection);
        let dir = std::env::temp_dir().join(format!("klm-reflections-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        write(&mind, &dir, 100_000).unwrap();
        let journal = fs::read_to_string(dir.join("reflections.jsonl")).unwrap();
        let entries: Vec<serde_json::Value> = journal
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["id"], "boundary");
        assert_eq!(entries[1]["id"], "new");
        assert_eq!(entries[2]["id"], "new");
        assert_eq!(entries[1]["brain"], 0);
        assert_eq!(entries[2]["brain"], 1);
        assert_eq!(
            fs::metadata(dir.join("reflections.jsonl"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let mut display_output = Vec::new();
        display(&mind, 100_000, &mut display_output).unwrap();
        let display = String::from_utf8(display_output).unwrap();
        assert!(display.contains("boundary"));
        assert!(!display.contains("future"));
        fs::remove_dir_all(dir).unwrap();
    }
}
