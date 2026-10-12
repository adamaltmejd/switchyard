# Y-101 review 996 input-boundary repair and test-audit

Review 996 identified two P2 defects in the implemented candidate 9e003a6.
This repair changes their source boundaries. Earlier historical archives are
unchanged. Proposals 227/228/229 are superseded, not prerequisites. No target
merge was needed; no canonical Y-95 proof or architecture change was replaced.

## Frozen admission across upgrades

`Loaded::inputs` now resolves canonical inputs only. Admission records the
current package identity once; inspection previews the current identity.
`load_attempt` checks the recorded package identity's schema and validates the
original canonical selected regular-file identities separately. It compares
all remaining canonical admission fields and the resolved plan exactly.
It never compares the recorded package hashes with hashes recomputed from a
later daemon's embedded package bytes, and it never rewrites those hashes.

This removes the path by which a new binary could refuse a live attempt as
an admitted-source mismatch merely because packaged bytes changed. Execution
still generates launch input from the original recorded package identity.
The unchanged native Pi bridge compares those hashes with the image-owned
launcher, controller and lock before launching package code. Missing or
incompatible image assets still name `runner_unavailable` and the asset before
model calls. The launcher retains its sealed closure and Node checks. Input
identity, observed runtime image identity and eligible result remain distinct.

## Scoped canonical input reads

`read_config` again inventories only the configuration/Dockerfile paths and
the existing ordinary agent-environment allowlist. `Loaded` no longer retains
or clones a whole-repository inventory. Unrelated source trees cannot exhaust
the existing 16 MiB Git output bound during ordinary configuration reads.

Packaged resolution queries only the declared skill and reference paths at
the selected canonical commit. These queries check each selected object type
before literal, nonrecursive `ls-tree` selection. Directories are excluded
without inventorying them; path metacharacters do not broaden selection.
The regular-file, path, byte and count bounds remain in force. Ordinary environment directory
reads retain their existing recursive scope.

Selected metadata is read before the existing admission transaction. The
transaction still checks readiness, ticket revision/target and current lane
capacity before recording admission and allocating its first execution.
The explicit path also refuses a revision change during preparation. No
cache, lifecycle, owner, recovery or reservation abstraction was introduced.
No secret or installed `.yard` input changed.

## Scoped test-audit disposition

Both complete `g08_durable_package.rs` scenarios are kept byte-for-byte.
Test lines removed: 0; binary lines removed: 41. No assertion, report oracle,
fixture hold or guarantee clause was removed or weakened. This is a scoped preservation audit using
the test-audit skill, not a suite sweep or fresh runtime evidence.

| Retained clause | Named sabotage | Distinct hardest case | External oracle |
|---|---|---|---|
| G8 explicit selection, native publication and identity refusals | Restore outer-model election, skip identity checks, accept packaged topology or ignore child status | Missing sealed asset then corrected image in one project; every model request is a controller request; held native publication remains ineligible | Fixture schema-valid test-audit report, host requests, operator MCP/CLI refusal identities, external session/registration/result rows |
| G8 package bounds and bare-answer refusal | Remove request/read/declaration limits, count report characters as bytes or synthesize a report from text | Traversal normalizes to an existing file, oversized multibyte summary, endless reads and bare answer followed by positive publication controls | Spec's byte/request limits, scripted tool results, external report/output/outcome eligibility |
| G8/G13 selected canonical instruction authority | Stage broad/candidate inputs or current canonical files instead of admitted blobs | Later canonical skill/reference and candidate skill changes followed by edited replacement | Host Git blob identities, fixture selected/excluded markers, hostile sentinels and boxed read-only probes |
| G12 containment and native route | Persist bearer/key or bypass injection | Selected controller runs with guest/source/proof/input/state/package/store/log/model-body probes | Fixture key reaches host route header only; guest probes, captured streams, external store reads and model bodies |
| G13 corrected Y-99 execution isolation | Reuse state, resume predecessor, omit/widen proof or follow checkpoint links | Completed predecessor exec held with its live box; replacement controller response held while old checkpoint remains on host | Fixture holds/status history, real box checkpoint/link planting, host checkpoint/sentinel, fresh proof/state and node/resume rows |

The repair's upgrade and unrelated-tree conclusions come from source-boundary
inspection, not a two-binary host run or a large-tree runtime measurement.
No new assertion claims either pass. The native bridge, launcher, controller,
both package scenarios, Y-99 isolation blocks, ownership, publication and
execution supervision are unchanged. The latent P3 additional-input event
finding was left outside this one-prompt execution scope.

ARCHITECTURE.md records the authoritative admitted package identity and scoped
canonical reads. G8/G12/G13 and all original Y-101 acceptance remain intact;
G12/G14 and exact ownership/identity/publication controls are preserved.
The dependent selected-controller supervision acceptance is still required
before Y-100 can complete; Y-100/Y-93 retain their full integration contract.

## Validation

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed in the candidate
box. The complete current e2e scenarios compile under clippy. Host e2e was
not run, as required by the operator. It remains the host landing gate after
approval. No sleeps, retries, binary barriers, public models, mocks or timeout
or concurrency masking were added.
