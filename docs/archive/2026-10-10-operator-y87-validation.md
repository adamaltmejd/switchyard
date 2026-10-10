# Y-87 operator validation — 2026-10-10

Ticket Y-87, attempt 63, revision 4. Base:
`d96aaa0a987e058a3b1fbcfca83707c25c02a030`.
Candidate: `92764a50d5b8eab7cc33fbaacfe1d95fa0becce8`.
Proof: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

This prerequisite records immutable resolved admission evidence and exposes
it in attempt/status inspection. Selected definitions and real configuration,
source, build-context, trusted-staging and runner identities are captured
with admission and its first execution. Repairs and restarts preserve the
record. Historical rows whose admission was never recorded stay explicitly
unknown; migration fabricates no historical input identity.

Admission evidence does not freeze execution or permit result reuse. Current
template lookup, digest invalidation, mutable-base trusted staging and
legacy approvals retain their existing behavior. Full atomic consumer
replacement remains Y-85, which retains all its original freeze, exact
identity and live-safeguard acceptance criteria.

The operator read the entire initial 10-file delta and every repair, including
binding architecture, the nullable migration, admission transaction,
selected definitions, shared staging predicate and all changed assertions.
The predicate shares the existing regular-file allowlist without expanding
it. A dedicated bounded Git root-tree read preserves the existing commit
resolver's contract. No new dependency, lifecycle or redundant execution row
was introduced.

Test-audit retains the three existing real host scenarios. G6 uses its
literal fixture plan and initial revision at actual admission. G10 uses an
explicit non-default configuration, full Git-derived input equality,
pinfold's harness pin and a host hash of the real binary. Candidate base,
templates, Dockerfile and skill inputs change in a real conflict repair;
the admission record stays identical through repair and quiescent restart.
The store proves no execution is running before that restart. G15's real
historical binary supplies ended planner and live limit rows; both stay
unknown while existing proposal and approve-at-limit controls succeed.
No assertion was removed or softened, and no twin scenario was added.

The first candidate, 12cae4b, passed boxed checks but failed the full host
suite: 0 passed, 53 failed in 40.10 seconds, exit 101. The new reader asked
the commit-only Git::rev_parse helper for a tree; its appended commit peel
made every normal configuration read fail. The operator missed this helper
contract in the initial source review. The host CLI controls caught it.
External Git confirmed the mismatch; supported revision-bound edit sent a
focused bounded-tree-read repair while preserving all assertions.
Log: `/tmp/switchyard-y87-12ca-host.log`.

Candidate bf25cf9 fixed that defect. Its full host run passed 52 scenarios
and failed one in 204.88 seconds, exit 101. The new fixture expected only its
custom skill and AGENTS.md; real initialization also commits three embedded
operator skills. Capture correctly included them. Their host Git source
hashes independently matched the observed inputs. The failed fixture had
already cleaned itself up. The next repair retained full equality and added
literal paths with blob values read from host Git in the live fixture.
Log: `/tmp/switchyard-y87-bf25-host.log`.

The operator read skill-corrected candidate 8a02a1 but ran no host suite on
it. A further test-audit finding required the expected definition values to
come from explicit fixture declarations, rather than undocumented parser
defaults. Candidate 92764a5 declares benign non-default workflow, gate and
review settings and names default-substitution sabotage. Original behavioral,
immutability and legacy assertions remain. Dated worker evidence records
each correction without rewriting earlier archive files.

Fresh boxed candidate fmt/check 704, clippy/check 705 and build/check 706
passed. In isolated checkout
`/home/yard/validation/switchyard-y84-20261010`,
`cargo test -p e2e --locked` at exact clean 92764a5 passed all 53 scenarios,
zero failed, in 202.60 seconds. Session 94995 ended with exit 0. The checkout
remained clean at the same head. This checkout was reused only after prior
host executions were terminal; their exact-head logs were preserved.
Log: `/tmp/switchyard-y87-927-host.log`.

Review execution 1325 published check 707: pass, round 1, no findings.
The exact approval diff matched the fully read and host-tested candidate
except the CLI trailing newline. Protected architecture hunks were read.
Approval 75 bound the exact head and empty proof above.

Landing execution 1326 ran fresh fmt 1327, clippy 1328, build 1329 and
host e2e 1330. All passed. The host suite passed all 53 scenarios, zero
failed, in 240.13 seconds. Landing was recorded at audit seq 4588 for
92764a5; ticket completion and attempt termination followed at 4589 and
4590. Cleanup 1331 ended removed at 4592. Supported yard sync consumed
the exact landing into the operator checkout.

Y-85 now has its landed admission prerequisite. Its complete atomic freeze
scope remains, followed by Y-86 and Y-59's full integration scope. Y-60
and Y-66 remain dependent on Y-59. Deferred Y-81 stays parked. The useful
redesign release and required platform validation remain incomplete.
No push, publication, installation, operational daemon replacement or
model/reviewer switch was performed.
