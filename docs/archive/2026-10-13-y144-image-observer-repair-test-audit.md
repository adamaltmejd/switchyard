# Y144 image consumer observer repair

The host landing gate on target
`d5292144f98be6d06dea70d7f245aa78c18f3b73`, merged ref
`77c4edf39d41ee122c4d6a245fa553a64a1f0b16`, reported 78 passing scenarios
and one failure in G2's quiescent source-adoption scenario. Both observed
runtime image IDs were null. The assertion assumed pinfold always supplied
an image ID. The store records that metadata optionally; the contract does
not require it to exist. The target and its qualification archive are now
merged into this candidate.

## Test-audit Gate

Replace the invalid image-ID comparison in the existing G2 scenario with
direct image-file observation by each affected review consumer. Each review
now reads `/image-input` through real Pi bash and the host model asserts the
fixture bytes before issuing publication. Previously bash tested the bytes
but publication was issued alongside it, so a failed command could still
publish. The implementer's existing chained command already requires those
bytes before committing the externally observed changed feature.

| Gate question | Answer |
| --- | --- |
| Guarantee and clause | G2: independent image adoption keeps definition/staging fixed, binds host Git Dockerfile/context identities, runs real image consumers and preserves old evidence while host command definitions remain unchanged |
| Named binary sabotage | Build current HEAD instead of the selected image context, or reuse the prior consumer image. The test comment names both; the later unselected context contains different bytes |
| Distinct hardest case | Adoption follows completed consumers and checks, with newer poisoned definitions/staging and a later unselected image context. Digest and source-row assertions alone cannot establish actual image bytes. This repairs the same scenario rather than adding a twin |
| Outside expected values | The fixture writes `image-v2` into the selected context; real box bash returns the bytes through Pi's host model route. Host Git supplies the independently selected blob/tree identities |

The ENV marker still selects image consumers, and the implementer's feature
observation requires its selected image. Both reviewers must independently
observe the COPY bytes before publication. The unchanged host-command digest
assertions and all historical execution/settings/check/revision snapshots
remain. No guarantee or Shown by clause is removed or weakened. Runtime
image metadata remains recorded when supplied; no synthetic ID or product
fallback is introduced.

Static Gate accepts this repair. Test diff: 18 lines removed, 17 added;
binary lines removed: 0. One invalid optional-metadata assertion is replaced
by direct consumer evidence. No suite sweep or weakening decision is needed.
Host e2e and named runtime sabotages remain post-approval gates and were not
run in this box. The independent `Y144-historical-landing-intent` sabotage
and the exact resolved-intent refusal/no-write control remain unchanged.

The imported Y143/Y146 qualification archive, earlier failed qualifications,
and current qualified G9 controls remain unchanged. All original Y144 and
parent integration obligations, including the separate packaged activation
child, remain mandatory.
