# Y-82 candidate repair assertion gate — 2026-10-10

Scope: the operator's P1 lock finding and the review issues described in the
repair request. The full text of check 583 was not exposed by yard_context
or available in this workspace. This note records the addressed mechanisms,
not a claim to have inspected additional unpublished findings.

## Dispositions

Keep all existing G3 and G13 scenarios and assertions unchanged. The held
merge-tree and update-ref mechanisms require ordinary edits to answer
before releasing their children. Only a workflow edit now waits on
canonical, with an intent refusal checked first and again in its mutation
transaction. Proposal edit/link acceptance and rejection read no execution
configuration and take no canonical lock. An open ticket proposal's
acceptance still serializes its configuration reading and creation with
default selection. Default selection and admission retain serialization.

Keep G7's closed-list assertion. Add `sync.default` to its expected event
list because the binding Store contract already includes that event.
Expected allowed names come from ARCHITECTURE.md, not runtime output.
The existing sabotage is an event outside the spec list; adding one named
spec event preserves that check. No new scenario duplicates it.

Extend G6/G11's default selection scenario. It now consumes committed
feature/b history declaring the former default, from a clean matching
checkout after selecting feature/a. Consume must not reinterpret imported
history as incoming configuration. It then selects an imported canonical
head with a different checkout, a behind checkout branch, and an absent
checkout branch with untracked operator work. Selection changes symbolic
HEAD and leaves checkout refs and files untouched. Expected heads come
from fixture git commits, branch identities from the fixture, and file
contents from host writes. Its added sabotage comments name revalidation
of the former default and imposing consume preconditions on selection.
The existing import/consume tests do not change default before consume or
select from an absent checkout branch. The second operations after a
switch are the cases most likely to expose that distinction.

Keep the new G11 held-intent scenario. A PATH git wrapper holds the real
landing update-ref after gates and intent. The CLI default-switch refusal
must answer before the fixture releases that child. It names both the live
attempt and open intent from externally read store rows. Host git proves
neither canonical branch nor symbolic HEAD moved. After releasing the
child, landing history and store rows establish that no work or intent
remains, and the same selection succeeds in the same project. Sabotage:
omit the intent identity or check it only after waiting for canonical.
The held-model attempt scenario cannot expose waiting behind a git child
or the intent identity. No timeout, retry, assertion weakening or binary
barrier replaces the hold. This is not a crash scenario.

The binding G11 row names the new shown-by clauses. The sync contract now
separates import validation, consume's clean matching checkout, and
selection from already imported canonical history. It retains the live
attempt/intent refusal and checks it before waiting and under the lock.

Test lines removed: 0. Binary lines removed in this repair: 33.

## Validation

`cargo fmt --check`, locked all-targets clippy with denied warnings and
`git diff --check` pass. The existing G3/G13 and all four G11 scenarios
compile.

The requested commands were attempted on the repaired tree:

- `cargo test -p e2e --locked g13_`
- `cargo test -p e2e --locked g03_`
- `cargo test -p e2e --locked g11_`
- `cargo test -p e2e --locked`

Each aborts in fixture setup at `pinfold on PATH`, followed by the cleanup
error `run pinfold: No such file or directory`. No scenario reaches its
assertions. This worker workspace has no pinfold executable or host
container runtime. Real host execution remains required; these are not
passing e2e results. The logs are under `/yard/proof/y82-repaired-*.log`.
No sabotage was run here. No operational configuration or frozen daemon
was changed or opened by a development binary.
