# Y-70 landing decision choice

Keep the extension to `only_the_declared_choice_is_checked_and_landed`.
It shows G2's exact choice binding and G8's explicit landing-omission
acceptance. The five-producer fixture declares a second landing gate but
omits it from the admitted graph. Exact approval requests observation of
that named omission. After the prepared landing releases ownership, the
operator accepts it and the same candidate reaches the existing held CAS
and successful landing control.

Sabotage: remove choice from the queue's landing attention payload.
The exact acceptance then refuses as stale instead of reaching CAS.
The test comment names that sabotage. Existing omission scenarios have
no selector choice, so their null identities cannot expose this failure.
The existing stale-choice approval assertions remain: the first choice
refuses after a second choice of the same candidate, with both identities
reported. No existing assertion or guarantee is removed or weakened.

The configured fixture supplies the omitted node name. Git supplies the
candidate head and the existing final content observation. Choice equality
compares the approval and landing attention, as G2 requires those outputs
to agree. The acceptance result must be true, and continuation must reach
the externally held Git CAS and land B's fixture bytes. Release is observed
through history; no sleep, retry, mock or binary barrier is added.

Both landing attention producers now carry the choice: named omissions
and eligible merged gate failures. The shared stale identity check stays
unchanged. ARCHITECTURE.md states this current payload contract and extends
the G8 evidence clause. Existing non-selector merged failure coverage is
retained.

Operator dispositions: none. Test lines removed: 1 approval-call line,
replaced to request omission observation; no assertion blocks removed.
Binary lines removed: 2 payload lines, each replaced with its choice field.

Local validation: cargo fmt --check, all-target locked clippy with warnings
denied, and cargo build --locked. Host e2e is deferred to the reviewed
landing gate as explicitly instructed; it was not run in this clone.
