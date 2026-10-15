# Y80 successor migration and switchover handoff

Prepared 2026-10-15. This is a candidate runbook, not a release approval or a
record of successful host execution. Candidate version is 0.0.5, following
the existing 0.0.4 metadata. No installation, publication, configuration sync,
construction-state access or daemon replacement was performed by this work.

## Identities and artifacts

The fixture builds genuine 0.0.3 source
`81da8c677a0ab43966588429c24a8affedb9e8ec` once and verifies the executable's
`version --json`. Both historical upgrade scenarios share that build. The
previous `c85369dfaec8f8554da49ddf02a4dbec03bcad02` source declares 0.0.4;
its prior passing results are not 0.0.3 evidence.

The operator separately identified the installed frozen 0.0.3 executable as
boundary `05ae25832964703c`, SHA256
`05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
These separate source and binary identities do not prove reproducibility.
Keep the actual frozen executable with the private operational backup.

Release assets retain the existing names:

- `yard-0.0.5-x86_64-unknown-linux-musl`
- `yard-0.0.5-aarch64-unknown-linux-musl`
- `yard-0.0.5-aarch64-apple-darwin`
- `SHA256SUMS`

The tag-triggered release workflow builds these targets and publishes. Do
not trigger it during validation. CI's existing manual dispatch can validate
a temporary candidate branch without a tag. Artifact builds can use
`cargo build -p yard --release --locked --target TARGET` on the appropriate
isolated builder. Preserve the source commit, Rust version, target,
`yard version --json`, SHA256, schema before/after, and checks for each asset.
The local GNU release build in `/yard/proof/y80-artifacts` is boxed build
evidence only; it is not one of the static Linux release assets.

## Isolate before opening anything

Use an absolute, disposable project copy and an empty registry. Set separate
`XDG_STATE_HOME`, `XDG_CONFIG_HOME` and `XDG_CACHE_HOME` for both client and
foreground daemon. The socket is `$XDG_STATE_HOME/yard/yard.sock`; use a short
path such as `/tmp/y80-S/s` on Mac. Verify the resolved path is distinct from
the construction socket before any call. Run `yard daemon run`, never service
install/restart, for validation. Check status through that same environment.

Give validation images distinct names in the disposable configuration. Yard's
project/runtime identity derives from its absolute project path: use separate
paths for upgrade, restore and construction, and inspect actual runtime box
names and mounts before releasing any held effect. Never copy a construction
registry into the validation daemon without remapping its entries to isolated
copies. Runtime names, image names, state and socket must all be independent.
Do not prune shared runtime resources. Host runtime/security setup, including
the registered Mac runner, needs operator review before execution.

Use only the scripted host model route and real pinned harnesses in real
boxes. Do not supply public model endpoints or operational credentials.
Operator secrets remain in private host configuration, outside artifacts and
candidate files. Workers do not change installed `.yard` configuration.

## Stopped-writer backup and restore rehearsal

1. Record the old executable identity and the registry's exact project paths.
   Through the old supported interface, record ticket/attempt/execution IDs,
   open decisions, candidate heads/proofs/checks and canonical refs. Record
   live boxes and children and which held effects can still act.
2. Stop and reap the only daemon writer. A stopped SQLite writer alone does
   not freeze a surviving Git child or box. Quiesce operational effects or
   hold them at an externally proved point. Do not treat seeing a row/ref as
   holding that point. Preserve unknown ownership for operator resolution.
3. Snapshot the complete project tree, including `.yard/local/store.sqlite`,
   any `store.sqlite-wal` and `store.sqlite-shm`, canonical Git objects/refs,
   retained clones, proofs, checkpoints, effect locks and other required live
   artifacts. Copy the registry alongside it. Preserve file permissions and
   links; inventory external link targets/mounts and snapshot required data
   without sharing writable originals. Take the database/WAL together after
   writer exit; do not copy a live database piecemeal. Keep private operator
   configuration and the old executable in the operator's protected backup.
4. Hash/inventory the stopped snapshot, recording paths, sizes and modes.
   Seal that original backup. Make independent working copies for old restore
   and successor validation. Remap only the disposable restore registry to
   its copied project. No old executable may open an upgraded working copy.
5. Restore the unupgraded copy under the real old binary in its separate
   environment. Verify ticket/dependency/proposal/check rows, audit history,
   canonical objects and retained candidate bytes from outside. Stop and reap
   this daemon before switching that disposable copy to the successor.
6. Migrate the successor copy under its writer lock; record schema version,
   foreign-key integrity and original IDs/columns/audit prefix. Reopen once
   more and verify the schema and decisions do not change or duplicate.
   Keep raw failed, skipped where recorded, and overridden outcomes intact.
   Unknown historical plan/producer/usage evidence remains unknown.

The prepared G15 representative uses the old CLI only for historical setup.
It creates a completed ticket, parked dependent, ended read-only planner with
open child proposals, and a real failed candidate at review limit. Before
upgrade it observes the planned result, proposals, exact candidate Git head
and failed review. After migration the ended target stays unknown; accepting
the proposal preserves omitted child target and parent-blocking edge.
Repeated accept/reopen refuses resolved identity and cannot duplicate children.
The exact candidate-bound attention supports override and one native landing;
raw failed checks stay failed. Every historical execution/check column and
old audit prefix is compared rather than synthesized.

Its separate restored old copy proves the approve-at-limit exit, then holds a
real native ref update before mutation. With the old writer dead it proves
the child's PID, old canonical ref, approval override and open intent before
snapshot/startup. Successor raises ownership attention on that exact intent.
After the old child finishes, canonical agreement alone cannot release an
unproved legacy owner or replay landing. Another existing machine retains the
FIFO-held pre-box child, exact reservations and capacity waiter across upgrade.
No store rows are forged. These fixture assertions are prepared, not run here.

Ambiguous active mappings, missing required artifacts, unproved ownership,
foreign-key violations, stale target/proof/check decisions or a changed
canonical ref stop the rehearsal for the operator. Never guess successful
conversion, fabricate a historical target, force-release a reservation or
reverse-migrate. Preserve the failed copy and evidence. Return to the old
binary only on a fresh, unupgraded backup copy with exclusive writer/effects.
A backup is not permission to rewind canonical changes made after it.

## Gates and operator decision

Boxed `cargo fmt --check`, locked all-targets clippy and build are necessary.
They do not exercise migration/restore, native Git/boxes or Mac runtime.
Before approval obtain exact-candidate full host gates on Linux x86_64 and
arm64 and the operator's Mac release validation, including reviewed runtime
security, short socket paths, migration/restore and versioned assets. Use
`cargo test -p e2e --locked` only on those hosts with the required pinned
runtime and verified Durable inputs. Archive raw results, hashes, elapsed
wall time and actual fixture/build/box/model counts, including failures.
All these fresh host/platform results remain pending for this candidate.

The retained normal-graph trials cover five isolated implementers of one
frozen ticket, the same reviewer on each exact candidate, candidate selection,
and one implementer with multiple reviewers plus finding selection. Review
Y133 exact source-model/finding provenance, opportunities, applied choices,
raw losses/failures, time/token/cost and cleanup/restart observations in the
host result. Measured local usefulness is not objective correctness. The
cache-inclusive Pi scenario stays intact; do not replace real provider fields
with synthetic runtime frames or infer measured cost from rates.

After those gates, the operator reviews ARCHITECTURE/README and the cleanup
record, then decides publication, installation, credential placement,
configuration commit/sync, daemon replacement and any push. Immediately before
operational switchover take a new stopped-writer consistent backup and repeat
identity checks. Preserve that backup and frozen executable. Post-start verify
registered paths, schema, current attention/ownership, heads/proofs/checks and
no duplicate effects through supported operator MCP/API. Keep unresolved
legacy ownership stopped. No automatic release or construction switchover is
a worker effect.
