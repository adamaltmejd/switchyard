//! The project's agent environment: the files a worker's harness loads at
//! its user level, read from a base commit in canonical's objects on the
//! host, never from a worker's tree.

use crate::git::Git;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// Every path staged for any harness. Anything else in the base stays out.
const PATHS: [&str; 10] = [
    "AGENTS.md",
    "CLAUDE.md",
    ".agents/skills",
    ".claude/skills",
    ".pi/skills",
    ".pi/extensions",
    ".claude/settings.json",
    ".codex/config.toml",
    ".codex/skills",
    ".codex/hooks.json",
];
/// The staged paths that are single files; the rest are directories.
const FILES: [&str; 5] = [
    "AGENTS.md",
    "CLAUDE.md",
    ".claude/settings.json",
    ".codex/config.toml",
    ".codex/hooks.json",
];
const TOPS: [&str; 4] = [".agents", ".claude", ".pi", ".codex"];
const GUIDANCE: [&str; 2] = ["AGENTS.md", "CLAUDE.md"];
const GUIDANCE_MAX: u64 = 64 * 1024;
const FILE_MAX: u64 = 1024 * 1024;
const TOTAL_MAX: u64 = 8 * 1024 * 1024;
const COUNT_MAX: usize = 512;
/// The largest value one argv entry can carry, matching the prompt bound.
const ARG_MAX_BYTES: usize = 128 * 1024 - 1;
/// Where Codex looks for workspace skills, and how deep it looks.
const WORKSPACE_SKILL_ROOTS: [&str; 2] = [".agents/skills", ".codex/skills"];
const WALK_DEPTH: usize = 32;

#[derive(Debug)]
pub struct File {
    pub path: String,
    pub bytes: Vec<u8>,
    pub exec: bool,
}

#[derive(Debug)]
pub struct AgentEnv {
    files: Vec<File>,
    /// `SKILL.md` paths in the workspace as the worker sees them, so Codex
    /// can be told not to load them.
    pub workspace_skills: Vec<String>,
}

impl AgentEnv {
    /// The named paths of `base`, verbatim. A link or submodule at any path
    /// component, or a file over a bound, refuses the whole read.
    pub async fn load(
        git: &Git,
        canonical: &Path,
        base: &str,
        workspace: &Path,
    ) -> Result<AgentEnv, String> {
        let mut files = Vec::new();
        let mut total = 0;
        // A link at any component of a staged path is refused: the top
        // directories here, everything below them in the listing.
        for entry in git.ls_tree(canonical, base, &TOPS, false).await? {
            if !entry.dir {
                return Err(format!(
                    "{} is {} in the base, not a directory",
                    entry.path,
                    if entry.regular {
                        "a file"
                    } else {
                        "a link or submodule"
                    }
                ));
            }
        }
        for entry in git.ls_tree(canonical, base, &PATHS, true).await? {
            if !entry.regular {
                return Err(format!("{} is a link or submodule in the base", entry.path));
            }
            let wrong_type = PATHS.iter().any(|path| {
                if FILES.contains(path) {
                    entry.path.starts_with(&format!("{path}/"))
                } else {
                    entry.path == *path
                }
            });
            if wrong_type {
                return Err(format!(
                    "{} is the wrong type in the base (a file where a directory is \
                     expected, or a tree where a file is)",
                    entry.path
                ));
            }
            let max = if GUIDANCE.contains(&entry.path.as_str()) {
                GUIDANCE_MAX
            } else {
                FILE_MAX
            };
            total += entry.size;
            if entry.size > max || total > TOTAL_MAX || files.len() == COUNT_MAX {
                return Err(format!(
                    "{} is over the agent environment bounds ({max} bytes a file, \
                     {TOTAL_MAX} in all, {COUNT_MAX} files)",
                    entry.path
                ));
            }
            if entry
                .path
                .split('/')
                .any(|part| part == ".." || part == ".")
            {
                return Err(format!("{} is not a plain path", entry.path));
            }
            files.push(File {
                bytes: git.blob(canonical, &entry.oid).await?,
                path: entry.path.clone(),
                exec: entry.exec,
            });
        }
        let guidance: Vec<_> = files
            .iter()
            .filter(|file| GUIDANCE.contains(&file.path.as_str()))
            .collect();
        if let Some(file) = guidance
            .iter()
            .find(|file| std::str::from_utf8(&file.bytes).is_err())
        {
            return Err(format!("{} is not UTF-8", file.path));
        }
        let joined: usize = guidance.iter().map(|file| file.bytes.len() + 2).sum();
        if joined > ARG_MAX_BYTES {
            return Err(format!(
                "AGENTS.md and CLAUDE.md together are over the {ARG_MAX_BYTES} byte prompt bound"
            ));
        }
        if let Some(file) = files
            .iter_mut()
            .find(|file| file.path == ".codex/config.toml")
        {
            file.bytes = without_mcp_servers(&file.bytes)?;
        }
        Ok(AgentEnv {
            files,
            workspace_skills: workspace_skills(workspace)?,
        })
    }

    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|file| file.path == path)
            .map(|file| file.bytes.as_slice())
    }

    /// The files under `prefix` (a directory, ending in `/`), by the path
    /// below it.
    pub fn under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = (&'a str, &'a File)> {
        self.files
            .iter()
            .filter_map(move |file| Some((file.path.strip_prefix(prefix)?, file)))
    }

    /// Whether the base has any file under `prefix`.
    pub fn has(&self, prefix: &str) -> bool {
        self.under(prefix).next().is_some()
    }

    /// The root guidance files, as system-prompt text for a harness that does
    /// not load them from its user level.
    pub fn guidance(&self) -> Option<String> {
        let parts: Vec<_> = GUIDANCE
            .iter()
            .filter_map(|name| self.get(name))
            .map(String::from_utf8_lossy)
            .collect();
        (!parts.is_empty()).then(|| parts.join("\n\n"))
    }

    /// Replace the file `rel` under `state` with the guidance, or remove it
    /// when the base has none.
    pub fn stage_guidance(&self, state: &Path, rel: &str) -> std::io::Result<()> {
        let dest = prepare(state, rel)?;
        match self.guidance() {
            Some(text) => write(
                &dest,
                &File {
                    path: String::new(),
                    bytes: text.into_bytes(),
                    exec: false,
                },
            ),
            None => Ok(()),
        }
    }

    /// Replace `rel` under `state` with the files under `prefix`.
    pub fn stage_dir(&self, state: &Path, rel: &str, prefix: &str) -> std::io::Result<()> {
        let dest = prepare(state, rel)?;
        for (rest, file) in self.under(prefix) {
            write(&dest.join(rest), file)?;
        }
        Ok(())
    }

    /// Replace the file `rel` under `state` with `path`'s bytes, or remove it
    /// when the base has none.
    pub fn stage_file(&self, state: &Path, rel: &str, path: &str) -> std::io::Result<()> {
        let dest = prepare(state, rel)?;
        match self.files.iter().find(|file| file.path == path) {
            Some(file) => write(&dest, file),
            None => Ok(()),
        }
    }
}

/// Clear `rel` under `state` and return its path, with every parent a real
/// directory. The box mounts `state` writable, so a previous execution may
/// have left a link anywhere in it; nothing is written through one.
fn prepare(state: &Path, rel: &str) -> std::io::Result<PathBuf> {
    // The state dir itself is Yard's, made here on a first execution.
    std::fs::create_dir_all(state)?;
    let mut path = state.to_path_buf();
    let parts: Vec<_> = rel.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        path.push(part);
        if i + 1 == parts.len() {
            break;
        }
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                std::fs::remove_file(&path)?;
                std::fs::create_dir(&path)?;
            }
            Err(_) => std::fs::create_dir(&path)?,
        }
    }
    match std::fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(&path)?,
        Ok(_) => std::fs::remove_file(&path)?,
        Err(_) => {}
    }
    Ok(path)
}

fn write(dest: &Path, file: &File) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(if file.exec { 0o755 } else { 0o644 })
        .open(dest)?
        .write_all(&file.bytes)
}

/// Every `SKILL.md` Codex would discover in the workspace, by the path the
/// worker sees. Nothing is followed. A tree beyond the bounds refuses the
/// execution, since a skill left unlisted would load.
fn workspace_skills(workspace: &Path) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    for root in WORKSPACE_SKILL_ROOTS {
        walk(workspace, root, 0, &mut found)?;
    }
    Ok(found)
}

fn walk(workspace: &Path, rel: &str, depth: usize, found: &mut Vec<String>) -> Result<(), String> {
    let dir = workspace.join(rel);
    if depth == 0 {
        // Every component: a link above the root is followed too.
        let mut at = PathBuf::new();
        for part in rel.split('/') {
            at.push(part);
            match std::fs::symlink_metadata(workspace.join(&at)) {
                Ok(meta) if meta.is_symlink() => {
                    return Err(format!("{} is a link; Codex would follow it", at.display()));
                }
                Ok(meta) if meta.is_dir() => {}
                _ => return Ok(()),
            }
        }
    }
    if depth > WALK_DEPTH {
        return Err(format!("{rel} is deeper than {WALK_DEPTH} directories"));
    }
    let entries = std::fs::read_dir(dir).map_err(|error| format!("{rel}: {error}"))?;
    for entry in entries.flatten() {
        let (Ok(name), Ok(kind)) = (entry.file_name().into_string(), entry.file_type()) else {
            continue;
        };
        let child = format!("{rel}/{name}");
        if kind.is_symlink() {
            return Err(format!("{child} is a link; Codex would follow it"));
        }
        if kind.is_dir() {
            walk(workspace, &child, depth + 1, found)?;
        } else if kind.is_file() && name == "SKILL.md" {
            if found.len() == COUNT_MAX {
                return Err(format!("{child} is past {COUNT_MAX} workspace skills"));
            }
            found.push(format!("/workspace/{child}"));
        }
    }
    Ok(())
}

/// The Codex config without its `mcp_servers` table: an override of the table
/// merges into it, so Yard's server stays the only one by not staging any.
/// The one file not copied verbatim.
fn without_mcp_servers(config: &[u8]) -> Result<Vec<u8>, String> {
    let mut table: toml::Table = std::str::from_utf8(config)
        .map_err(|_| ".codex/config.toml is not UTF-8".to_string())?
        .parse()
        .map_err(|error| format!(".codex/config.toml is not valid TOML: {error}"))?;
    table.remove("mcp_servers");
    Ok(table.to_string().into_bytes())
}
