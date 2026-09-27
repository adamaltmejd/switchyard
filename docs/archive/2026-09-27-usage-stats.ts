#!/usr/bin/env bun
// Read-only usage statistics from one Yard project's store, for the feature
// review of the Switchyard rebuild. Opens .yard/local/store.db read-only and
// never writes. Run from the project root (or pass the root as argv[1]):
//
//   bun run yard-usage-stats.ts [PROJECT_ROOT] > usage-stats.json
//
// Schema 49 (Yard v0.17.x). Every query is independent; a missing table or
// column is reported as an error string, not a crash.

import { Database } from "bun:sqlite";
import { existsSync, statSync } from "node:fs";
import { join } from "node:path";

const root = process.argv[2] ?? process.cwd();
const dbPath = join(root, ".yard", "local", "store.db");
if (!existsSync(dbPath)) {
  console.error(`no store at ${dbPath}`);
  process.exit(2);
}
const db = new Database(dbPath, { readonly: true });
const q = (sql: string): unknown => {
  try {
    return db.query(sql).all();
  } catch (e) {
    return { error: String(e).split("\n")[0] };
  }
};

const out = {
  project: root,
  storeBytes: statSync(dbPath).size,
  walBytes: existsSync(`${dbPath}-wal`) ? statSync(`${dbPath}-wal`).size : 0,
  schema: q("select max(id) as version from schema_migrations"),
  span: q("select min(created_at) as first, max(created_at) as last from ticket"),
  tickets: {
    byStatus: q("select status, count(*) as n from ticket group by 1"),
    byWorkflow: q("select workflow, count(*) as n from ticket group by 1"),
    planFirst: q("select plan_first, count(*) as n from ticket group by 1"),
    parkedNow: q("select count(*) as n from ticket where parked = 1"),
    triaged: q("select count(*) as n from ticket where triage is not null"),
    fromProposal: q("select count(*) as n from ticket where origin_execution_id is not null"),
    links: q("select count(*) as n from ticket_link"),
    scopeHints: q("select count(*) as n from ticket_scope_hint"),
    plans: q("select count(*) as n from plan"),
    editsPerTicket: q("select avg(revision) as mean, max(revision) as max from ticket"),
  },
  lanes: {
    byStatus: q("select status, count(*) as n from lane group by 1"),
    attemptsPerTicket: q(
      "select attempts, count(*) as tickets from (select ticket_id, count(*) as attempts from lane group by 1) group by 1 order by 1",
    ),
  },
  executions: {
    byKindStatus: q("select kind, status, count(*) as n from execution group by 1,2 order by 1,3 desc"),
    generationDepth: q(
      "select kind, max(generation) as maxGeneration, avg(generation) as meanGeneration from execution group by 1",
    ),
    deepGenerations: q(
      "select kind, generation, count(*) as n from execution where generation >= 5 group by 1,2 order by 1,2",
    ),
    resumedSessions: q("select kind, count(*) as n from execution where session_id is not null group by 1"),
    nudges: q("select count(*) as n from execution where queued_turn is not null"),
    costUsdByKind: q(
      "select kind, round(sum(json_extract(usage,'$.costUsd')),2) as usd, count(usage) as withUsage from execution group by 1",
    ),
    startDispositions: q("select start_disposition, count(*) as n from execution group by 1"),
    cancellationCauses: q("select cancellation_cause, count(*) as n from execution where cancellation_cause is not null group by 1"),
  },
  checks: q("select kind, verdict, count(*) as n from check_result group by 1,2"),
  approvals: q("select decision, actor, count(*) as n from approval group by 1,2"),
  merges: q("select outcome, count(*) as n from merge_attempt group by 1"),
  attention: q("select kind, count(*) as n from attention_item group by 1 order by 2 desc"),
  attentionOpen: q("select kind, count(*) as n from attention_item where resolved_at is null group by 1"),
  audit: {
    rows: q("select count(*) as n from audit_event"),
    byAction: q(
      "select action, count(*) as n, sum(length(detail)) as detailBytes from audit_event group by 1 order by 2 desc",
    ),
    byActor: q("select actor, count(*) as n from audit_event group by 1"),
  },
  // Verbs the operator used, as the audit stream records them. Nudge, park,
  // abandon, relate, residual acceptance, relocation and proposals live here.
  operatorVerbs: q(
    "select action, count(*) as n from audit_event where action in ('execution.nudge-queued','lane.abandoned','lane.rejected','lane.residual-accepted','lane.approval-superseded','ticket.linked','ticket.unlinked','ticket.edited','proposal.file','proposal.accept','proposal.reject','plan.publish','admission.paused','admission.resumed','project.relocated','project.canonical-synced') group by 1 order by 2 desc",
  ),
};

console.log(JSON.stringify(out, null, 2));

const rows = (v: unknown) => (Array.isArray(v) ? v : []);
const line = (label: string, v: unknown) =>
  console.error(`${label}: ${rows(v).map((r: any) => Object.values(r).join("=")).join(", ")}`);
console.error(`\n${root}  store ${(out.storeBytes / 1e6).toFixed(0)} MB`);
line("tickets", out.tickets.byStatus);
line("workflows", out.tickets.byWorkflow);
line("lanes", out.lanes.byStatus);
line("attention", out.attention);
line("operator verbs", out.operatorVerbs);
line("cost by kind", out.executions.costUsdByKind);
