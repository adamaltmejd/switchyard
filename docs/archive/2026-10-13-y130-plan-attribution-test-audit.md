# Y-130 read-only plan attribution

Base: `5769da1535b219fb0630cfd88b73ed668867ff6a`.
Only the existing attribution/content block in `g08_audit_reports.rs`
changes. No runtime, spec, scenario, ticket-edit branch or other assertion
changes. This is a scoped test-audit gate, not a suite sweep.

## Test-audit gate

1. G9's selected read-only implementer clause shows superseded publication
   refusal and publication on the new revision with its diff. G8's frozen
   report binding and publication assertions remain. The replacement's
   public model request `last_user` is the observer.
2. Actor-only sabotage changes only the production plan annotation to
   `Plan revision:` while retaining its full Git diff. Diff-only sabotage
   omits only that diff while retaining the operator annotation. Comments
   name both beside the changed block; earlier sabotage comments remain.
3. The fresh replacement has `max_session_executions = 1`. Initial full
   instructions already contain the marker, making unscoped presence an
   insufficient observer. The writable capacity-waiting handoff observes
   another mechanism and remains distinct. All held-publication, history,
   missing-report, superseded-report, frozen-identity and upgrade checks
   remain in their existing scenarios.
4. Git supplies public diff and hunk addition semantics. The fixture supplies
   `Revised audit input`; the spec supplies the operator actor. The local
   scan selects a diff containing that marker in added hunk content, then
   checks only its immediately preceding nonempty annotation for wholeword
   `operator`, ignoring case. It clears the annotation after another diff
   and never falls back to a different diff after selecting the marker.
   Initial instructions, deleted/context lines and unrelated headings cannot
   supply the added marker or the selected annotation. No private filenames,
   hashes, serialized plan shape or generated sentence are expected values.

Three old test lines are replaced by 46 lines. Zero binary lines are removed.
There is no shared parser or new dependency.

## Worker validation and limits

Toolchain: `rustc 1.98.1 (48a229cea 2026-09-01)` and
`cargo 1.98.1 (797e8a9bc 2026-08-05)`.
`cargo fmt --check` and
`cargo clippy --all-targets --locked -- -D warnings` passed.
`cargo build --locked` passed.

Source SHA-256 pins:

- `crates/e2e/tests/guarantees/g08_audit_reports.rs`:
  `98e01bb5df0c5953e464a916a271a95e172b63b7e34c5510ad35703ea079c248`
- Unchanged `crates/yard/src/jobs/supervise.rs`:
  `78b1c7070841f44746fea42051512716bc804209327f0814570b589e75a5c6ea`

No native scenario, sabotage or full e2e suite ran in this worker box.
No host assertion was reached; no first host failure was observed here.
These checks establish compilation only, not the sabotage outcomes.
The prior operator evidence at
`/home/yard/validation/y111-operator-evidence-20261013/audit-g09-g08-plan-attribution-refresh-5769da1.md`
is outside the worker mount. Its prior failures remain evidence; this change
does not replace them or claim they passed.

## Deferred operator qualification

After boxed build and review, qualify an isolated real candidate host source
without overlapping host validation. Pin candidate commit, source and binary
hashes for every run. The focused command is:

```sh
cargo test -p e2e --locked a_plan_revision_hands_off_a_selected_read_only_implementer -- --nocapture
```

Run unchanged source first. Then separately change only the plan heading to
`Plan revision:` with its full diff retained; record the first failure at
the attached-operator observer. Restore source, then omit only the plan diff
with its operator heading retained; record the first failure at the added
marker observer despite the initial revised instructions. Restore source,
then use `Operator plan update:` with full diff; require success. Restore
the exact production source and rerun the focused command. For each run,
retain the commands, mutations, hashes, first failure, reached and unreached
blocks, and all earlier failures. These are required observations, not
results inferred from static mutations. Finally run the host landing gate:

```sh
cargo test -p e2e --locked
```
