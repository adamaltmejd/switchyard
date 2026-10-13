# Y-115 native cache-write correction

Correctness review blocked candidate 67ee927. Its fixture sent
prompt_tokens_details.cache_creation_tokens. The pinned Pi 1.1.0 reads
cache_write_tokens instead. The rejected fixture therefore could not expose
omitting cache writes: Pi would report input 800, read 200 and write 0.
The total equality could pass even with Yard's cache-write sum removed.
The earlier archive's write sabotage claim did not apply to that candidate.

The repair changes that single provider key to cache_write_tokens. The
fixture total remains 1000, with read 200 and write 100. Under the parser
identified by review, Pi's reported input is 700. These are expected values,
not observations from a host run of the repaired candidate. At the reviewer's
explicit request, the earlier archive's provider key spelling is corrected;
this file records the rejected spelling and its coverage limitation.

All assertions, the existing scenario, explicit zero, unrelated responses,
production and ARCHITECTURE.md remain unchanged. The test-audit Gate still
uses G7's interrupted resumed Pi observation and the existing second["usage"]
equality against host total 1000. Omitting reads should produce 800; omitting
writes should produce 900. The unchanged reopen equality and third["usage"]
explicit-zero equality remain the later hardest observations. No assertion
or production line is removed. The fixture supplies expected token values;
the spec supplies provenance, completeness and null-cost expectations.

The cfc29ca control and mutation evidence is carry-in only. It does not
qualify either candidate. Host runs must separately exercise both mutants,
restore exact production source, and run the unchanged control and full
landing gate. Record the first failed assertion and unreached reopen/zero
limits. The worker has not run e2e or mutations, per the task instruction.

Repair checks passed: cargo fmt --check,
cargo clippy --all-targets --locked -- -D warnings,
cargo build --locked and git diff --check.
