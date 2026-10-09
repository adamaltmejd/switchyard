# Release cleanup, 2026-10-09

Release base: `v0.0.2` (`b9d0b23`). Clean, green audit start:
`5f454bf`. Source cleanup: `de141d6`. Release candidate: 0.0.3.

Three reviewers read all jobs, store, git, box, harness, agent environment,
MCP, fixtures and guarantee tests. The operator agent read the daemon,
API, CLI, configuration, entry point, build files, workflows and user docs.
All 37 built CLI help pages were read. The whole-tree traces covered
configuration, commands, tools, reasons, attention, events and G1–G15.
The prior cleanup's rejected list was checked before judging new findings.

## Measurements

| Measurement | Before audit | Release candidate |
|---|---:|---:|
| Rust and TypeScript lines in crates | 18389 | 18387 |
| Yard source lines | 12256 | 12254 |
| E2E source lines | 6133 | 6133 |
| Largest source: jobs/supervise.rs | 1499 | 1499 |
| ARCHITECTURE.md lines | 814 | 814 |
| Independently runnable scenarios | 36 | 36 |
| Tag-to-tree additions / deletions, excluding this report | 62 / 14 | 66 / 20 |

Audit source diff: +1/-3, delete/add ratio 3.00, net -2.
Binary source removed: 3 lines; test lines removed: 0.
No dependency was added or removed.
Release delete/add ratio including this report: 0.1361 (20 / 147).
The report contributes 81 lines.

## Dispositions

`de141d6` removes duplicate path canonicalization in `cli.rs`.
Its only caller already supplies a canonical path. Relative paths,
symlinks, linked worktrees and upward project discovery retain their behavior.
The commit passed format, Clippy and the locked build.

The test sweep found no invalid block or twin to delete. All existing
Shown-by clauses remain covered. G12 lives in G8; G14's local listener
and route observations live in G7; G2's repair approval and stale accepted
proposal observations live in G10 and G15. No sabotage claim was doubted,
so no mutation experiment was needed.

No new rework ticket or control change was admitted. Historical proposals
for shared harness validators, service child boundaries and a ps output cap
remain deferred: no re-admission condition fired. Argument structs still
remove no shared state. Approval rereads, recovery ordering, no-follow
writes and proof bounds remain controls. No operator finding is pending.

## Mechanical checks and spec pass

Yard's twelve direct dependencies match the Code section. E2E's rusqlite
and serde_json serve outside observation. The three lint suppressions
explain independent execution inputs. No TODO, FIXME, XXX or fired revisit
trigger was found. Git and pinfold spawn only through their controlled
modules; host gates retain their cwd, scrubbed environment, bounded output,
timeout, process group and lock.

ARCHITECTURE.md and README.md state the current contract. No resolved
open question or repeated rationale needed removal. The guarantees table,
box table and worker grants were retained. CI pins Pinfold 0.2.2.
The existing dependency-update edits were preserved in `5f454bf`.

## Validation and host upkeep

Linux x86_64, yard-sthlm, rootless Podman 5.4.2, Pinfold 0.2.2:
format, Clippy with warnings denied, locked debug and release builds passed.
Baseline: 36 scenarios passed in 141.58 seconds.
Final 0.0.3 source: 36 passed in 135.78 seconds, no retries.
The final log is `/tmp/switchyard-v0.0.3-e2e.log`.

The operator explicitly waived the Mac host gate for this patch.
GitHub's release workflow retains both Linux architecture gates and the
macOS binary build. Their results are not claimed by this host report.

Pinfold's installed release checksum matched its published SHA256SUMS.
A fresh user `default` profile was copied from the 0.2.2 built-in and
its image rebuilt. Old profile and dangling images were reclaimed.
After the suite and final prune, Podman listed no containers.
No open local ticket, GitHub issue or pull request existed at inspection:
the local store held 48 done tickets and 5 abandoned tickets.
