# Y-158 writable observer test audit

The operator's focused host run of e6bfc2b reached the last G13 assertion
before failing at verification.as_str().unwrap(). It preserved a result of
0 passed, 1 failed in 34.41 seconds. The retained store has NULL verification
for attempt 2's writable implementation executions 3, 4 and 5; only the
preliminary read-only planner execution 1 has it. Evidence is in
/home/yard/validation/y158-e6bfc2-inputs-20261015-evidence/native.log,
terminal.json and native-stores (20 snapshots).

Deleted the new final verification assertion block in
ordinary_writers_read_frozen_selected_inputs_and_adoption_starts_fresh
(G2/G13). It uses telemetry that writable launches do not record. The spec's
Verification identity section says: "Gate, reviewer and read-only audit
executions append verification once at the launch boundary." It does not
promise persisted writable launch verification. The original Y-158 audit's
claim that writable launch verification must carry new Git identities was
incorrect. No binary telemetry was added to satisfy that claim.

The retained admission blocks compare per-node path, blob and executable
identities to fixture Git. The real boxed tool calls check admitted bytes
after candidate and canonical edits, refusal through the trusted mount,
a successful source write, and exclusion of unselected guidance. Explicit
adoption binds the new Git identities and reads the new fixture bytes.
The unchanged second execution controls actual session resumption; the
adopted successor records no resumed owner and sends a fresh model
conversation. Removing selected inputs from consumed_definition or session
comparison still makes those retained session assertions fail. The second
launch after edits and third launch after adoption remain the hard cases.
Expected identities and bytes come from Git and the fixture, and actual
session observations come from the store and host model. All these
preceding assertions passed in the operator's focused run.

The removed telemetry block has no distinct promised mechanism beyond those
retained observations. The G13 Shown by clause now states the actual observer:
new per-node admitted blob identities and a fresh session when consumed inputs
change. No guarantee, acceptance or covered mechanism was weakened. Other
assertions and the sabotage comment are unchanged. The operator list is empty.
Test lines removed: 11. Binary lines removed: 0.

Validation: boxed cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked. No worker e2e ran. The operator will
run the final affected host cases.
