# Y-109 committed-definition integration

This read starts at 96b7d72. The checkout contains the independent source
recording from Y134, consumer qualification from Y135, and ordinary and
packaged activation from Y136 and its implementation children. The earlier
f8053434 assessment and abandoned attempt 130 are not implementation evidence.
This checkpoint extends an existing observer. It adds no runtime shape,
grant, dependency, lifecycle or ownership path.

## Source trace

`operator_mcp.rs` exposes definition, staging and image commits on the
existing revision command. `jobs/admit.rs::attempt_revise` defaults each
omitted source to the current recorded source. Explicit definition adoption
re-resolves the recorded selection; independent staging and image adoption
retain the selected definitions. `jobs/mod.rs::load_attempt` validates the
exact commits, plan and committed inputs. `resolved` overlays recorded
definitions rather than the current template.

The revision transaction rechecks attempt identity and expected revision,
preserves original admission and appends revision evidence. It withdraws
active approval and supersedes candidate questions. Target, worker grants
and supported ordinary or packaged shape stay frozen. Running gate/review
source adoption is refused. Open intents refuse edits; changed future landing
requirements also refuse after a resolved intent. Landing gates consume their
recorded definitions, including candidate-stage gates selected at landing.

`store/executions.rs::consumed_definition` binds each consumer's selected
runner, settings, harness, staged paths/blobs/modes and boxed image context.
Host gates do not consume the boxed image. Reviewers also bind review policy;
packaged consumers bind their frozen package and launch capability.
`qualifies` compares actual node policy and typed producer inputs, without
relabeling the old producer. `store/checks.rs::current`, panel supplier
selection and `selected_checks_current` share this qualification. Whole
gate/panel identities bind decisions; individual checks bind their consumers.

Implementation startup and handoff read recorded execution revisions.
`jobs/capacity.rs::release` qualifies a stopped packaged implementer through
`attempts::for_execution`, not the latest attempt admission. Checked box
absence and freedom of its existing host lock precede reservation release
and reconciliation of only its exact ownership item. Missing historical
definition proof cannot qualify that reconciliation. Stop, raw unchanged
result and ineligibility remain; explicit start consumes the adopted revision.
Ordinary stopped preparation and unknown-owner containment retain their exits.

## Original acceptance mapping

| Requirement | Existing scenario and outside observation |
| --- | --- |
| Adopt one gate after completed gate/review; preserve unrelated checks and withdraw approval | G2 `quiescent_plan_cas_retains_only_exact_node_and_producer_evidence`, revision 5: host gate counters, one new gate execution, changed gate digest, retained reviewer check and withdrawn approval |
| Stale revision decision writes nothing; corrected identity succeeds | Same G2 scenario: exact expected/current revision errors, stored rows and audit snapshots, then current revision approval and landing in the same project |
| Template-only sync leaves active plan unchanged before adoption | New observations in that G2 scenario before gate adoption and revisions 7/8/9; existing G2 workflow/seat template-sync scenario remains |
| Tool/skill adoption invalidates actual consumers and changes their identities | G2 revisions 7/8: separately committed tool and skill blobs, actual host model requests, changed reviewer digests, fresh implementation supplier and typed candidate/panel bindings; unaffected host-gate definitions remain fixed |
| Independent definition, staging and image inputs | G2 revisions 5/6 retain omitted sources; revisions 7/8 adopt staging alone; revision 9 selects an older exact image context against later unselected sources and observes real COPY/ENV bytes |
| Targeted profiles and routing preserve grants and shape | Earlier revisions in the same G2 scenario retain unrelated seats, invalidate changed policy and downstream supplier bindings, and preserve skipped nodes and historical rows |
| Frozen startup and teardown handoff | G9 capacity-held implementation starts from old inputs, then its actual successor consumes adopted inputs with an operator diff; predecessor checked release precedes startup |
| Running packaged committed adoption | G8 Durable scenario independently adopts definition, staging and image; native host requests observe instructions, model and effort while package identity and grants stay frozen |
| Old stopped packaged owner after revision | G8 held-stop scenario uses real teardown/listing holds and a same-inode host lock; old execution and both questions survive adoption, checked release reconciles only ownership, and explicit start publishes with adopted instructions |
| Live safeguards outside the plan | Reuse G2's existing held automatic approval and safeguard-sync controls; no additional assertion or twin |
| No edits during intent, including future landing requirements | G3 held open and resolved intent controls preserve exact prepared input and refuse mutation; corrected pre-intent adjustments succeed |
| Immutable original revision/execution/result/check rows | G2/G8 snapshots compare historical rows through adoption, teardown and reopen; G9 observes actual old execution startup and successor bindings |

## Test-audit Gate

Disposition: keep two added assertion blocks in the existing G2 scenario.
No assertion, scenario or guarantee is removed. No whole-tree sweep is made.
Test lines removed: 0. Binary lines removed: 0.

1. G2's template-sync clause now explicitly includes staged tool/skill
   changes before adoption. The gate block observes after a committed gate
   sync, with completed checks and an active approval held behind real
   project capacity. The loop block observes after tool-only, skill-only
   and image-only template sync, before the explicit CAS revision.
2. Sabotage: silently overwrite active resolver evidence during sync.
   The test comment names this fault. Each block must fail directly at the
   post-sync plan comparison, before explicit adoption can conceal it.
3. Existing selective execution, digest and historical-row assertions run
   after adoption. They cannot detect an active rewrite that adoption later
   replaces. The existing workflow/seat sync case does not stage changed
   tool/skill bytes. These observations extend the completed-work scenario,
   where accidental reuse or invalidation matters; they add no first-run
   scenario or safeguard-sync twin.
4. The equality expectation is G2's unchanged active plan contract. The CLI
   supplies its pre-sync observation; comparison of two outputs is the
   promised guarantee. Changed definitions and bytes come from committed
   fixture inputs; host Git supplies source identities. No expected digest
   is computed from Yard's logic. Later host counters, requests and stored
   immutable rows remain the adoption controls.

## Validation boundary

This is a source integration assessment and candidate observer change, not
new host behavioral qualification. Earlier dated archives retain their
individual passes, failures and sabotage limits. In particular, the last
Y136 archive records the failed 96f0aae native run and its setup correction;
this document does not turn that run into a pass.

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`
and `git diff --check` passed. All-targets clippy compiled the changed e2e
scenario. The real host e2e gate and independent fault/restoration qualification remain after
approval. No daemon or development binary opens construction state, and no
`.yard` file is changed. Full original Y62 integration and acceptance remain
required; Y63/Y65 stay attached to Y62. This commit does not close Y62 or
claim operator approval or host acceptance for Y109.
