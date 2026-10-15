# Y-71 reduced target merge

Merge target: 9e4788b754d2c47d43704c5f9de67630b69144f0. The operator authorized
the reduced test surface and updated AGENTS/test-audit policy, and reported
all 45 retained real-host scenarios passed. This merge preserves that policy
and every target test file exactly, except for registration of Y-71's two
existing integration scenarios. No removed permutation is restored.

The G7 conflict takes main's complete g07_usage.rs unchanged, including its
already-qualified usage/identity projection. It has the same selected fields
as the prior Y-71 repair. No new assertion or observer is introduced. Earlier
gate3497 remains failed; its retained identity and evidence location remain in
2026-10-15-y71-pi-reopen-test-audit.md. Archive records remain dated history.

The architecture conflict takes main's concise Shown by inventory, adding
only the existing G2/G9 integration observations: named-input C and exact
approval, retained partial work through restart, source write denial, actual
canonical access denial with own-write control, and conflict repair before
selection. Y-71's implemented contract outside that inventory is retained.
Product controls are unchanged by this merge.

Test-audit disposition: retain main's 45 scenarios and Y-71's two existing
integration scenarios, without new machines, boxes or model turns. Their
material risks remain independent input approval leaking to C, canonical
exposure in integration startup, lost partial work and selection before
conflict repair. Existing fixture expectations and precise sabotages remain
unchanged in g09_integration.rs and its prior audit records. This merge does
not change that file or any production source relative to d64a10a.

Worker validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings and cargo build --locked. Native validation is the required full
prepared-candidate host gate after approval. No redundant preapproval focused
or full native runs are added or claimed.
