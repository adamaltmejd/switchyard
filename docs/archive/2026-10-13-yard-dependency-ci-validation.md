# Yard dependency CI validation, 2026-10-13

Exact source: `213592e0573b21020e5c1fb77bc415c0207d82e3`.
Run: https://github.com/adamaltmejd/switchyard/actions/runs/38389060290. Workflow completed successfully.

Both real Linux native runtime gates passed:

- x86_64: test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 463.18s
  Full raw log SHA256: `36e42b72ca8f0d3016551bba2ae49dcfd8de67d195dcb27786a643b844f85df7`.
- arm64: test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 383.08s
  Full raw log SHA256: `96ae198d26dbbac6f564f3872049d54753b22a8eb8b3475a7bbf607c0fb641c0`.

Local corrected combined source also passed 79/79 in 788.26s; its first 78/79 failure remains retained. This proves this dependency/source checkpoint on Linux x86_64 and arm64, not final Y80 or Mac readiness.

Supported operator sync imported `213592e0573b21020e5c1fb77bc415c0207d82e3` without changing either original user file, index or prior stashes. Y136 attempt 133 is implementing against this exact base. Source templates use Rust 1.99 and the verified 1.2 public archive cache; active model/reviewer settings and installed operational binaries were preserved.

No main remote push, release tag/publication, installed Yard replacement, daemon replacement, reviewer switch or Mac SSH action occurred.
