# Y-67 before-host scenario corrections

Review1623 found two deterministic fixture errors before any native run of
fe40659. This candidate batches both corrections and the missing writable
command control. Original Y-67 scope and required host gates remain intact.
No production code, historical advancement fix, capacity mechanism or
project `.yard/` configuration changes in this correction.

The required scenario remains
`concurrent_readers_share_capacity_and_join_exact_results_across_restart`.
Its graph is still the literal implementer fan-out to candidate-gate-read,
review-0 and review-1, followed by review-2 requiring candidate, command,
left and right results. There is no selected graph landing gate. The
independent ordinary ticket retains candidate-gate-read, three sequential
reviews and landing-gate-read. Attempts remain 1/2 for completed planning,
3 for the graph and 4 for the independent ticket. The fixture cap remains 2.

The original graph stays immutable in the fixture. The scenario captures
the complete original admission while implementation is held and checks its
plan against that literal graph. After revision 2 it compares the complete
original admission unchanged, then compares the revised literal graph to
`attempt.plan.plan`. `attempt.admission.plan` remains revision 1. Final Y-1
approval explicitly names target main, the host Git head and plan revision 2.
Y-2 retains its revision-1 approval identity and feature/a target.

The boxed graph command still attempts `touch forbidden` and exits 97 if
the write succeeds. Its passing exit 0 establishes refusal. In the same
project, the ordinary command now requires independent.txt, successfully
performs the corresponding `touch forbidden` and checks that file exists.
Its actual candidate gate must end pass with exit 0 before the existing Y-2
approval control. Both commands use fresh private gate checkouts; the
ordinary landing consumer also performs the write. Final canonical fixture
bytes and checked cleanup expectations stay unchanged.

## Test-audit gate

Keep the corrected blocks. G13's read-only command clause now names its
same-project ordinary writable control. G2's immutable admission and exact
plan identity contract distinguishes original admission from current plan;
G13's fresh-supplier/revision clause still uses exact named results. The
test comment adds sabotage that rewrites admission or mounts ordinary
commands read-only, alongside its existing writable-reader and stale-result
sabotage. Expected graph, revisions, exits and bytes come from the fixture;
original-admission equality observes the specified immutability relation.
The current plan is compared to a revised fixture literal, never copied
from runtime output. This revises the existing hardest held restart/revision
scenario; it introduces no twin or new race. Existing different-target cap
coverage remains the survivor and is not duplicated.

The full authored selection and adjacent lists/filters were read together.
Revision 1 has one command, left, interrupted right, replacement right and
one join. The passing-right filter is read before revision 2, so it names
only the replacement. Revision 2 has one fresh command and exactly three
review rows, ordered by their literal review node names; the join binds the
fresh command/left/right IDs with supplier revision 2. Left and the first
join row snapshots remain unchanged. Opening totals remain left 2, original
right 2, revised right 1 and join 2.

Fixture holds and the same-inode crash lock remain intact. Retained-history
`find` observes exact releases and the first approvals. After restart a new
watch begins at 0. Future `until` observes revision-2 approval after the
revised hold. Landing and cleanup still select the exact tickets, literal
attempts 3/4 and cleanup execution releases. Real CLI subprocess status is
preserved; no sleep, retry, binary barrier or extra prerequisite is added.

Test lines removed: 6, replaced for the command control and distinct
original/current identity checks. Binary lines removed: 0. No assertion or
guarantee is weakened. No native e2e was run; required serial host acceptance
and the full landing gate still follow correctness review.

Boxed checks passed: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check`.
