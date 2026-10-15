# Y-156 review correction test gate, 2026-10-15

Keep the integrated G2/G9/G13 assertion block admitted in
`2026-10-15-y156-test-audit.md`. Correct its judgment query to read head and
attempt from the joined execution. Store migrations removed those duplicated
columns from the check table. The other queries in this scenario use current
execution and audit columns.

The four gate answers remain: stale publication judgment is the material G2
failure; carrying old judgment to a new publication is the named sabotage;
the existing integrated scenario is the survivor with no added fixture or
model cost; expected heads and producer identities come from canonical Git
and the separately observed repair executions. No assertions are removed or
weakened. Test lines removed: 1 (replaced). Binary lines removed: 0.

Formatting and all-target Clippy are the candidate checks. Host e2e remains
the landing gate after approval; it was not run inside the candidate box.
