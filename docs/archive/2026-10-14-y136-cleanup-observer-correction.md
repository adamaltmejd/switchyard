# Y-136 cleanup observer correction

The host landing gate on merged ref
1727a14106e2c7d95b9a488651ffc3d9f8d84cfa against target
c1c81d641a3758aed1e873575b6921c872b8f7df passed 78 scenarios and failed
the existing G8 missing-assets/corrected-publication scenario. The observer
waited for a ticket-bound cleanup terminal event. The supplied history instead
shows attempt 6, execution 15 ending with kind cleanup and outcome removed
at sequence 120, then releasing at sequence 121. Both terminal events have
null ticket. Cleanup completed; the observer missed it.

Bind the three new cleanup waits to their actual attempt identities. Keep the
execution.ended and cleanup-kind requirements. No binary, timeout, retry,
concurrency or publication behavior changes. Current ARCHITECTURE promises
remain accurate and unchanged. Earlier dated evidence remains untouched.

## Test-audit Gate

1. G8's native packaged publication and checked teardown clauses are the
   promise. These waits preserve completed cleanup between the existing
   corrected, older-controller and capability controls.
2. Sabotage: omit the named attempt's cleanup completion. A completion for
   another attempt must not satisfy the observer. The test comment names it.
3. These are sequencing observations within one existing scenario, not new
   guarantee scenarios. Earlier cleanup waits concern different attempts and
   cannot establish completion for these controls. The hardest boundary is
   the same project moving between actual preserved admissions and daemons.
4. Status history is the outside observer. The expected identity comes from
   the actual attempt exposed by the CLI; comparing it to history establishes
   the promised identity agreement. The supplied host trace confirms cleanup
   terminal events use that identity, independently of the test predicate.

Disposition: replace three invalid ticket predicates with attempt predicates;
retain every publication, refusal, source, ownership and early-terminal
observer. Three test lines are replaced; zero binary lines are removed.
No guarantee clause is removed or weakened.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. Host suite rerun and independent capability faults/restoration
remain post-approval. Neither capability branch is qualified by this repair
or by the failed host run, which did not reach those later controls.
