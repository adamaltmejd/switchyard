//! The worker-written proof directory: a content-addressed snapshot that is
//! part of the candidate identity. Yard copies `live` into `root/<digest>`
//! without ever following a link, and refuses any entry that is not a regular
//! file or a directory, by name.

use crate::config::hex;
use crate::daemon::Project;
use sha2::{Digest, Sha256};
use std::os::unix::ffi::OsStrExt;
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
    relative: PathBuf,
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

/// The digest of an empty proof directory. A candidate that has never had a
/// proof is compared against it, so an empty proof is not a change.
pub fn empty_digest() -> String {
    hash(&[]).expect("an empty proof hashes")
}

/// Copy `live` into `root/<digest>` and return the digest. Idempotent when
/// the copy already exists. A copy is staged and renamed into place, so a
/// crash never leaves a partial `root/<digest>` that a later run trusts.
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
    let staging = root.join(format!(".staging-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    if let Err(error) = copy(&entries, &staging) {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(error);
    }
    std::fs::rename(&staging, &destination).map_err(|error| {
        let _ = std::fs::remove_dir_all(&staging);
        error.to_string()
    })?;
    Ok(digest)
}

fn copy(entries: &[Entry], staging: &Path) -> Result<(), String> {
    for entry in entries.iter().filter(|entry| entry.kind == Kind::File) {
        let target = staging.join(&entry.relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::copy(&entry.source, &target).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn collect(
    live: &Path,
    dir: &Path,
    entries: &mut Vec<Entry>,
    bytes: &mut u64,
) -> Result<(), String> {
    let reader =
        std::fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))?;
    // Bound the walk while reading: a directory with more entries than the
    // whole budget is refused without materialising all of them. The entry
    // that crosses the bound is named relative to `live`.
    let mut children: Vec<PathBuf> = Vec::new();
    for entry in reader {
        let path = entry.map_err(|error| error.to_string())?.path();
        if entries.len() + children.len() >= PROOF_MAX_FILES {
            let relative = path.strip_prefix(live).unwrap_or(&path);
            return Err(format!(
                "proof has more than {PROOF_MAX_FILES} entries at {:?}",
                relative
            ));
        }
        children.push(path);
    }
    children.sort();
    for path in children {
        let relative = path.strip_prefix(live).unwrap_or(&path).to_path_buf();
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            entries.push(Entry {
                relative: relative.clone(),
                kind: Kind::Dir,
                length: 0,
                source: path.clone(),
            });
            collect(live, &path, entries, bytes)?;
        } else if file_type.is_file() {
            *bytes = bytes.saturating_add(metadata.len());
            if *bytes > PROOF_MAX_BYTES {
                return Err(format!(
                    "proof is larger than {PROOF_MAX_BYTES} bytes at {:?}",
                    relative
                ));
            }
            entries.push(Entry {
                relative,
                kind: Kind::File,
                length: metadata.len(),
                source: path,
            });
        } else {
            return Err(format!(
                "proof entry {:?} is a {}",
                relative,
                type_name(&file_type)
            ));
        }
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
        // The raw path bytes, so two names that differ only in invalid UTF-8
        // are two identities and a snapshot path cannot collide.
        let path = entry.relative.as_os_str().as_bytes();
        hasher.update((path.len() as u64).to_be_bytes());
        hasher.update(path);
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
