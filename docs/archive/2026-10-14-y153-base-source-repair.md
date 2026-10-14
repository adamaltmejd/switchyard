# Y-153 bounded pre-host repair

Review found two defects before the host slot was used. The semantic candidate
retained the base tree but omitted the exact base source referenced by the
declared environment. The new box-up observer returned tee's exit status,
which could hide the real Pinfold owner's exit.

Semantic identity now includes the existing base commit source alongside its
tree. This is a source identity already recorded by Yard, not an environment
value dump or fingerprint. The observer uses the established per-process FIFO
pattern. It forwards and records real stdout, saves the real Pinfold status,
waits for the reader and returns the saved status.

The same independently runnable G2 block remains the focused host gate:
`closed_box_gate_inputs_are_declared_stable_and_immutable`.
No assertion was weakened, no scenario was added, and deadlines and concurrency
were unchanged. The approved trusted runtime contract remains the boundary.

Test-audit disposition:

1. G2's closed-input identity clause now explicitly observes the exact base
   source as well as its tree. Both genuinely equivalent gates still must have
   identical semantic identity.
2. Named sabotage: drop the exact base source from semantic identity. The
   comment names it; the new assertion requires the actual source commit.
3. An empty fixture commit creates different base commits with equal trees.
   This catches substituting tree equality or the preceding source commit for
   the declared source. Existing equality assertions alone could not catch it.
4. The fixture creates the empty commit; host Git supplies both commit IDs and
   proves tree equality. The semantic record must name the new base commit,
   retain that tree and differ from the preceding commit. No expected identity
   is derived from Yard's identity construction.

The FIFO correction adds no plumbing-only scenario. The existing observer
continues to run real Pinfold and preserves its exit without a pipeline status
substitution. Original execution/result/check immutability assertions remain.
The original dated disposition is retained as history. This note records the
repair, not host validation. The operator runs the existing focused scenario
from the integrated exact candidate next, then the required full landing gate.

Repair diff: 2 test lines and 3 binary lines replaced; no assertions deleted.
Boxed `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed. No host suite ran in this worker box.
