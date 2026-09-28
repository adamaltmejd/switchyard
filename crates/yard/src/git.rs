//! Git as a bounded child under a scrubbed environment. Returns data and
//! never touches the store.

use std::os::fd::RawFd;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Overrides no repository configuration can undo.
const HARDENING: &[&str] = &[
    "core.hooksPath=/dev/null",
    "core.fsmonitor=false",
    "commit.gpgsign=false",
    "tag.gpgsign=false",
    "protocol.allow=never",
    "protocol.file.allow=always",
    "protocol.ext.allow=never",
    "transfer.fsckObjects=true",
    "submodule.recurse=false",
    "gc.auto=0",
];

const TIMEOUT: Duration = Duration::from_secs(300);
const OUTPUT_MAX: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Out {
    fn ok(self, what: &str) -> Result<Out, String> {
        if self.code == 0 {
            Ok(self)
        } else {
            Err(format!(
                "git {what} exited {}: {}",
                self.code,
                self.stderr.trim()
            ))
        }
    }
}

#[derive(Clone)]
pub struct Git {
    path: String,
}

pub enum Merge {
    Clean { tree: String },
    Conflict { paths: Vec<String> },
}

impl Git {
    /// `path` is the daemon's `PATH`, where git is found.
    pub fn new(path: String) -> Git {
        Git { path }
    }

    pub async fn run(&self, cwd: &Path, args: &[&str]) -> Result<Out, String> {
        self.run_with(cwd, args, None).await
    }

    /// `lock` makes a landing child: it inherits the landing's lock
    /// descriptor, so its life is visible to a restarted daemon through
    /// `flock`, and it is never killed; a timeout leaves it running.
    pub async fn run_with(
        &self,
        cwd: &Path,
        args: &[&str],
        lock: Option<RawFd>,
    ) -> Result<Out, String> {
        let mut command = Command::new("git");
        command
            .current_dir(cwd)
            .env_clear()
            .env("PATH", &self.path)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .env("GIT_AUTHOR_NAME", "Switchyard")
            .env("GIT_AUTHOR_EMAIL", "yard@localhost")
            .env("GIT_COMMITTER_NAME", "Switchyard")
            .env("GIT_COMMITTER_EMAIL", "yard@localhost")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(lock.is_none());
        for pair in HARDENING {
            command.arg("-c").arg(pair);
        }
        command.args(args);
        if let Some(fd) = lock {
            inherit(&mut command, fd);
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("spawn git: {error}"))?;
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let run = async {
            let (stdout, stderr) = tokio::join!(
                crate::r#box::read_capped(stdout, OUTPUT_MAX),
                crate::r#box::read_capped(stderr, OUTPUT_MAX)
            );
            let status = child.wait().await.map_err(|error| error.to_string())?;
            Ok::<_, String>(Out {
                code: exit_code(status),
                stdout,
                stderr,
            })
        };
        match tokio::time::timeout(TIMEOUT, run).await {
            Ok(result) => result,
            // Dropping the future drops the child, which kills it unless it
            // is a landing child.
            Err(_) => Err(format!("git {} timed out", args.first().unwrap_or(&""))),
        }
    }

    /// A bare repository whose HEAD names `branch`: the target's name.
    pub async fn init_bare(&self, dir: &Path, branch: &str) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
        let initial = format!("--initial-branch={branch}");
        self.run(dir, &["init", "--bare", "--quiet", &initial, "."])
            .await?
            .ok("init")?;
        Ok(())
    }

    /// The branch a bare repository's HEAD names.
    pub async fn head_branch(&self, repo: &Path) -> Result<String, String> {
        self.current_branch(repo)
            .await?
            .ok_or_else(|| "HEAD names no branch".to_string())
    }

    /// The commit `rev` names, or None.
    pub async fn rev_parse(&self, repo: &Path, rev: &str) -> Result<Option<String>, String> {
        let out = self
            .run(
                repo,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    "--end-of-options",
                    &format!("{rev}^{{commit}}"),
                ],
            )
            .await?;
        Ok((out.code == 0).then(|| out.stdout.trim().to_string()))
    }

    /// A file's bytes at a commit, or None when absent.
    pub async fn show(
        &self,
        repo: &Path,
        commit: &str,
        path: &str,
    ) -> Result<Option<String>, String> {
        let out = self
            .run(repo, &["cat-file", "blob", &format!("{commit}:{path}")])
            .await?;
        Ok((out.code == 0).then_some(out.stdout))
    }

    /// Fetch `refspec` from the repository at `from` into `repo`.
    pub async fn fetch(&self, repo: &Path, from: &Path, refspec: &str) -> Result<(), String> {
        let from = from.to_str().ok_or("path is not UTF-8")?;
        self.run(
            repo,
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-write-fetch-head",
                "--end-of-options",
                from,
                refspec,
            ],
        )
        .await?
        .ok("fetch")?;
        Ok(())
    }

    /// A `--no-hardlinks` clone of `canonical` at `start` on a new `branch`,
    /// with no remote left behind.
    pub async fn clone_branch(
        &self,
        canonical: &Path,
        dest: &Path,
        branch: &str,
        start: &str,
    ) -> Result<(), String> {
        self.clone_detached(canonical, dest, start).await?;
        self.run(dest, &["switch", "--quiet", "-c", branch, start])
            .await?
            .ok("switch")?;
        Ok(())
    }

    /// A fresh checkout of `commit` made from canonical's objects: nothing
    /// else of any clone reaches it.
    pub async fn clone_detached(
        &self,
        canonical: &Path,
        dest: &Path,
        commit: &str,
    ) -> Result<(), String> {
        let parent = dest.parent().ok_or("checkout has no parent")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let source = canonical.to_str().ok_or("path is not UTF-8")?;
        let target = dest.to_str().ok_or("path is not UTF-8")?;
        self.run(
            parent,
            &[
                "clone",
                "--quiet",
                "--no-hardlinks",
                "--no-checkout",
                "--end-of-options",
                source,
                target,
            ],
        )
        .await?
        .ok("clone")?;
        // A candidate or merged commit is reachable only from a yard ref,
        // which a clone does not copy; protocol v2 serves it by id.
        self.run(dest, &["fetch", "--quiet", "--no-tags", "origin", commit])
            .await?
            .ok("fetch")?;
        self.run(dest, &["checkout", "--quiet", "--detach", commit])
            .await?
            .ok("checkout")?;
        self.run(dest, &["remote", "remove", "origin"])
            .await?
            .ok("remote remove")?;
        Ok(())
    }

    /// Merge `theirs` onto `ours` without a worktree.
    pub async fn merge_tree(&self, repo: &Path, ours: &str, theirs: &str) -> Result<Merge, String> {
        let out = self
            .run(
                repo,
                &[
                    "merge-tree",
                    "--write-tree",
                    "--name-only",
                    "--no-messages",
                    ours,
                    theirs,
                ],
            )
            .await?;
        let mut lines = out.stdout.lines();
        match out.code {
            0 => Ok(Merge::Clean {
                tree: lines.next().unwrap_or_default().to_string(),
            }),
            1 => Ok(Merge::Conflict {
                paths: lines.skip(1).map(str::to_string).collect(),
            }),
            _ => Err(format!("git merge-tree failed: {}", out.stderr.trim())),
        }
    }

    pub async fn commit_tree(
        &self,
        repo: &Path,
        tree: &str,
        parents: &[&str],
        message: &str,
    ) -> Result<String, String> {
        let mut args = vec!["commit-tree", tree];
        for parent in parents {
            args.push("-p");
            args.push(parent);
        }
        args.push("-m");
        args.push(message);
        let out = self.run(repo, &args).await?.ok("commit-tree")?;
        Ok(out.stdout.trim().to_string())
    }

    /// Compare-and-swap a ref. Ok(false) when `old` no longer matches.
    pub async fn update_ref(
        &self,
        repo: &Path,
        name: &str,
        new: &str,
        old: Option<&str>,
        lock: Option<RawFd>,
    ) -> Result<bool, String> {
        // An absent ref is expected as the zero id.
        let zero = "0".repeat(40);
        let old = old.unwrap_or(&zero);
        let out = self
            .run_with(repo, &["update-ref", "--no-deref", name, new, old], lock)
            .await?;
        Ok(out.code == 0)
    }

    pub async fn delete_ref(&self, repo: &Path, name: &str) -> Result<(), String> {
        self.run(repo, &["update-ref", "-d", name])
            .await?
            .ok("update-ref -d")?;
        Ok(())
    }

    pub async fn is_ancestor(
        &self,
        repo: &Path,
        ancestor: &str,
        of: &str,
        lock: Option<RawFd>,
    ) -> Result<bool, String> {
        let out = self
            .run_with(repo, &["merge-base", "--is-ancestor", ancestor, of], lock)
            .await?;
        match out.code {
            0 => Ok(true),
            1 => Ok(false),
            _ => Err(format!("git merge-base failed: {}", out.stderr.trim())),
        }
    }

    /// Every path `base..head` touches, renames split into both names.
    pub async fn changed_paths(
        &self,
        repo: &Path,
        base: &str,
        head: &str,
    ) -> Result<Vec<String>, String> {
        let out = self
            .run(
                repo,
                &[
                    "diff",
                    "--name-only",
                    "--no-renames",
                    "-z",
                    base,
                    head,
                    "--",
                ],
            )
            .await?
            .ok("diff")?;
        Ok(out
            .stdout
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect())
    }

    pub async fn diff_stat(&self, repo: &Path, base: &str, head: &str) -> Result<String, String> {
        Ok(self
            .run(repo, &["diff", "--stat", base, head, "--"])
            .await?
            .ok("diff")?
            .stdout)
    }

    pub async fn diff(&self, repo: &Path, base: &str, head: &str) -> Result<String, String> {
        Ok(self
            .run(repo, &["diff", base, head, "--"])
            .await?
            .ok("diff")?
            .stdout)
    }

    /// The branch the operator's checkout has checked out, if any.
    pub async fn current_branch(&self, checkout: &Path) -> Result<Option<String>, String> {
        let out = self
            .run(checkout, &["symbolic-ref", "--quiet", "--short", "HEAD"])
            .await?;
        Ok((out.code == 0).then(|| out.stdout.trim().to_string()))
    }

    /// Fast-forward the operator's checkout to `commit`, which its object
    /// store must already hold.
    pub async fn fast_forward_checkout(
        &self,
        checkout: &Path,
        branch: &str,
        old: &str,
        commit: &str,
    ) -> Result<(), String> {
        if self.current_branch(checkout).await?.as_deref() == Some(branch) {
            self.run(checkout, &["merge", "--quiet", "--ff-only", commit])
                .await?
                .ok("merge --ff-only")?;
        } else if !self
            .update_ref(
                checkout,
                &format!("refs/heads/{branch}"),
                commit,
                Some(old),
                None,
            )
            .await?
        {
            return Err(format!("the checkout's {branch} moved"));
        }
        Ok(())
    }
}

/// Clear close-on-exec on `fd` in the child, so it inherits the descriptor.
pub fn inherit(command: &mut Command, fd: RawFd) {
    // SAFETY: fcntl is async-signal-safe; it only clears close-on-exec on a
    // descriptor this process owns.
    unsafe {
        command.pre_exec(move || {
            let flags = nix::libc::fcntl(fd, nix::libc::F_GETFD);
            if flags < 0
                || nix::libc::fcntl(fd, nix::libc::F_SETFD, flags & !nix::libc::FD_CLOEXEC) < 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

pub fn exit_code(status: std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

pub fn target_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

pub fn candidate_ref(attempt: i64) -> String {
    format!("refs/yard/candidates/{attempt}")
}

pub fn canonical_dir(root: &Path) -> PathBuf {
    root.join(".yard/local/canonical.git")
}
