# Y-115 pinned Pi parser evidence

The amended ticket supplies operator source verification for the carried
Pi 1.1.0 artifact. pinfold artifacts names
/home/yard/.cache/pinfold/artifacts/pi/1.1.0/linux-x64/pi. Its executable
pi/pi SHA256 is
5d527eae57d99f80f9397a84d0228528a92b3595b2c5f3d969e45b02ea72a6bc.
The embedded parseChunkUsage reads
rawUsage.prompt_tokens_details?.cache_write_tokens || 0 and subtracts
cache reads and writes from the prompt total. It does not read
cache_creation_tokens. The operator retains parser excerpts and identity in
/home/yard/validation/y111-operator-evidence-20261013/operator-y115-pinfold-pi110-artifact-parser-evidence.json.
This is supplied evidence, not a worker inspection of that private file.
Older Pi 0.87.1 is not the carried pin.

Candidate 67ee927 used the unqualified cache_creation_tokens key. Its
unchanged total equality could pass with input 800 plus reads 200 and
writes 0. Its write-loss mutant could not establish the write leg. The
original test-audit archive is restored byte for byte as committed in
67ee927, preserving that history. The earlier repair archive's account of
editing its spelling describes db87a4c; that historical edit is now undone.

The code correction in db87a4c uses cache_write_tokens. Optional details
remain limited to round 1 of the existing interrupted/resumed G7 scenario.
Every assertion, unrelated response and explicit zero remains unchanged.
There is no production, spec, runtime, configuration or service change.

Native qualification has not been run by this worker. The earlier different
fixture's evidence does not qualify this candidate. The operator owns real
boxed Pi unchanged, separate read-loss and write-loss, restored control and
full host gate runs, only when no other host gate runs. The existing
second["usage"] equality must fail at input 800 versus 1000 for read loss
and 900 versus 1000 for write loss. Record each first failure and whether
reopen/zero observations were reached; restore exact source between mutants.
The unchanged reopen equality and later resumed explicit-zero equality
remain required observations. A green unchanged control alone is insufficient.

Boxed cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked and git diff --check passed again. Candidate review
must judge this corrected head; no host qualification is claimed.
