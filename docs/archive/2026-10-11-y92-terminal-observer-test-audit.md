# Y-92 planner terminal observer repair

The operator observed exact head 2233412 in focused host session 52135.
Both fixtures reached successful positive-control planners. External SQLite
observations showed the fixture report, teardown=true, outcome=unchanged and
removed cleanup. No attention existed for those planned completions. The
session was still live when this finding was supplied. This record claims
neither its eventual result nor package acceptance.

The local ended helper waited for attention.raised, which only covers the
failure path. It now reads status --history --since, identifies the planner's
execution.started by the admitted ticket and implementation kind, and waits
for execution.ended with that exact execution, attempt, ticket and kind.
It reads supported attempt show and selects that execution by identity rather
than array position. The existing teardown assertion remains. The CLI source
exposes execution, report, result and teardown in each executions entry;
cleanup entries cannot satisfy the planner observation.

Successful planner cleanup can remove daemon-owned transcripts before the
host test reads them. The existing pinfold observer now tees real box exec
stdout and stderr separately into fixture-owned host files and forwards both
streams live. It waits for both readers before returning the original command
exit status. Each subsequent real exec appends, including git inspection.
The refusal and secret assertions read these observed streams instead of
the removed private attempt directory. No configured command, harness frame,
registration, response or launch input is substituted. Guest permission and
secret probes still run through real pinfold after the real command returns.

## test-audit Gate

Read the complete package test and its changed assertion blocks using the
test-audit skill. Retain all blocks. G8's packaged-controller Shown by clause
requires native publication after teardown, named pre-call missing-asset
refusal, declared-path refusal, tool/report bounds and the internal request
bound with same-project publication controls. G12/G13 cover route secret
containment and read-only source/trusted inputs with distinct writable state.

The observer change adds no assertion or guarantee. Existing expectations
remain outside Yard: report text, references, sentinel programs and key are
fixture literals; request/read/report limits, teardown and refusal tokens
come from the spec; route requests and guest exit statuses are external
observers. Matching event and CLI execution identities establishes agreement,
not an expected value copied from Yard's implementation. Removing checked
teardown or native publication fails the retained teardown/report assertions.
Skipping runtime identity, admitting an undeclared tool, executing candidate
code, leaking the fixture key or dropping the iteration bound fails the
existing distinct blocks and their documented sabotage.

The new declared traversal block arrived as working-tree edits and is retained
with its separate new dated Gate record. candidate/../large.txt selects an
existing fixture file if normalization bypasses the declaration guard. Its
null report, zero controller requests and invalid_input naming declared file
prove that pre-call boundary. Restoring large.txt in the same project supplies
the allowed control. The existing undeclared tool-key refusal exercises
allowlist lookup and cannot detect this sabotage. The new G8 clause describes
that distinction without weakening any existing promise.

Only the wait and transcript observer are replaced. No assertion is removed
or relaxed. Binary lines removed: 0. Assertion lines removed: 0. Test helper
lines replaced: 19, including the invalid attention wait and private transcript
walk. No sleeps, retries, deadline changes or binary lifecycle changes occur.
The real Pi still registers with native MCP and invokes the packaged child
inside the same Yard-owned box. Source/package pins, Y-97 controls, old
archives and repository .yard remain unchanged. Daemon selection remains
outside this package split until Y-93.

Validation results follow below. No image, host package or controller executes
in this candidate, and no boxed e2e runs here. Fresh exact combined source
Gate, focused package host tests, full suite, current review and actual landing
remain required.

cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked, shell observer syntax and git diff --check passed.
No e2e test was run. These candidate checks do not establish packaged runtime
acceptance or a terminal result for operator session 52135.
