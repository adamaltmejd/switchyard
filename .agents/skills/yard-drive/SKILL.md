---
name: yard-drive
description: Answer a Yard board through operator MCP. Keep the watch loop running, use named exits, read before approving, handle findings and retire tickets in order.
---

# yard-drive

Yard runs the work. You take decisions between its steps. For tool schemas
and operations see yard-operator; for filing see yard-file.

## The loop

Call `yard_attention` with the absolute project path. Its result has seq,
attention and idle. Empty attention means drained only when idle is true.
An empty bounded poll with idle false requires another poll at the same
cursor. To wait past deliberately open items, pass the seq returned after
a wake as since. It waits for a newer open item or an idle board after a
newer attempt/execution end, then returns every open item.

After every decision return to that loop; sync when woken. `yard_status`
shows decisions still owed. `yard_events` reads bounded ordered audit pages:
advance since only through consumed events. Closing or cancelling observation
leaves work running. If idle and nothing is open, file, unpark, start work,
or report the idle board and its seq.

## Attention

Answer each item with one of its exits. Nothing else.

- Start continues the named stopped/red work through the existing lifecycle.
- Candidate accept records exact failed evidence and runs remaining selected
  checks. Read the evidence first. Repair takes its bounded repair route.
- Edit uses `yard_ticket_edit` and the read revision. On timeout it renews
  the clock; on limit it permits another review round. Exact limit approval
  can land over findings; read them first.
- If no exit fits, read the reason, `yard_attempt_show` and `tools/list`.
  Leave unresolved items open and ask. Unknown outcomes need investigation.

Touch Yard state only through supported operator tools. Never edit
`.yard/local`, remove a clone or box by hand, or kill a process.

## Approval

1. Read `yard_attempt_show`: selected checks, raw findings and decisions.
2. Read `yard_attempt_diff` against the ticket body and current spec.
3. Call `yard_attempt_approve` with the exact frozen target, head, proof
   when named and choice when selected. Copy all other required bindings
   from the attention evidence; explicitly name any accepted omissions.

Worker summaries and commit messages are not approval evidence. Green
gates and review inform the decision. Read protected guidance hunks with
care because future workers consume them. Reject with `yard_attempt_reject`
using the same exact identities and text describing the repair.

Landing is complete at `landing.recorded`, not at approval. Call
`yard_sync` to consume it into the checkout.

## Findings and retirement

Land over findings below blocking by default. Reject only for integrity:
a guard, identity binding, fail-safe path or irreplaceable data. File work
only when it passes yard-file's admission rule.

Edit the ticket with its read revision. A running implementer finishes
first; stop followed by the stopped item's start delivers it sooner. Gates
and review stop immediately and approval is superseded.

Retire in order: `yard_ticket_park`, then `yard_attempt_abandon` with a
reason, then `yard_ticket_abandon` or `yard_ticket_done` with a reason.
Abandoning without parking can make the ticket ready again immediately.

## Sync and upgrades

Use `yard_sync` for committed operator changes between landings. Changed
target, prepared ref or selected evidence retires pre-intent continuation.
Unchanged exact evidence can retain eligible passing or explicitly accepted
failed results after ownership release and current identity/policy checks.
Interrupted and infrastructure-error children rerun.

Template sync affects future admissions. Active work keeps admitted inputs
until explicit plan revision. Approval and protected-path policy remain live;
containment, capacity and credentials remain operational constraints.
Unrecorded historical identity stays unknown.

A deliberately installed new binary requires `yard daemon restart`.
Interrupted work returns stopped; answer its named exits. Construction's
frozen executable is never switched merely because source skills changed.

Ask when a refusal cannot be resolved, an exit does not fit, an outcome is
unknown, a diff is unclear, or a decision changes a guarantee, threat model
or scope. Then return to the watch loop.
