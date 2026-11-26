use std::env;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const HISTORY_LIMIT: usize = 1_000;

pub fn is_safe_command(input: &str) -> bool {
    matches!(
        input.to_ascii_lowercase().as_str(),
        "status"
            | "hypotheses"
            | "questions"
            | "inquire"
            | "tags"
            | "oracles"
            | "reflections 24h"
            | "pack"
            | "unpack"
            | "compact"
            | "nap"
            | "sieste"
            | "rest"
            | "repos"
            | "hibernate"
            | "dream"
            | "teach"
            | "clear"
            | "cls"
            | "quit"
            | "exit"
    )
}

pub fn state_dir() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("XDG_STATE_HOME").filter(|p| !p.is_empty()) {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "XDG_STATE_HOME must be absolute",
            ));
        }
        return Ok(path.join("klm"));
    }
    let home = env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home).join(".local/state/klm"))
}

fn prepare_dir(dir: &Path) -> io::Result<()> {
    DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    let meta = fs::symlink_metadata(dir)?;
    if !meta.is_dir() || meta.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("History directory must be private: {}", dir.display()),
        ));
    }
    Ok(())
}

fn read_history(path: &Path) -> io::Result<Vec<String>> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    if !meta.is_file() || meta.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "History file must be a private regular file",
        ));
    }
    let entries: Vec<String> = serde_json::from_slice(&fs::read(path)?)?;
    if entries.len() > HISTORY_LIMIT || entries.iter().any(|entry| !is_safe_command(entry)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "History contains invalid or unsupported entries",
        ));
    }
    Ok(entries)
}

pub fn write_private_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "File has no parent directory")
    })?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "File has no name"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();
    let temp = parent.join(format!(
        ".{}.{}.{}.tmp",
        name.to_string_lossy(),
        std::process::id(),
        nonce
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp)?;
    let result = (|| {
        file.write_all(contents)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        File::open(parent)?.sync_all()
    })();
    if result.is_err() && temp.exists() {
        fs::remove_file(&temp)?;
    }
    result
}

pub struct CommandHistory {
    dir: PathBuf,
}

impl CommandHistory {
    pub fn new(dir: PathBuf) -> io::Result<Self> {
        prepare_dir(&dir)?;
        Ok(Self { dir })
    }

    fn lock(&self) -> io::Result<File> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(self.dir.join("history.lock"))?;
        file.lock()?;
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn history_is_private_bounded_and_excludes_conversation() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = env::temp_dir().join(format!(
            "klm-history-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let history = CommandHistory::new(dir.clone()).unwrap();
        assert!(history.record("status").unwrap());
        assert!(!history.record("my private conversation").unwrap());
        assert!(!history.record("hypothesis sensitive text").unwrap());
        assert!(!history.record("tag private-name").unwrap());
        let entries: Vec<&str> = (0..HISTORY_LIMIT)
            .map(|i| if i % 2 == 0 { "hypotheses" } else { "status" })
            .collect();
        write_private_atomic(&dir.join("history"), &serde_json::to_vec(&entries).unwrap()).unwrap();
        history.record("questions").unwrap();
        let entries = history.load().unwrap();
        assert_eq!(entries.len(), HISTORY_LIMIT);
        assert_eq!(entries.last().unwrap(), "questions");
        assert_eq!(
            fs::metadata(dir.join("history"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(fs::metadata(&dir).unwrap().permissions().mode() & 0o077, 0);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn refuses_insecure_existing_history() {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = env::temp_dir().join(format!(
            "klm-history-permissions-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let history = CommandHistory::new(dir.clone()).unwrap();
        fs::write(dir.join("history"), "[]").unwrap();
        fs::set_permissions(dir.join("history"), fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            history.load().unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        fs::remove_dir_all(dir).unwrap();
    }
}

impl CommandHistory {
    pub fn load(&self) -> io::Result<Vec<String>> {
        let _lock = self.lock()?;
        read_history(&self.dir.join("history"))
    }

    pub fn record(&self, input: &str) -> io::Result<bool> {
        if !is_safe_command(input) {
            return Ok(false);
        }
        let _lock = self.lock()?;
        let path = self.dir.join("history");
        let mut entries = read_history(&path)?;
        if entries.last().is_none_or(|last| last != input) {
            entries.push(input.to_owned());
            if entries.len() > HISTORY_LIMIT {
                entries.remove(0);
            }
            write_private_atomic(&path, &serde_json::to_vec(&entries)?)?;
        }
        Ok(true)
    }
}
