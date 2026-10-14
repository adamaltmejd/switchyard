# Y137 Check1458 Gate correction

Delete only the `frozen_sources` Dockerfile blob assertion used at G2
revisions 5 and 6. The selected definition/settings commits have identical
Dockerfile bytes. This assertion cannot fail under the named omitted-source
fault. Exact image/staging commits and the whole context tree can fail.
The G2 Shown by wording now names only those independent observations.
The original integration audit remains unchanged as history.

| Gate question | Remaining source block | Remaining reviewer-digest block |
| --- | --- | --- |
| Guarantee and clause | G2 independent committed-source adoption retains exact omitted staging/image commits and whole context at each gate-only/settings-only revision | G2 declared reviewer settings adoption changes every affected consumer digest |
| Sabotage | Adopt an omitted source in revision 5 or 6, then restore it before staging adoption; the existing comment names this fault | Omit agent settings from review consumer digests while launching the adopted model; the existing comment names this fault |
| Distinctness and hardest case | Later source observations miss a transient violation after completed judgments on the same candidate | Model requests and execution counts cannot prove changed digest bindings after completed reviews on the same candidate |
| Outside expectations | Fixture commits and host Git captured before admission supply exact definition, staging, image and context identities | The fixture changes the selected model; the spec requires a changed consumed identity. Stored digests are observations, with no copied Yard hashing logic |

Disposition: remove one test line, zero binary lines. Keep every other
source and reviewer-digest assertion. Revision 9's Dockerfile/image
assertions remain. No guarantee, product behavior or acceptance clause is
weakened. No new scenario, fixture mutation or `.yard` edit. This is an
evidence-integrity correction, not a suite sweep or discretionary cleanup.
Host e2e and independent runtime faults remain post-approval obligations.

Validation: `cargo fmt --check`,
`cargo clippy --all-targets --locked -- -D warnings`, `cargo build --locked`
and `git diff --check` passed. No e2e or runtime sabotage ran in this box.
