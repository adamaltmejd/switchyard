# Classification route edited-worker hold repair

The operator's current 637be9 native route run, integrated with observer
05bffabd5a46cd2b6f42faba4dee589a28d0d3ae, failed 0/1 in 333.71 seconds.
Child 101 waited for an obsolete model hold at model.rs:335. Its 300-second
expiry is failure, not product evidence. The operator's live reading showed
execution 13 with reason edit, model fake-model and resumed NULL. Its full
prompt delivered New brief and the candidate diff. The previous fixer used
fixer-model, so the fresh original-profile session correctly omitted resumed
session delta wording. Executions 13/14/15 had already ended and released,
with classification missing_report attention, while the test waited.
Evidence remains at
`/home/yard/validation/y73-637be9-integrated-routes-20261015-evidence`.

## Test-audit disposition

Adapt the existing G2/G8
`delayed_candidate_acceptance_names_the_changed_identity_and_writes_nothing`
in `g02_judgments_bind_identity.rs`. Remove only the prompt-dependent hold
condition. Immediately before the ticket edit, arm one shared optional FIFO
path. The next worker opening consumes it once, observes the original model
and actual New brief, writes the existing-style request marker and holds the
same reply. History selects that attempt's exact edit execution; the FIFO
observer fails if it ends or raises attention before the request marker.
Only then does the test wait for the latch and observe model fake-model,
reason edit and resumed NULL in that execution row.

Retain revision-stale/no audit writes while the edited worker is held, then
release into the intentional missing-publication infrastructure error. Retain
operator classifier retry, the valid candidate question, head-stale/no writes,
allowed repair and authorized landing. Original/fixer/acceptance routes,
undeclared and stale publications, truthful failures and group/source controls
are unchanged. The existing positive controls stay in the same project.

This protects stale acceptance and correct selected-profile inputs without
requiring stale session reuse. The CLI edit supplies the expected brief;
fixture-selected models and externally read execution rows supply session
expectations. Sabotage comments name resumed fixer state, omitted brief,
stale decisions and false classification. The representative route remains
in its existing machine. No machine, historical build, box, model turn, retry,
sleep, endpoint or binary hook is added. Test lines: one removed, 40 added.
Binary lines removed or added: zero. No Shown by or product contract change
is needed. Both authorized test-phase cuts remain intact.

## Qualification and retained evidence

Candidate fmt, Clippy with all targets/locked and warnings denied, build with
locked dependencies, and diff whitespace checks pass. Source review checked
one-shot consumption, marker-before-latch ordering, exact execution selection
and unchanged continuation/assertion phases. Review 2001 predates this repair;
current candidate review and real host qualification remain operator gates.
No host e2e is run here, and no installed runtime, daemon, binary, model,
reviewer, observer or .yard configuration is changed.

Production remains identical to e3b7598 with all measured repairs. The operator
reports the complete grouped case passed 1/1 in 106.30 seconds with corrected
521 observer, including the original stack-crash point and later five-writer
FIFO. That focused result is not a full suite pass. Original failed results
remain retained; this fixture repair makes no new production fix or native
pass claim. Normal final prepared host qualification includes the two
previous phase cuts; no extra full run or restoration ticket is introduced.
