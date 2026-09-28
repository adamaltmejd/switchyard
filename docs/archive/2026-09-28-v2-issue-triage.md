# v2 issue triage

On 2026-09-28 all 79 issues of the old implementation's repository
(adamaltmejd/switchyard, TypeScript on Bun) were read against
ARCHITECTURE.md, before that repository is removed at the switchover.
Each issue was sorted into one of these:

- **Covered:** the spec or v3 code handles it.
- **Retired:** it belonged to a v2 mechanism v3 does not have, or the spec's
  Open questions rule it out.
- **Candidate:** it still applies.

## Filed

The candidates checked against v3 code became tickets:

- **Y-13, from #91.** An accepted edit proposal is applied at the ticket's
  current revision, so an operator edit made in between is lost.
- **Y-14, from #101.** Git output past the 16 MiB cap is dropped silently.
- **Y-15, from #86.** A pruned project image stays cached until canonical
  moves or the daemon restarts.
- **Y-16, from #100.** In a linked worktree, `init` registers a second,
  empty project.
- **Y-18, from #122.** Gates receive `YARD_BASE`. Gate checkouts are full
  clones, so the changed-paths file #122 asked for is not needed.
- **Y-19, from #123 and #77.** A planning ticket for a proof directory. The
  operator named the pinfold builder's two requests as wanted.

## Not filed

Each of these has the condition that would admit it.

- **#73 and #98: overriding `parked`, `priority` or `workflow` on
  `proposal accept`.** When the operator hits it on v3.
- **#110: gates chosen per workflow.** It changes the gate digest and G2.
  Admit it when a project's gate cost is measured to matter.
- **#59, #62, #71, #105 and #118: bounding `attempt tail`.** v2 transcripts
  reached 10 to 12 MB. Admit it when an operator agent is hit by it.
- **#84: a brief line for a self-contradicting ticket.** It serves no
  guarantee.
- **#115: an operator inside a VM or box.** Not a supported surface.

## All issues

| # | Verdict | Reason |
|---|---|---|
| 48 | retired | Bun fetch idle timeout; `preflight` is gone |
| 49 | retired | Generations ordered by timestamp; v3 orders by row id |
| 50 | retired | v2 skill wording |
| 51 | retired | Branch-name retargeting is gone |
| 52 | retired | `lane tail` id parsing |
| 53 | covered | `stopped:timeout` offers nudge |
| 54 | retired | autoreview `pass_reports`; replaced by `yard_publish_review` |
| 55 | covered | The clean check ignores ignored files; gates use fresh checkouts |
| 56 | covered | `stopped` and `red` always offer start, nudge, abandon |
| 57 | retired | The `.yard` refusal returns to the implementer |
| 58 | covered | Ignored output never counts (G13) |
| 59 | not filed | `attempt tail` size |
| 60 | retired | v2 skill wording |
| 61 | retired | v2 refusal wording |
| 62 | not filed | `attempt tail` size |
| 63 | covered | The Pi stream has no line cap |
| 64 | retired | Claude subagents; Pi only |
| 65 | covered | The image build log is carried into the `red` detail |
| 66 | retired | No gate artifacts |
| 67 | covered | The image comes from the target head |
| 68 | retired | autoreview log parsing |
| 69 | covered | Execution rows carry start and end |
| 70 | covered | `status` lists running executions |
| 71 | not filed | Progress notes are whole up to 4000 bytes |
| 72 | retired | The hand-rolled parser and `--scope` are gone |
| 73 | not filed | Proposal accept overrides |
| 74 | covered | Park bumps no revision |
| 75 | covered | Approval text is recorded |
| 76 | retired | No gate artifacts |
| 77 | Y-19 | Attachments, planned with the proof directory |
| 78 | retired | Declined; `depends_on` sequences work |
| 79 | covered | Configuration is read at each execution start |
| 80 | covered | G2 |
| 81 | retired | The verdict is derived from findings |
| 82 | retired | v2 verb JSON shapes |
| 83 | retired | Proposal sibling references (G15) |
| 84 | not filed | Brief wording |
| 85 | covered | `stopped:failed` offers start |
| 86 | Y-15 | Pruned image cached |
| 87 | retired | v2 state tree |
| 88 | retired | The on-demand daemon is gone |
| 89 | covered | Same as #85 |
| 90 | covered | Monotonic deadlines |
| 91 | Y-13 | Edit proposal revision |
| 92 | retired | Finding carry declined; `max_rounds` bounds it |
| 93 | covered | The queue lands one at a time (G10) |
| 94 | covered | Exit cause and start reason are on rows |
| 95 | retired | Artifact headline |
| 96 | covered | Ignored `node_modules` is not dirty |
| 97 | retired | `watch` prints every event |
| 98 | not filed | The ticket half is covered; parked is #73 |
| 99 | covered | An expired clock offers nudge |
| 100 | Y-16 | Linked worktree |
| 101 | Y-14 | Output cap |
| 102 | covered | An unchanged head raises `stopped` |
| 103 | retired | Bun code signature |
| 104 | retired | The `TMPDIR` share mount is gone |
| 105 | not filed | `attempt tail` follow |
| 106 | retired | pinfold owns liveness |
| 107 | retired | No JUnit parsing |
| 108 | retired | v2 history replay |
| 109 | covered | Sync validates only incoming configuration |
| 110 | not filed | Gates per workflow |
| 111 | retired | `preflight` is gone |
| 112 | retired | v2 docs |
| 113 | covered | G6 |
| 114 | retired | `init --patch` is gone |
| 115 | not filed | Operator in a box |
| 116 | covered | The reviewer box gets its model route (G12) |
| 117 | covered | Same as #85 |
| 118 | covered | Line reads apply backpressure |
| 119 | covered | A daemon exit reconciles as interrupted (G5) |
| 120 | covered | `doctor` lists every connection |
| 121 | retired | Replay is out |
| 122 | Y-18 | `YARD_BASE` for gates |
| 123 | Y-19 | Proof directory |
| 124 | retired | No `watch --sync` |
| 125 | covered | Gates run inside the attempt's lane |
| 126 | covered | The landing lane is outside `max_lanes` |
