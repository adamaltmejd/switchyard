# Y-157 filter decoding and selected fixture inputs — 2026-10-15

Review3697/check1926 identified a P1 in `commit_source`: Node's default
UTF-8 replacement decoding could change a raw filter subsection name.
The resulting override would disable a different filter before `git add`.
Git output now remains bytes within the existing 32768-byte stream bound.
Filter discovery uses fatal UTF-8 decoding before any add or commit.
Invalid encoding returns `source_commit_failed` through the existing tool
failure path. Hooks, fsmonitor, signing, environment, cwd, time bounds and
unsafe replay declarations are unchanged. No new package or lifecycle exists.
ARCHITECTURE.md states this implemented refusal.

## Test-audit gate

The retained G9/G13 selected cleanup case keeps its audit refusal, ordinary
Git positive controls, held unsafe write, interrupted-state observations,
exclusive ownership, checked release, continuation, independent publication
and landing checks. It also keeps the nested Git refusal and its native
positive control. No existing observation or expected source value changes.

The operator's native 5e19c0 run ended the ordinary implementer execution
before cleanup, with exit 1, no terminal frame and an empty transcript.
Its retained staged extension and sentinel are an investigation lead.
The fixture gave repair selected inputs but left the implementer on broad
trusted-base guidance. Give the implementer its own explicit selected skill
using existing admission and staging semantics. Keep hostile unselected
extensions and guidance in the fixture for packaged audit controls. No
production staging behavior changes.

Before either initial cleanup latch, observe the exact attempt's implementer
terminal event through the existing history stream and require `candidate`.
The execution's node is read from the external store. A failed predecessor
now fails there instead of waiting for an unreachable cleanup request.
The expected candidate outcome follows the graph's ordinary candidate
producer contract and the fixture's real Git commit. The comment names
staging broad base extensions despite selected inputs as sabotage. This
protects the representative handoff's G9/G13 isolation and truthful outcomes.
It adds no machine, build, box, model turn or alternate assertion matrix.
Reuse of the existing difficult case is sufficient; no new runtime test
targets the non-UTF-8 subsection variant in this candidate.

Test diff: 22 lines added, 3 removed. Runner diff: 7 lines added, 3 removed.
This is a test-change gate, not a suite sweep.

## Validation and retained evidence

`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed. No e2e command
ran in this candidate environment, as instructed. Node is not installed
here; no local runner execution or real-box pass is claimed.

The host evidence directories are not available in this checkout. The
earlier bare-canonical observer failure remains separate from the ordinary
implementer failure:

- `/home/yard/validation/y157-3af1e6-durable-20261015-evidence`
- `/home/yard/validation/y157-5e19c0-durable-20261015-evidence`

Neither directory was changed. The actual ordinary-role failure cause
still needs host confirmation. The complete affected case and final
integrated host gate remain required before approval; neither failed run
proves successful unsafe-write interruption, continuation or landing.
