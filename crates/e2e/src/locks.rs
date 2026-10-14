//! Host-only daemon-lock evidence for Y-147. Never read argv or environment.

#[cfg(target_os = "linux")]
pub fn report(state: &std::path::Path, phase: &str, daemon_pid: u32) {
    use serde_json::json;
    use std::os::unix::fs::MetadataExt;

    let path = state.join("yard/daemon.lock");
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) => {
            eprintln!(
                "Y-147 lock evidence: {}",
                json!({"phase": phase, "daemon_pid": daemon_pid,
                    "metadata_errno": error.raw_os_error(), "incomplete": true})
            );
            return;
        }
    };
    let dev = metadata.dev();
    let inode = metadata.ino();
    // Linux's dev_t encoding, as used by /proc/locks.
    let major = ((dev >> 8) & 0xfff) | ((dev >> 32) & 0xffff_f000);
    let minor = (dev & 0xff) | ((dev >> 12) & 0xffff_ff00);
    let mut incomplete = false;
    let kernel_locks = read(std::path::Path::new("/proc/locks")).map(|text| {
        text.lines()
            .filter(|line| {
                line.split_whitespace().any(|word| {
                    let parts: Vec<_> = word.split(':').collect();
                    parts.len() == 3
                        && u64::from_str_radix(parts[0], 16) == Ok(major)
                        && u64::from_str_radix(parts[1], 16) == Ok(minor)
                        && parts[2].parse::<u64>() == Ok(inode)
                })
            })
            .take(128)
            .map(|line| {
                let words: Vec<_> = line.split_whitespace().collect();
                let owner = words
                    .iter()
                    .position(|word| word.contains(':') && word.matches(':').count() == 2)
                    .and_then(|index| index.checked_sub(1))
                    .and_then(|index| words[index].parse::<u32>().ok());
                let owner_identity = owner
                    .and_then(|pid| identity(&std::path::PathBuf::from(format!("/proc/{pid}"))));
                json!({"row": line, "owner_pid": owner, "owner_identity": owner_identity})
            })
            .collect::<Vec<_>>()
    });
    if kernel_locks.is_none() {
        incomplete = true;
    }
    let mut descriptors = Vec::new();
    let mut processes_seen = 0;
    let mut descriptors_seen = 0;
    let mut inaccessible_processes = 0;
    let mut vanished_descriptors = 0;
    let mut daemon_identity = None;
    if let Ok(processes) = std::fs::read_dir("/proc") {
        'processes: for entry in processes {
            let Ok(entry) = entry else {
                incomplete = true;
                continue;
            };
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
                continue;
            };
            processes_seen += 1;
            if processes_seen > 32768 {
                incomplete = true;
                break;
            }
            let process = entry.path();
            let birth = identity(&process);
            if pid == daemon_pid {
                daemon_identity = birth.clone();
            }
            let Ok(fds) = std::fs::read_dir(process.join("fd")) else {
                inaccessible_processes += 1;
                continue;
            };
            for fd in fds {
                descriptors_seen += 1;
                if descriptors_seen > 131072 || descriptors.len() == 128 {
                    incomplete = true;
                    break 'processes;
                }
                let Ok(fd) = fd else {
                    vanished_descriptors += 1;
                    continue;
                };
                let Ok(metadata) = std::fs::metadata(fd.path()) else {
                    vanished_descriptors += 1;
                    continue;
                };
                if metadata.dev() != dev || metadata.ino() != inode {
                    continue;
                }
                let fdinfo = read(&process.join("fdinfo").join(fd.file_name()));
                let locks = fdinfo.as_ref().map(|text| {
                    text.lines()
                        .filter(|line| line.starts_with("lock:"))
                        .take(128)
                        .collect::<Vec<_>>()
                });
                let flags = fdinfo
                    .as_ref()
                    .and_then(|text| text.lines().find(|line| line.starts_with("flags:")));
                // A changed start identity makes this a raced observation.
                let after = identity(&process);
                let still_matches = std::fs::metadata(fd.path())
                    .is_ok_and(|metadata| metadata.dev() == dev && metadata.ino() == inode);
                descriptors.push(json!({"pid": pid, "identity": birth,
                    "identity_after": after, "fd": fd.file_name().to_string_lossy(),
                    "still_matches": still_matches, "flags": flags, "locks": locks}));
            }
        }
    } else {
        incomplete = true;
    }
    eprintln!(
        "Y-147 lock evidence: {}",
        json!({"phase": phase, "daemon_pid": daemon_pid,
            "daemon_identity": daemon_identity, "dev": dev, "inode": inode,
            "kernel_locks": kernel_locks, "matching_descriptors": descriptors,
            "incomplete": incomplete, "inaccessible_processes": inaccessible_processes,
            "vanished_descriptors": vanished_descriptors,
            "processes_seen": processes_seen, "descriptors_seen": descriptors_seen})
    );
}

#[cfg(target_os = "linux")]
fn read(path: &std::path::Path) -> Option<String> {
    use std::io::Read;
    let file = std::fs::File::open(path).ok()?;
    let mut text = String::new();
    file.take(262145).read_to_string(&mut text).ok()?;
    (text.len() <= 262144).then_some(text)
}

#[cfg(target_os = "linux")]
fn identity(process: &std::path::Path) -> Option<serde_json::Value> {
    let stat = read(&process.join("stat"))?;
    // comm may contain spaces or ')'. Do not record it.
    let (_, fields) = stat.rsplit_once(')')?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    Some(serde_json::json!({
        "ppid": fields.get(1)?.parse::<u32>().ok()?,
        "start_ticks": fields.get(19)?.parse::<u64>().ok()?
    }))
}

#[cfg(not(target_os = "linux"))]
pub fn report(_state: &std::path::Path, _phase: &str, _daemon_pid: u32) {}
