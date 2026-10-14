# Y-149 integrity repair

Review1518 P2 confirmed that Claude byte evidence included shadowed sources.
The repair extracts the existing top-level directory selector from staging.
Staging and byte evidence now call that selector with one Claude root order.
Broader admitted source blobs remain provenance. Evidence capture, unknown
identities, checked teardown and disabled new reuse retain their Y-149 scope.

The G2 pinfold observer previously returned its pipeline reader's status.
A FIFO now lets the real pinfold child run outside a pipeline. The reader
records each line before forwarding it. The wrapper waits for the reader and
exits with the saved real child status. No shell-plumbing scenario was added.

The focused test-audit gate extends the existing real Claude scenario:

- Row: G2's effective launch byte evidence clause, alongside the existing
  G8/G12 fixture. No guarantee or assertion block was removed or weakened.
- Sabotage: enumerate all admitted files as effective byte evidence, or
  shadow only matching relative paths instead of whole top-level entries.
- Hard case: conflicting skill bytes plus a lower-only file under the same
  top-level name, while an unshadowed shared skill remains the allowed control.
  Existing G2 adoption exercises Pi, which does not perform Claude's merge.
- Observer and expected values: the real review box reads effective bytes;
  fixture literals supply their expected contents. Host SHA-256 of fixture
  files supplies the recorded byte hashes. Host Git supplies the shadowed
  provenance blobs. Stored effective paths exclude that entire lower subtree.

Diff lines replaced: 4 in tests and 13 in binary source. No operator list,
assertion deletion or whole-suite sweep applies to this focused repair.

Validation: fmt, clippy with warnings denied and locked build passed. A POSIX
probe extracted the actual observer script and confirmed child exit 7 and
exit 0 are preserved, with equal recorded and forwarded bytes. Host e2e and
fresh review remain pending. The operator reported the unchanged pre-repair
G2 scenario passed 1/0 in 83.40 seconds, 100.09 seconds wall time, on an isolated
candidate. Those retained results are not validation of this repair; no full
landing gate had been submitted.
