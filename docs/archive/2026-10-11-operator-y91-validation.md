# Y-91 operator validation - 2026-10-11

Actual combined candidate landed at e65730bc84ab99974e65966b32d3c181751524a3.
Fresh full host validation and actual landing each passed all 67 scenarios.

Y-91 implements the report-publication slice of original Y-64. Durable
packaging and runtime remain Y-92/Y-93 work; full Y-64 integration follows.
The original admission depends on actually landed Y-60 and Y-66.

## Publication contract and source audit

The operator read the full original 747-line candidate diff0a2b1838fc4c200c130a1db84bf75cd2bafb93bf,
including complete protected grant/spec rows and its 51-line test audit.
A nullable execution report uses the existing lifecycle and daemon-only
transaction. Audit is a capability for read-only implementers, not another
job kind. Exactly one typed publication records declared outcome and report;
summary/findings bounds and unknown fields are validated. The daemon binds
execution/node/plan/ticket/inputs/base/head/proof. Worker-supplied identities
are not admitted. Missing publication, wrong registration and infrastructure
failure produce no eligible report. Held publication stays ineligible until
checked teardown and released reservation. No transcript/exit infers a report.

Initial focused host36919 passed2/25.28s; full71919 passed64/301.23s.
Published review1575/check828 failed P1: report.published was absent from the
closed Store list; P2: an edit before publication hit missing_report instead
of continuing the edited audit. Old green tests did not clear those findings.

Repair78ffa71667443acf12008f9ad2c7fdfe14fb1515 aligned the event contract
and G7 oracle and exempted pending edits before missing_report. The new
held-before-publication edit scenario differs from held-after-publication
and missing-without-edit controls. Original assertions and harness/teardown
guards remained. The complete repair delta and protected row were read.
Focused60168 passed3/25.54s; full42502 passed65/284.99s.

Review1584/check836 passed with P2 legacy unbound continuation and P3 prompt
alignment/event coverage findings. The operator confirmed strict admission
publication versus unconditional missing_report and rejected for recovery
integrity. These were not waived by isolated greens.

Repaira2316d222d4e0d5926d9f697d0fdd3f4f115bec7 refuses an unbound legacy
read-only start before another execution or attention resolution; scheduler
readiness prevents an edit from continuing it. Abandon then fresh admitted
start is the supported recovery, with historical bindings/results unknown
and G5 owners retained. The planner's body-edit instruction was restored.
The successful publication block gained an exact event-target observation.
A distinct real pre-target stopped-planner upgrade proves refusal and the
same-project fresh admission control. Complete delta, audit and protected
G8 row were read; all original assertions retained. Focused6619 passed
4/27.24s; full58462 passed66/298.87s. Exact head and clean checkout were
verified after each full run. Those results validate only that isolated head.

## Actual landed-target integration

Y-94 landed da5e3ccb99f81f59a8aecfa464279eb95300cf99, followed by operator
records40f49060c6cdff63594455d6e4ae78531f207fe7 and
480868503a5999dad7c5f35b6e7e1888eca86092. Its G1 selected-verification
planner returned bare ready text and required explicit report publication
when combined. A full scope-preserving Y-91 revision2 carry named this.

Edit1608 returned unchanged a231 because its box had no target.bundle or
remote. The operator read the terminal worker evidence and confirmed the
frozen CLI has no target staging command; source stages the bundle only for
queue target-conflict repair. Host git merge-tree showed an ARCHITECTURE
conflict. Review1613/check848 failed P0 missing combined Y-94 integration;
ordinary review repair likewise lacked target staging. No combined pass or
approval was claimed for unchanged a231.

The operator created a public incremental Git bundle of committed HEAD4808685,
requiring existing ancestorfe6543410ef1ee336d18fcd8f612a0bb5382a4e6.
Its24364 bytes verified in the isolated validation clone. SHA256:
ad0842a83aa4a743b960fcf2434072f4cfcfa2032155394606ecda9653dc455c.
The complete revision2 body was retained in revision3, with this exact source
transport appended through supported ticket edit. No secret, runtime state,
manual store/clone/box mutation or daemon replacement was involved.

Combinede65730bc84ab99974e65966b32d3c181751524a3 is a two-parent merge of
auditeda231 and actualtarget4808685. Ancestry was checked through host git.
Y-94 plan/queue are byte-identical to landed target; audited Y-91 report,
MCP, settlement, store and audit-test files remain byte-identical toa231.
Production merge changes and complete new78-line audit were read. G1 changes
only bare ready to planned(request), retaining every omission assertion.
G8 retains the machine-specific real-image setup and all old assertions.
Protected publication/legacy/event/grants hunks and complete combined G8
row were read; G1/G3 rows are byte-identical to target. Both implemented
contracts and all controls remain. Source/test-audit Gate passed.

## Fresh combined host evidence

Report-focused15455 endedexit0:4/0/21.44s. Omission-focused87185 endedexit0:
1/0/38.38s. Exact detachede657 head and clean checkout were verified after
both. Logs: /tmp/switchyard-y91-e657-reports-host.log and
/tmp/switchyard-y91-e657-omission-host.log. Full67 host61640 endedexit0:67/0/320.85s at
/tmp/switchyard-y91-e657-host.log. Fresh published review1624 round3, check860, passes with no findings.

Protected attention192 names exact e657/revision3 and checks849/850/851/860.
All are current passing checks; approval95 authorizes the protected changes.
Fresh supported CLI diff matches the frozen-base git diff with one extra
trailing newline; the audited actual-target diff remains byte-identical.
All isolated host runs were terminal before approval. The next command
was watch26475. Actual landing1642 passed fmt1643, clippy1644 and build1646;
host gate1647 passed 67/0/325.40s. Approval was followed by actual landing.

## Actual landing, cleanup and consume

Supported history records host1647 ending PASS at5697, intent5698,
landing1642 ending landed5699, and landing.recorded5700. The recorded
candidate and merged head are both e65730bc84ab99974e65966b32d3c181751524a3;
the old target is480868503a5999dad7c5f35b6e7e1888eca86092. Ticket DONE5701,
attempt ended5702, cleanup1648 started5703 and ended removed5704.
Fresh attempt show independently confirms the ended landed attempt, resolved
intent, all four actual landing gates passed and cleanup removed.

Supported yard sync consumed the exact recorded e657 head into the clean
checkout. The next command was watch30880. No isolated host suite overlapped
the actual landing host gate. This new operator record is imported only
after the landing and consume; it does not replace runtime evidence.

## Remaining agreed scope

Y-92 packaging, Y-93 runner integration and original Y-64 full integration
remain pending. Y-97 must integrate the landed report contract, including
its two new planners and attention migration order. No isolated Y-97 pass
validates that combined contract. The complete redesign/release scope
through Y-80 remains required; Y-81 is deferred and parked. Required release
cleanup, test audit, platform checks, artifacts and operator handoff are not
claimed complete. Models, reviewer and the frozen daemon were unchanged.
