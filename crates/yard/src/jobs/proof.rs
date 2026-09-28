//! The worker-written proof directory: a content-addressed snapshot that is
//! part of the candidate identity. Yard copies `live` into `root/<digest>`
//! without ever following a link, and refuses any entry that is not a regular
//! file or a directory, by name.

use crate::config::hex;
use crate::daemon::Project;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const PROOF_MAX_BYTES: u64 = 64 * 1024 * 1024;
pub const PROOF_MAX_FILES: usize = 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dir,
    File,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Dir => "dir",
            Kind::File => "file",
        }
    }
}

struct Entry {
    relative: String,
    kind: Kind,
    length: u64,
    source: PathBuf,
}

/// The snapshot directory for one digest.
pub fn snapshot_path(project: &Project, attempt: i64, digest: &str) -> PathBuf {
    project
        .attempt_dir(attempt)
        .join("proof-snapshots")
        .join(digest)
}

/// Copy `live` into `root/<digest>` and return the digest. Idempotent when
/// the copy already exists.
pub fn snapshot(live: &Path, root: &Path) -> Result<String, String> {
    let mut entries = Vec::new();
    let mut bytes = 0u64;
    collect(live, live, &mut entries, &mut bytes)?;
    entries.sort_by(|a, b| a.relative.cmp(&b.relative));
    let digest = hash(&entries)?;
    let destination = root.join(&digest);
    if destination.exists() {
        return Ok(digest);
    }
    std::fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
    for entry in entries.iter().filter(|entry| entry.kind == Kind::File) {
        let target = destination.join(&entry.relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::copy(&entry.source, &target).map_err(|error| error.to_string())?;
    }
    Ok(digest)
}

fn collect(
    live: &Path,
    dir: &Path,
    entries: &mut Vec<Entry>,
    bytes: &mut u64,
) -> Result<(), String> {
    let mut children: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|error| format!("read {}: {error}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()
        .map_err(|error| error.to_string())?;
    children.sort();
    for path in children {
        let relative = path
            .strip_prefix(live)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            entries.push(Entry {
                relative: relative.clone(),
                kind: Kind::Dir,
                length: 0,
                source: path.clone(),
            });
            check_count(entries, &relative)?;
            collect(live, &path, entries, bytes)?;
        } else if file_type.is_file() {
            *bytes = bytes.saturating_add(metadata.len());
            if *bytes > PROOF_MAX_BYTES {
                return Err(format!(
                    "proof is larger than {PROOF_MAX_BYTES} bytes at {relative:?}"
                ));
            }
            entries.push(Entry {
                relative: relative.clone(),
                kind: Kind::File,
                length: metadata.len(),
                source: path,
            });
            check_count(entries, &relative)?;
        } else {
            return Err(format!(
                "proof entry {relative:?} is a {}",
                type_name(&file_type)
            ));
        }
    }
    Ok(())
}

fn check_count(entries: &[Entry], relative: &str) -> Result<(), String> {
    if entries.len() > PROOF_MAX_FILES {
        return Err(format!(
            "proof has more than {PROOF_MAX_FILES} entries at {relative:?}"
        ));
    }
    Ok(())
}

fn type_name(file_type: &std::fs::FileType) -> &'static str {
    use std::os::unix::fs::FileTypeExt;
    if file_type.is_symlink() {
        "symbolic link"
    } else if file_type.is_fifo() {
        "fifo"
    } else if file_type.is_socket() {
        "socket"
    } else if file_type.is_block_device() {
        "block device"
    } else if file_type.is_char_device() {
        "character device"
    } else {
        "special file"
    }
}

fn hash(entries: &[Entry]) -> Result<String, String> {
    use std::io::Read;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    for entry in entries {
        hasher.update((entry.relative.len() as u64).to_be_bytes());
        hasher.update(entry.relative.as_bytes());
        hasher.update(entry.kind.name().as_bytes());
        if entry.kind == Kind::File {
            hasher.update(entry.length.to_be_bytes());
            let mut file = std::fs::File::open(&entry.source).map_err(|error| error.to_string())?;
            loop {
                let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
        }
    }
    Ok(hex(&hasher.finalize()))
}
