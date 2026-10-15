# Y-70 selected operator failure policy

Review1711 found three gaps in the selected graph: admission rejected
operator failure policy on reviewers and native gates, landing admission
rejected its failed outcome, and the selected review path skipped the
operator decision item. The repair admits the policy only on actual
verification roles. Writable candidate producers still reject it, and
the picker still rejects it. Native gate definitions, access, inputs and
outcomes remain exact, including the ordinary failed landing outcome for
operator handling. Selected review failures use the same candidate item
producer as ordinary failures, after required candidate gates complete.
Automatic repairs and review limits keep their existing paths.

Read together: graph admission and normalization; selected source review
and candidate gates; current check and immutable evidence lookup; exact
candidate answers; selected producer repair and frozen feedback; merged
child eligibility, prepared failure evidence, decision continuation and
landing return. Candidate items bind choice, candidate, proof, ticket,
plan and configuration. Decisions revalidate that item and the exact
eligible evidence. Landing decisions additionally prove the original
preparation, approval, child, current target and prepared ref. Existing
decision, queue and owner lifecycles remain unchanged.

Disposition: keep one sequential real-host G2/G8 qualification in the
selector scenario file. Two independent differently configured producers
publish disagreeing Git content. Their same frozen reviewer profile
passes A and blocks B. A read-only picker chooses B. The host first asks
to repair B's failed candidate gate, then accepts its new gate failure.
It repairs B's own blocking review, then accepts the new blocking review
after the gate passes. Required checks and exact approval then reach a
real failed merged gate. Repair returns to B with the original prepared
failure evidence frozen in its inputs. B republishes and receives a fresh
own review, choice and checks; exact approval reaches another real merged
failure, which the operator accepts. Only B's Git content lands.

The test comment names the sabotage: reject policy on admitted verification,
skip selected review attention, omit the failed landing outcome, copy A's
review, treat repair as acceptance, or rewrite failed evidence. Existing
ordinary selected-failure tests lack candidate selection. The original
five-model scenario uses automatic repair and review-limit approval, which
cannot show the operator failure-policy contract. Both remain unchanged.
This scenario crosses repair and acceptance in each affected role on one
project and graph; it adds no framework or qualification lifecycle.

The observer is real Git, CLI/MCP, the store and the scripted host model.
Expected versions and source bytes come from the producer scripts. Git in
B's retained clone supplies the head; canonical Git supplies the actual
prepared ref and final landed bytes. The graph requires one own review
and one native candidate gate. The preceding gate check refuses as the
answer to B's blocked-review item, with selected_result and both identities
and no audit write; the corrected same-project decisions succeed. Fresh
passing check heads match the new Git source, with no inherited acceptance.
Original failed checks, execution results and choice rows remain identical
through landing. Those comparisons prove exact identity and immutable
history, rather than deriving expected verdicts from Yard.

No existing assertion or guarantee was removed. Test lines removed: 0.
Binary lines removed: the blanket reviewer-policy rejection, native-policy
equality restriction and the duplicate ordinary candidate-item producer.
The same producer now serves ordinary and chosen verification failures.

Validation: fmt check, all-target locked clippy with warnings denied and
locked build passed. Native/e2e was not run in this
box, as instructed; host qualification remains the operator's landing
gate. No .yard project configuration changed.
