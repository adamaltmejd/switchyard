# Y-158 integration target merge

Merged target 986307d81228346d8b27364d06ffc77f3ea56482, which adds Y-71
named-input integration, into the Y-158 candidate. The only conflict was
the G2 guarantee-table row. It now retains both integration's own candidate
identity and ordinary agents' selected admitted input identity observations.
G13 input containment and the reduced test policy remain unchanged.

The launch and admission changes merged without conflicts. Reviewed their
shared boundaries: integration's candidate snapshot and bundle mounts are
added separately from the selected trusted input mount. Selected skills and
references still stage from admitted blobs, and both ordinary and packaged
agents keep their shared selected-input prompt. Integration does not replace
that staging or introduce a writable trusted mount. Reviewer candidate mounts
and selected input staging also remain separate.

No manual product or test changes were needed. The target's integration
tests and existing audit records are retained; no scenario or coverage was
added by the resolution. No .yard files changed.

Validation: cargo fmt --check and cargo clippy --all-targets --locked --
-D warnings. No e2e ran inside the candidate box. Real host qualification
remains the landing gate.
