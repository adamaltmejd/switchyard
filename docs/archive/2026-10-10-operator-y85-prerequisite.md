# Y-85 admission-provenance prerequisite — 2026-10-10

Y-85 attempt 62 started from operator evidence commit
`2ca89e43cc752dc702cbe417e710c8670c4f1186`. Implementation execution 1304
stopped unchanged after proposing admission provenance, atomic consumption
and a scope edit. It created no candidate, source change or commit. No
candidate check or host pass is claimed for that attempt.

The operator read all three proposal bodies and checked their mechanisms
against the current source. `store/attempts.rs` inserts workflow,
implementer, base and target but records no immutable resolved admission
plan or source/configuration identity. `supervise.rs` can move candidate
base when a repair incorporates its handed target. Implementation and
review use that base for `AgentEnv::load`; image construction clones
`Loaded.head`. The prerequisite serves the already agreed new G6 admission
identity guarantee. These observations are source evidence, not claimed
reproduced CLI defects.

One new ticket, Y-87, records truthful immutable admission provenance.
Its observable contract includes selected definitions and revision, real
configuration/source/Dockerfile/build-context input identities, persistence
through repair and restart, and explicitly unknown legacy evidence. Its
tests use fixture definitions and host Git identities. Existing historical
binary scenarios remain the migration observers. No secret values or
fabricated historical inputs are admitted. Input identity is distinct from
an actual observed runtime image or result identity.

Y-87 explicitly preserves current template lookup and invalidation. It
does not claim that admission evidence freezes execution or permits result
reuse. Y-85 remains the complete atomic consumer replacement, including
live safeguards, exact result identity, changed G2/G8 assertions and binding
specification. Its original body is retained verbatim, followed by the
prerequisite decision and source evidence. Trusted staging must keep the
admitted source identity after repair changes the candidate base. A mutable
image tag cannot stand for an observed original runtime result.

Proposal 153 was rejected because its replacement body reduced Y-85 to
split bookkeeping. Proposal 151's scope was used for Y-87, but its payload
was rejected: it omitted `parked`, and proposal acceptance defaults that
field to false. The CLI has no parked override for proposal acceptance.
The user requires every new ticket to be created parked. Y-87 was therefore
filed with `yard ticket new --parked`. Proposal 152 duplicated the complete
atomic consumer work retained in Y-85 and also omitted parked; it was
rejected. Each rejection records its condition for re-filing.

Supported commands parked Y-85, rejected the three proposals, created Y-87
parked, linked Y-85 to it, abandoned the unchanged attempt, and edited Y-85
at expected revision 1 to revision 2. Watch was the next command after each
decision. The operator read back both ticket bodies and compared them with
the filed text. The open dependency graph is acyclic.

The sequence is Y-84 (landed), Y-87, Y-85, Y-86, then Y-59's full original
integration scope. Y-60 and Y-66 still depend on Y-59. Y-85 was not completed
by the stopped attempt. Y-86, Y-59 and deferred Y-81 remain parked. The
operator will start Y-87 after committing and importing this evidence.

No runtime, binding specification or assertion changed in this operator
decision. The useful redesign release, platform validation, cleanup and
release artifacts remain incomplete. No push, publication, installation,
daemon replacement or model/reviewer change was performed.
