---
name: yard-operator
description: Drive Switchyard through trusted operator MCP. Load yard-drive for board judgment and yard-file for filing and proposals.
---

# Operating yard

Run `yard mcp` on the trusted host with the daemon already serving. Tools
are `yard_` plus daemon methods with dots replaced by underscores.
Use `tools/list` for schemas. Pass an absolute registered `project` path
on every project operation; `yard_init` and `yard_project_forget` take
an absolute `path` instead. There is no task-management CLI.

Results are `structuredContent.result`; errors are
`structuredContent.error` with `code`, `message`, and `data`.
Keep expected/current identities on stale refusals. Re-read before a new
decision; never substitute a new identity into an old judgment.

## Board and filing

`yard_status` shows tickets, attempts, attention, queue, running and seq.
Ticket bodies are omitted: use `yard_ticket_show` to read one.
`yard_attention` polls at most 25 seconds. Without `since`, it returns on
open attention or an idle board. With `since`, it waits for a later open
item or an idle board after a later attempt/execution end. A bounded poll
with no item and `idle: false` is not a drained board; poll again with the
same cursor. Pass the returned seq after an actual wake.

`yard_events` returns at most 500 events after `since`, waiting at most
25 seconds when empty. Advance only to the last event consumed. Empty pages
do not advance it. Cancel or close a wait to release observation, not work.

`yard_ticket_new` takes title, body, priority, depends_on, parked, workflow
and optional target. The body is the plan. Ready unparked tickets start
automatically once dependencies are done. `yard_attempt_start` admits or
retries one. Planning workers propose work without changing code.

## Decisions

Read each item's kind, reason, ticket, attempt and exits. Use only an exit
it names. The tool schema lists types; the item and inspection supply the
exact identities. Do not infer missing ones.

- Approval: read `yard_attempt_show` and `yard_attempt_diff`, then use
  `yard_attempt_approve`, `yard_attempt_reject`, or `yard_attempt_abandon`.
  Approve/reject bind the frozen target, head, proof when present and choice
  when selected. Reject takes nonempty text for repair.
- Proposal: `yard_proposal_accept` or `yard_proposal_reject` takes the
  attention id and decision text.
- Candidate: read the failed gate or review evidence. `yard_attempt_accept`
  or `yard_attempt_repair` binds attention, check, plan_revision, target,
  head, proof when present and text. Copy any named landing, child, base,
  revision, old, merged and node bindings too. Acceptance retains the
  original failure and runs remaining selected checks before approval.
  Omitted-node decisions also bind the named omissions and prepared input.
- Stopped: start, edit or abandon as the item permits. Timeout permits only
  edit and abandon; limit can also permit exact approval over findings.
- Red: start, edit or abandon as named. A landing item with no attempt
  permits only its named start exit.

## Inspection and edits

`yard_attempt_tail` reads the transcript. `yard_attempt_stats`,
`yard_project_stats` and execution inspection retain unknown/partial usage.
`yard_ticket_edit` needs the revision read. It edits title, body, priority,
workflow and inactive target override; default_target clears the override.
`yard_ticket_park`, `yard_ticket_unpark`, `yard_ticket_depend`,
`yard_ticket_done` and `yard_ticket_abandon` handle the remaining ticket
operations; done requires a reason.

`yard_plan_catalog`, `yard_plan_resolve` and `yard_plan_validate` inspect
target-bound capabilities and resolve structured plans. `yard_attempt_revise`
uses plan revision CAS; `yard_attempt_select` binds exact selected inputs.
Running inputs stay immutable. Standalone `yard_execution_start`, show and
stop use the same lifecycle and confer no landing authority.

`yard_sync` imports committed history and consumes canonical into the
checkout. Optional branch selects a local branch; default selects it as
project default. Divergence names both heads and refuses. `yard_doctor`
reports pinfold, configuration, image and credentials. `yard_project_list`
lists the registry; forget retires its named project image.

Service commands remain `yard daemon run|install|uninstall|status|restart`;
`yard version` reports the executable. Construction stays on its frozen
installed executable until the operator deliberately switches it. Shipped
successor skills do not authorize that switch or construction-state access.
