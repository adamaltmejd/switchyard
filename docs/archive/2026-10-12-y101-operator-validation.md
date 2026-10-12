# Y-101 operator validation

Y-101 landed through queue execution 1996. The candidate was
`de5c8f385cc07e9a2c9b639ae3af13337d1e4220`, based on
`a121749d5e799fac5ac4fee1ca37e274df7e9f93`. Approval 112 bound that head
and the empty proof digest. Its actual merged head is
`c826310620e4c2b3803110202d6a4ad3707f2aca`, over target
`d0935530aef8bb3a2498425c39d9349c541a9e8c`.

The operator read the complete implementation, successive repair deltas,
changed assertions and dated audit records before approving. Candidate
formatting, Clippy and build gates passed. Review 1994 passed with no
findings. Fresh public head, proof and diff matched the audited candidate.

Actual merged gates 1997, 1998 and 1999 passed formatting, Clippy and build.
Host gate 2000 ran the real full e2e suite: 75 passed, zero failed, in
551.73 seconds. Both complete packaged-audit scenarios passed, including
the valid resealed changed-asset refusal and corrected-image control.
The landing intent records the exact old and merged heads and is resolved.
The attempt ended landed, the ticket is done, and cleanup 2006 ended removed.
The operator consumed the landing with `yard sync`.

This was the existing Linux x86_64 validation profile with its isolated
FUSE overlay store. The command remained:

```sh
PATH=/home/yard/validation/podman-shifting-profile/bin:$PATH \
YARD_E2E_DURABLE_INPUTS=/home/yard/validation/switchyard-durable-inputs \
cargo test -p e2e --locked
```

No Linux arm64, operator Mac or native-overlay acceptance is claimed.
No additional operator sabotage run was performed. The measured suite time
exceeds the architecture's approximate five-minute landing budget; this
record changes neither the budget nor the scenario list.

Failed host gates 1932, 1964 and 1989 remain failed evidence. Gate 1989's
image-not-known and layer-not-known causes remain unidentified. This pass
does not establish that those runtime failures are fixed. A read-only host
observer captured native build/removal process identities and storage metadata
during gate 2000; it did not change the workload or runtime configuration.
Its sampling establishes observations, not removal causality or lock holders.
Private evidence is retained under `/home/yard/validation/` and `/tmp/`.

Y-102 still must establish selected-controller cancellation and failed-teardown
publication eligibility. Y-100 and Y-93 retain their full integration scope.
This landing does not complete those tickets or the release through Y-80.
