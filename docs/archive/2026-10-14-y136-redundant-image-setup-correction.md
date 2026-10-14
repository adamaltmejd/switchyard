# Y-136 redundant image setup correction

Native gate2944 on 96f0aae passed 78 scenarios and failed one in 522.42
seconds. The corrected cleanup boundary passed. The actual 559f730 daemon
built in 40.84 seconds. The historical setup then failed at the image
helper's unconditional Git commit. The operator retains the full raw log
and terminal identity at
`/home/yard/validation/y111-operator-evidence-20261013/approved-y136-96f0aa-gate-capture/`.

The preceding corrected-image control already committed the isolated
Dockerfile and image-input. The historical setup commits its actual runner
bytes and syncs them before calling the image helper. That helper stages
only the unchanged Dockerfile and image-input. Git correctly refuses the
empty commit. The later capability loop repeats that same redundant call
after committing and syncing each selected runner.

Remove those two calls. Keep the existing isolated Dockerfile, machine
label and image-input. Keep each runner commit and sync so the committed
whole image context contains the selected runtime. Do not change the image
helper or ignore Git failures.

## Test-audit Gate

1. G8's old six-field startup, unsupported-effort refusal and supported
   omission publication clauses remain. Its independent recorded-metadata
   and exact-package fallback clauses remain. The edit repairs setup for
   these existing controls; it changes no assertion.
2. The existing comments name the binary faults: inject unsupported effort
   into old startup, relabel the old package, require current hashes, remove
   the adoption guard, or remove either capability arm. Existing boxed asset,
   provider request, refusal and immutable-row assertions retain those
   observations after this repair.
3. Current-package admission cannot detect old six-field incompatibility.
   Same-package new admission masks a missing capability arm. These existing
   real-binary upgrade controls hold actual controller requests across
   stop, checked release and explicit adoption. The repair adds no scenario
   or duplicate observer.
4. Expected runner bytes come from actual archived source and the committed
   fixture variant. Source pins, the old non-null effort fixture and committed
   max/omitted settings remain. Host model requests, boxed asset reads, CLI
   identities and read-only store snapshots remain the outside observers.

Disposition: delete two redundant setup calls. Zero assertion blocks and
zero binary lines are removed. Preserve the execution-FK check snapshot,
attempt-bound cleanup waits, early-terminal observations, actual binaries,
source pins and every existing publication and no-write assertion. No
guarantee clause changes or loses coverage.

Validation: cargo fmt --check, cargo clippy --all-targets --locked --
-D warnings, and cargo build --locked. Host e2e and independent direct
fault/restoration qualification remain required after approval. This failed
run reached only the first historical build; it cannot measure the complete
three-build suite budget. Earlier dated evidence remains unchanged.
