# Y-56 target binding assertion audit — 2026-10-10

This is the test-audit gate for the atomic serial target-binding slice,
not a whole-suite sweep. ARCHITECTURE.md changes with the implementation.
The ticket explicitly authorizes changing the default-selection contract:
live attempts freeze their branches, while open landing intents still
refuse selection. No concurrent landing queue is introduced.

## Dispositions and the operator's list

- **G2/G9/G10, feature target scenario:** replace
  `a_conflict_with_a_config_change_merges_from_the_bundle` with
  `a_ticket_binds_feature_inputs_repairs_and_lands_on_its_target`.
  It retains the conflict-path, bundle merge, fresh judgments, retired
  approval, new base and preserved operator-config observations. Keeping
  both would duplicate the bundle mechanism. Main warms the default inputs
  before feature/a runs. Three fixture image markers and host counters
  distinguish main, the first feature inputs and the changed feature inputs.
  A missing target writes no attempt. Its inactive revision-bound target
  edit is the control; a held live target edit names its attempt and branch.
  A same-head approval naming main writes no approval row; naming feature/a
  then succeeds. Git proves the initial base, repair target and final ref.
  Sabotages are resolving through HEAD, using default config/image inputs,
  supplying main's repair bundle, dropping conflict paths or target identity,
  keeping the old base, or carrying the old approval.
- **G3/G9, held landing intent:** keep the existing real update-ref hold
  and body-edit assertions. Add a target edit that answers with the same
  intent identity before the child is released. The other parked ticket
  accepts a target edit at its fixture revision. This differs from the
  held-worker refusal: an intent owns canonical, so waiting for that lock
  before checking the target guard cannot answer. The sabotage comment names
  that ordering. The intent comes from the externally read store.
- **G9/G11, explicit default selection:** adapt the prerequisite scenario.
  Selection succeeds while the real plan worker remains held on main.
  Its base equals the fixture's main commit and its branch stays main.
  After release it finishes as planned. A later plan starts at the
  fixture's new feature/a head. No duplicate named import/consume assertions
  are added. Sabotage is retargeting existing work or resolving future
  omission to main. Fixture git commits and the scripted model supply
  expected heads and outcomes.
- **Existing judgment scenarios:** add the now-required
  `--target main` to approve/reject calls. Their assertion blocks and
  observations are unchanged.

Each changed block has a matching Shown by clause. The observers are CLI
results, host git, store rows, the real model route and host counter files.
Expected revisions, markers, gate invocation counts and branch names come
from the fixture and spec. Comparisons between recorded image/config
identities establish the specified distinction between the fixture inputs.
No unit seams, public endpoints, sleeps, retries, test-only flags or binary
barriers were added.

The three image identities, six host counter entries and four reviews are
the configured main candidate/landing, initial feature candidate, renewed
feature candidate after sync, repaired candidate and feature landing.
Image and gate checks prove the chosen Dockerfile and config together.

The diff removes 105 test lines beside 105 binary lines. These counts
include command adaptation and replaced blocks; they are not a cleanup
ratio. One commit contains the atomic implementation, spec and evidence
changes required by the ticket.

## Store and validation

The forward migration adds a nullable ticket override and attempt branch.
New admissions always write the branch. Before recovery, old live attempts
bind the current symbolic HEAD: the prerequisite implementation refused
changing that default under live work. Old ended attempts keep an unknown
branch, since their rows did not record one. Recovery uses the frozen
branch for intent proof. No development daemon or CLI opened a project,
and no construction state, runtime names or `.yard/` files changed.

`cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings` and
`cargo build --locked` passed.
The host e2e suite and runtime sabotage trials were not run in this box,
as the ticket instructs. They remain the post-approval host landing gate.
