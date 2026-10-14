# Y-153 published proof expectation repair

The operator's first required host scenario failed 0/1 at g02:114, after
27.23 seconds of suite time and 41.30097 seconds wall time. The raw log was
retained by the operator. Earlier eligibility, runtime, base and source
assertions reached successfully. Assertions after the failure remain
unqualified. No full suite ran.

The scenario incorrectly expected the first implementer's input proof to be
its published candidate proof. That input proof is null. The original immutable
result publishes the candidate proof at `outputs.result.value.proof`.
The gate correctly recorded that published digest.

The same G2 block now reads the expected proof from the original producer's
stored result. Its typed supplier execution must name that producer; its
semantic candidate proof must equal the producer's published proof. The host
snapshot at that digest must still contain the fixture's literal `first` bytes.
MCP after adoption must preserve the producer's original result as well as the
gate's original evidence/result. No history is rewritten and no hashing logic
is copied. Neither the digest expectation nor the snapshot path comes from
gate evidence.

Test-audit disposition:

1. G2's existing typed-supplier and immutable launch/result clause requires
   agreement between the gate and the original candidate publication.
2. Named sabotage: bind a gate to the producer's input proof instead of its
   published candidate proof. The test comment names this distinction.
3. This first publication has null input proof and nonempty published proof;
   conflating them is exposed in the existing scenario. This repairs its
   expectation without adding a scenario or weakening an assertion.
4. Equality across the two original public records is the named guarantee.
   The host snapshot's expected bytes remain the external fixture literal.

The approved trust boundary, prior exact-base and FIFO corrections, deadlines
and concurrency remain unchanged. No binary lines are changed. Existing proof
assertions are retained with their expected value corrected to the publication.
The operator must rerun this focused scenario before approval and the required
full landing gate. No host suite runs in this worker box.

Repair diff: 6 test lines replaced, 0 binary lines removed. No assertion deleted.
Boxed `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `cargo build --locked` passed. These checks do not qualify host assertions.
