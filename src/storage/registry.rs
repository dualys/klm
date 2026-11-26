use crate::storage::KpackRecipe;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct Entry {
    line_number: usize,
    line: String,
    id: String,
    recipe: KpackRecipe,
}

enum StoredTags {
    Stable(BTreeMap<String, String>),
    Legacy(BTreeMap<String, usize>),
}

struct Scan {
    line_count: usize,
    latest: Option<Entry>,
    by_id: BTreeMap<String, Entry>,
    by_line: BTreeMap<usize, Entry>,
}

struct ResolvedTags {
    tags: BTreeMap<String, String>,
    entries: BTreeMap<String, Entry>,
    scan: Scan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionReport {
    pub records_before: usize,
    pub records_after: usize,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub tags_kept: usize,
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn normalize_id(id: &str) -> io::Result<String> {
    if id.len() != 64 || !id.is_ascii() {
        return Err(invalid_data(
            "Snapshot ID must be 64 hexadecimal characters",
        ));
    }

    let mut normalized = String::with_capacity(64);
    for pair in id.as_bytes().chunks_exact(2) {
        let pair = std::str::from_utf8(pair).map_err(|error| invalid_data(error.to_string()))?;
        let byte = u8::from_str_radix(pair, 16)
            .map_err(|_| invalid_data("Snapshot ID contains a non-hexadecimal character"))?;
        normalized.push_str(&format!("{byte:02x}"));
    }
    Ok(normalized)
}

fn recipe_id(recipe: &KpackRecipe) -> String {
    recipe
        .blake3_hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn read_stored_tags(path: &Path) -> io::Result<StoredTags> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(StoredTags::Stable(BTreeMap::new()));
        }
        Err(error) => return Err(error),
    };

    if let Ok(mut tags) = serde_json::from_str::<BTreeMap<String, String>>(&content) {
        for (name, id) in &mut tags {
            if name.trim().is_empty() {
                return Err(invalid_data("Tag names cannot be empty"));
            }
            *id = normalize_id(id)?;
        }
        return Ok(StoredTags::Stable(tags));
    }

    if let Ok(tags) = serde_json::from_str::<BTreeMap<String, usize>>(&content) {
        if tags.keys().any(|name| name.trim().is_empty()) {
            return Err(invalid_data("Tag names cannot be empty"));
        }
        if tags.values().any(|line| *line == 0) {
            return Err(invalid_data("Legacy tag line numbers must start at 1"));
        }
        return Ok(StoredTags::Legacy(tags));
    }

    Err(invalid_data(
        "Tags file must map names to snapshot IDs or legacy line numbers",
    ))
}

fn scan_registry(
    registry_path: &Path,
    wanted_ids: &BTreeSet<String>,
    wanted_lines: &BTreeSet<usize>,
) -> io::Result<Scan> {
    let file = File::open(registry_path)?;
    let mut scan = Scan {
        line_count: 0,
        latest: None,
        by_id: BTreeMap::new(),
        by_line: BTreeMap::new(),
    };

    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        let recipe: KpackRecipe = serde_json::from_str(&line).map_err(|error| {
            invalid_data(format!(
                "Invalid registry record on line {line_number}: {error}"
            ))
        })?;
        let id = recipe_id(&recipe);
        let entry = Entry {
            line_number,
            line,
            id: id.clone(),
            recipe,
        };

        if wanted_ids.contains(&id) {
            scan.by_id.insert(id, entry.clone());
        }
        if wanted_lines.contains(&line_number) {
            scan.by_line.insert(line_number, entry.clone());
        }
        scan.line_count = line_number;
        scan.latest = Some(entry);
    }

    Ok(scan)
}

fn resolve_tags(registry_path: &Path, tags_path: &Path) -> io::Result<ResolvedTags> {
    let stored = read_stored_tags(tags_path)?;
    let (wanted_ids, wanted_lines) = match &stored {
        StoredTags::Stable(tags) => (
            tags.values().cloned().collect::<BTreeSet<_>>(),
            BTreeSet::new(),
        ),
        StoredTags::Legacy(tags) => (
            BTreeSet::new(),
            tags.values().copied().collect::<BTreeSet<_>>(),
        ),
    };
    let scan = scan_registry(registry_path, &wanted_ids, &wanted_lines)?;
    let mut tags = BTreeMap::new();
    let mut entries = BTreeMap::new();

    match stored {
        StoredTags::Stable(stored_tags) => {
            for (name, id) in stored_tags {
                let entry = scan.by_id.get(&id).ok_or_else(|| {
                    invalid_data(format!("Tag '{name}' references missing snapshot {id}"))
                })?;
                tags.insert(name, id.clone());
                entries.insert(id, entry.clone());
            }
        }
        StoredTags::Legacy(stored_tags) => {
            for (name, line_number) in stored_tags {
                let entry = scan.by_line.get(&line_number).ok_or_else(|| {
                    invalid_data(format!(
                        "Legacy tag '{name}' references missing registry line {line_number}"
                    ))
                })?;
                tags.insert(name, entry.id.clone());
                entries
                    .entry(entry.id.clone())
                    .and_modify(|saved: &mut Entry| {
                        if entry.line_number > saved.line_number {
                            *saved = entry.clone();
                        }
                    })
                    .or_insert_with(|| entry.clone());
            }
        }
    }

    Ok(ResolvedTags {
        tags,
        entries,
        scan,
    })
}

fn write_temp(path: &Path, content: &[u8]) -> io::Result<PathBuf> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Destination has no file name"))?
        .to_string_lossy();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let temporary_path = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)?;
    if let Err(error) = file.write_all(content).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    Ok(temporary_path)
}

fn sync_parent(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    File::open(parent)?.sync_all()
}

fn replace_with_temp(temporary_path: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(temporary_path, destination)?;
    sync_parent(destination)
}

fn write_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    let temporary_path = write_temp(path, content)?;
    if let Err(error) = replace_with_temp(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    Ok(())
}

fn write_tags(path: &Path, tags: &BTreeMap<String, String>) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(tags).map_err(io::Error::other)?;
    write_atomic(path, &bytes)
}

fn verify_recipe(entry: &Entry) -> io::Result<()> {
    entry.recipe.decode().map(|_| ()).map_err(|error| {
        invalid_data(format!(
            "Snapshot {} failed verification: {error}",
            entry.id
        ))
    })
}

pub fn read_tags(registry_path: &Path, tags_path: &Path) -> io::Result<BTreeMap<String, String>> {
    Ok(resolve_tags(registry_path, tags_path)?.tags)
}

pub fn create_tag(registry_path: &Path, tags_path: &Path, name: &str) -> io::Result<String> {
    if name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Tag name cannot be empty",
        ));
    }
    let mut resolved = resolve_tags(registry_path, tags_path)?;
    let latest = resolved
        .scan
        .latest
        .as_ref()
        .ok_or_else(|| invalid_data("Registry has no snapshots"))?;
    verify_recipe(latest)?;
    let id = latest.id.clone();
    resolved.tags.insert(name.trim().to_owned(), id.clone());
    write_tags(tags_path, &resolved.tags)?;
    Ok(id)
}

pub fn delete_tag(registry_path: &Path, tags_path: &Path, name: &str) -> io::Result<bool> {
    let mut resolved = resolve_tags(registry_path, tags_path)?;
    if resolved.tags.remove(name).is_none() {
        return Ok(false);
    }
    write_tags(tags_path, &resolved.tags)?;
    Ok(true)
}

pub fn export_tag(
    registry_path: &Path,
    tags_path: &Path,
    name: &str,
    destination: &Path,
) -> io::Result<usize> {
    if destination == registry_path || destination == tags_path {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Export destination must differ from the registry and tags files",
        ));
    }
    let resolved = resolve_tags(registry_path, tags_path)?;
    let id = resolved.tags.get(name).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Tag '{name}' not found"))
    })?;
    let target = resolved
        .entries
        .get(id)
        .ok_or_else(|| invalid_data(format!("Tag '{name}' has no matching snapshot")))?;
    verify_recipe(target)?;

    let temporary_path = write_temp(destination, b"")?;
    let result = (|| {
        let source = File::open(registry_path)?;
        let mut reader = BufReader::new(source);
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&temporary_path)?,
        );
        let mut line = String::new();

        for _ in 0..target.line_number {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                return Err(invalid_data("Registry ended before the tagged snapshot"));
            }
            writer.write_all(line.as_bytes())?;
        }

        writer.flush()?;
        writer.get_ref().sync_all()?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    if let Err(error) = replace_with_temp(&temporary_path, destination) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    Ok(target.line_number)
}

pub fn compact_registry(registry_path: &Path, tags_path: &Path) -> io::Result<CompactionReport> {
    if registry_path == tags_path {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Registry and tags paths must differ",
        ));
    }
    let mut resolved = resolve_tags(registry_path, tags_path)?;
    let latest = resolved
        .scan
        .latest
        .take()
        .ok_or_else(|| invalid_data("Registry has no snapshots"))?;
    verify_recipe(&latest)?;

    resolved
        .entries
        .entry(latest.id.clone())
        .and_modify(|entry| {
            if latest.line_number > entry.line_number {
                *entry = latest.clone();
            }
        })
        .or_insert(latest);

    let mut retained: Vec<Entry> = resolved.entries.into_values().collect();
    retained.sort_by_key(|entry| entry.line_number);
    for entry in &retained {
        verify_recipe(entry)?;
    }

    let mut expected_ids = Vec::with_capacity(retained.len());
    let temporary_path = write_temp(registry_path, b"")?;
    let result = (|| {
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&temporary_path)?,
        );
        for entry in &retained {
            writeln!(writer, "{}", entry.line)?;
            expected_ids.push(entry.id.clone());
        }
        writer.flush()?;
        writer.get_ref().sync_all()?;
        verify_registry(&temporary_path, &expected_ids)
    })();

    if let Err(error) = result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }

    let bytes_before = fs::metadata(registry_path)?.len();
    let tags_kept = resolved.tags.len();
    write_tags(tags_path, &resolved.tags)?;
    if let Err(error) = replace_with_temp(&temporary_path, registry_path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }
    let bytes_after = fs::metadata(registry_path)?.len();

    Ok(CompactionReport {
        records_before: resolved.scan.line_count,
        records_after: retained.len(),
        bytes_before,
        bytes_after,
        tags_kept,
    })
}

fn verify_registry(path: &Path, expected_ids: &[String]) -> io::Result<()> {
    let file = File::open(path)?;
    let mut count = 0usize;
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = index + 1;
        let line = line?;
        let recipe: KpackRecipe = serde_json::from_str(&line).map_err(|error| {
            invalid_data(format!(
                "Invalid compacted record on line {line_number}: {error}"
            ))
        })?;
        let id = recipe_id(&recipe);
        if expected_ids.get(index) != Some(&id) {
            return Err(invalid_data(format!(
                "Compacted record {line_number} does not match the retained snapshot"
            )));
        }
        let entry = Entry {
            line_number,
            line,
            id,
            recipe,
        };
        verify_recipe(&entry)?;
        count += 1;
    }
    if count != expected_ids.len() {
        return Err(invalid_data("Compacted registry record count mismatch"));
    }
    Ok(())
}
