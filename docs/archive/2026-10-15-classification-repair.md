# Y73 host failure repair

The operator ran two focused host cases against 530fe30. Neither passed:
the command case failed before launch because Pinfold mounts directory sources;
the agent case failed because Pi's schema enum intercepted the deliberately
undeclared publication before Yard's shared validator could return its token.
These failures remain retained host evidence, not successful coverage.

Command input now occupies one dedicated read-only directory containing only
the frozen JSON. The fixture reads that file through the directory mount.
No writable execution state is mounted. The agent schema accepts an outcome
string; both runners still use the shared declared-outcome and identity
validator. The prompt retains the exact declared names.

Review 1986 also identified a native landing recovery deadlock. Parentless
classifier retries now fence native queue scheduling while running or reserved,
including a transactional recheck before starting a landing. A landing that
already selected an active classifier waits for that exact execution's end and
checked release through project execution events. It neither launches another
classifier nor calls an unfinished publication missing or invalid.

## Test-audit disposition

G2/G8's existing hard command retry, budget, edit and truthful-failure assertions
are retained. Only the command fixture's input path changes. Its expected values
still come from the declared bound, fixture commands and externally read store.
The original/fixer/acceptance route case retains every invalid, stale and valid
publication control, missing-publication assertion and operator decision. The
daemon validator must produce the asserted reason tokens; no assertion depends
on Pi error prose. Bypassing daemon validation or storing continuation as pass
remains the named sabotage. No fixture machine, build, box or model turn is
added, and no assertion is removed. One fixture command line and twelve binary
lines are replaced; zero assertion lines are removed.

Native retry recovery remains outside this focused automated fixture selection;
the production scheduling and ownership contract is unchanged. The operator's
Y132 grouped-finding result and current review remain pending. This repair does
not claim a whole-native pass. Host e2e execution remains operator-owned; only
candidate formatting, Clippy and build checks run in this workspace.
