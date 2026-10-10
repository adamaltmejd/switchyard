# 0.0.4 release preparation audit

Y-55 audit start: `66a81dd`. The landed dependency Y-54 ends at
`cbe1233`; `66a81dd` records its host validation. This pass reads the
whole current tree, not just the adapter diff. Three read-only reviewers
covered all Yard modules, all e2e sources and assertions, and spec traces
and user docs. All 37 built CLI help pages were read. The primary worker
checked manifests, workflows, mechanical findings and review dispositions.
The 2026-10-09 cleanup's rejected list was checked.

## Measurements

No tags exist in this worker clone. The skill's fallback base is root
`a482f4e`. The practical patch baseline is release commit `81da8c6`
(0.0.3); it is not represented as a locally verified tag.

| Measurement | Before | Prepared candidate |
|---|---:|---:|
| Yard Rust/TypeScript lines | 12300 | 12300 |
| E2E Rust lines | 6257 | 6257 |
| Largest source, jobs/supervise.rs | 1499 | 1499 |
| ARCHITECTURE.md lines | 826 | 843 |
| Runnable scenarios | 37 | 37 |

Root-to-start, all tracked files: +23057/-278, ratio 0.0121.
Patch baseline to prepared tree, excluding archive and operator `.yard`:
+246/-37, delete/add ratio 0.1504. This includes the previously landed
adapter, tests and build instructions. Y-55 changes metadata and prose;
binary and test source additions/deletions are both zero.
Root-to-tree excluding archive and operator `.yard`: before +20746/-237,
after +20772/-237; delete/add ratios 0.0114 in both cases. Archive reports
are excluded so recording the measurements does not change them.

## Source and mechanical dispositions

No source consolidation or rework was admitted; no cleanup commit was
needed. The twelve Yard dependencies match the spec. E2E's rusqlite and
serde_json support outside observation. The three argument-count lint
suppressions explain independent execution inputs. No TODO, FIXME, XXX or
revisit trigger was found. Git and pinfold spawns stay in their bounded
modules; host gates retain cwd, scrubbed environment, output/time bounds,
process group, lock and persisted handle ordering.

Rejected findings retain their prior conditions: shared harness validation
needs a concrete removed concept and control review; service-child boundary
and ps output changes need a resolved checklist contract or reproduced bug.
Approval rereads, child-first recovery, no-follow writes and proof bounds
remain controls. No new follow-up ticket was filed. Operator-owned cleanup
work is listed for judgment rather than started by this worker.

## Assertion sweep and operator list

No cross-file twin was found. Pi stderr registration and Claude stdout API
registration exercise distinct mechanisms. G12 lives in G8; G14 route and
listener observations live in G7; G2 repair and proposal observations also
live in G10/G15. All other scenarios are kept.

Two G7 assertion blocks need operator disposition. In
`g07_lands_end_to_end.rs:367-384`, the idle-board watch is spawned and the
ticket immediately unparked. A first poll after unpark can miss the
documented stale-drain sabotage. This doubt was already carried in the
2026-09-30 Linux audit. At lines 386-414, a watch past an open approval is
spawned and a second ticket immediately filed. A first poll after its
approval can miss an ignore-since sabotage. Both claim waiting that is not
deterministically observed. Each is the sole block for its clause, so the
disposition is operator-owned rewrite/judgment, not deletion. Neither is a
reproduced product defect. No sabotage run was performed in this box.

Tests and guarantee clauses remain intact. Test lines removed: 0; binary
lines removed: 0. The audit is complete as a read-only sweep, but these
two dispositions are pending. No green release audit is claimed.

## Spec pass and checks

Worker traces found no current-contract drift after adding the direct API
README setup. Store migrations and lifecycle/control modules are unchanged
from 0.0.3. The new Release section states the release boundary; it claims
no redesign feature. The operator's required ARCHITECTURE/README pass is
pending, including the two G7 judgments above. No answer is inferred.

Fmt, locked all-targets clippy and locked debug build passed. The release
build and hash are recorded in the companion release checklist. No e2e
ran here: the ticket explicitly reserves it for the host landing gate.
The latest measured host budget is Y-54's 37 scenarios in 150.21 seconds,
not a measurement of this candidate. Linux architecture gates and the
operator Mac checks remain release prerequisites.
