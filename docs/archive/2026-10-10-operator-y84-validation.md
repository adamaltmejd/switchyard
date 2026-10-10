# Y-84 operator validation — 2026-10-10

Ticket Y-84, attempt 61, revision 2. Base:
`10d1153db99c95527cfeefc9065b953b598364c5`.
Candidate: `6f70b68204f51d1cd88fd2b04f0846010436a588`.
Proof: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The first candidate, bdb8e63, passed boxed checks but failed operator source
review of the ticket's equivalent-default-plan requirement. Its translation
included candidate-stage gates once, although the native queue runs every
gate again on the merged ref. Its landing inputs also described the
implementer's candidate. No host pass or review verdict was claimed for that
candidate. Supported revision-bound ticket edit sent this focused repair
back to the worker and superseded the running first review.

The corrected plan uses distinct phase-qualified gate instances and the
existing typed external merged input. Native gate behavior is preserved;
private writable gate checkout access is described truthfully. No graph
runtime, admission selection, active-plan freeze or approval change is added.

The operator read the full initial delta and every repair hunk, including
binding architecture, schemas, target serialization, resolver validation,
cycle bounds, runnable equality and new assertion blocks. Test-audit Gate
keeps fixture-derived plans, exact named refusals, same-project controls and
read-only no-admission observations. The native control advances the target
before approval, then lands and observes the three fixture-declared check
instances and their boxed git inputs. G7 retains the general worker-content
assertion removed here. The two repeat refusals exercise separate explicit
zero-value and cycle-detection mechanisms.

Fresh boxed fmt/check 691, clippy/check 692 and build/check 693 passed.
The isolated checkout `/home/yard/validation/switchyard-y84-20261010` ran
`cargo test -p e2e --locked` at the exact candidate: 53 passed, zero failed,
in 231.93 seconds. Session 95474 ended with exit 0; checkout stayed clean.
Log: `/tmp/switchyard-y84-6f70-host.log`.
This includes the new G6 control and all previous 52 scenarios, including
legacy migration and ownership recovery.

Review execution 1297/check 694 passed round 1 with no findings. The
operator reread approval show and verified the approval diff equals the
fully read candidate diff, apart from a trailing newline. Approval 74 binds
the exact head and proof, with no blocking override.

Landing execution 1298 completed its fresh merged-ref gates:
fmt 1299, clippy 1300, build 1301 and host e2e 1302 all passed. The host
landing run passed all 53 scenarios, zero failed, in 225.38 seconds.
`landing.recorded` is sequence 4484, ticket done 4485, attempt ended landed
4486. Cleanup execution 1303 ended removed at 4488. Supported sync consumed
candidate 6f70b68 from base 10d1153 at 4489; the operator checkout is clean
at that head. Evidence: `/tmp/switchyard-y84-landed-show.json`,
`/tmp/switchyard-y84-landing-history.jsonl` and
`/tmp/switchyard-y84-cleanup-sync-history.jsonl`.

The review's duplicate clippy invocation encountered crates.io CONNECT
proxy refusals (403). That failed tool run was not counted as a passing
check. The recorded fresh candidate and landing checks above passed; the
review resumed source inspection and published no findings.

Y-84 is fulfilled. Y-85 now has no unresolved dependency and is the next
admission, after this operator evidence is committed and imported. Y-86
remains parked behind Y-85. Parent Y-59 remains parked behind Y-86 and
retains its full integration scope; Y-60 and Y-66 remain dependent on it.
The useful redesign release and required
platform validation remain incomplete. Y-81 remains parked.

The operational daemon PID 965905 and `/home/yard/.local/bin/yard` still
have SHA-256 `05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2`.
No push, publication, installation, daemon replacement or reviewer switch
was performed. Candidate work used the configured Sol 6.1 medium profile
and existing fast-tier request; the configured DeepSeek reviewer remained
in use. No new ticket or control waiver was needed for the focused repair.
