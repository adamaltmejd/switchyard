//! The schema, one forward migration per entry. A store whose
//! `user_version` is past the last entry is from a newer Yard and refused.

pub const MIGRATIONS: &[&str] = &[
    r#"
CREATE TABLE ticket (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    priority INTEGER NOT NULL,
    workflow TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('open', 'done', 'abandoned')),
    parked INTEGER NOT NULL DEFAULT 0,
    revision INTEGER NOT NULL DEFAULT 1,
    origin TEXT NOT NULL,
    created_at TEXT NOT NULL,
    closed_at TEXT,
    close_reason TEXT
);

CREATE TABLE dependency (
    ticket INTEGER NOT NULL REFERENCES ticket(id),
    depends_on INTEGER NOT NULL REFERENCES ticket(id),
    PRIMARY KEY (ticket, depends_on)
);

CREATE TABLE attempt (
    id INTEGER PRIMARY KEY,
    ticket INTEGER NOT NULL REFERENCES ticket(id),
    workflow TEXT NOT NULL,
    implementer TEXT NOT NULL,
    base TEXT NOT NULL,
    head TEXT,
    target TEXT,
    state TEXT NOT NULL CHECK (state IN ('live', 'ended')),
    outcome TEXT,
    lane INTEGER NOT NULL DEFAULT 1,
    lane_since INTEGER,
    work_ms INTEGER NOT NULL DEFAULT 0,
    next TEXT,
    nudge TEXT,
    rounds INTEGER NOT NULL DEFAULT 0,
    extra_rounds INTEGER NOT NULL DEFAULT 0,
    landing_reds INTEGER NOT NULL DEFAULT 0,
    cleaned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    ended_at TEXT
);
CREATE UNIQUE INDEX attempt_one_live ON attempt(ticket) WHERE state = 'live';

CREATE TABLE execution (
    id INTEGER PRIMARY KEY,
    attempt INTEGER NOT NULL REFERENCES attempt(id),
    parent INTEGER REFERENCES execution(id),
    kind TEXT NOT NULL CHECK (kind IN ('implementation', 'review', 'gate', 'landing', 'cleanup')),
    reason TEXT,
    status TEXT NOT NULL CHECK (status IN ('running', 'ended')),
    outcome TEXT,
    detail TEXT,
    base TEXT,
    head TEXT,
    ticket_revision INTEGER,
    digest TEXT,
    name TEXT,
    round INTEGER,
    handle TEXT,
    image_id TEXT,
    agent TEXT,
    harness TEXT,
    harness_version TEXT,
    provider TEXT,
    model TEXT,
    effort TEXT,
    resumed INTEGER REFERENCES execution(id),
    session_id TEXT,
    exit_cause TEXT,
    tokens_in INTEGER,
    tokens_out INTEGER,
    cost REAL,
    exit_code INTEGER,
    oom_kills INTEGER,
    progress TEXT,
    approval INTEGER REFERENCES approval(id),
    intent_old TEXT,
    intent_merged TEXT,
    intent_state TEXT CHECK (intent_state IN ('open', 'resolved')),
    started_at TEXT NOT NULL,
    ended_at TEXT
);
CREATE INDEX execution_attempt ON execution(attempt);
CREATE INDEX execution_running ON execution(status) WHERE status = 'running';

CREATE TABLE "check" (
    id INTEGER PRIMARY KEY,
    execution INTEGER NOT NULL REFERENCES execution(id),
    attempt INTEGER NOT NULL REFERENCES attempt(id),
    kind TEXT NOT NULL CHECK (kind IN ('gate', 'review')),
    name TEXT NOT NULL,
    base TEXT NOT NULL,
    head TEXT NOT NULL,
    ticket_revision INTEGER NOT NULL,
    digest TEXT NOT NULL,
    verdict TEXT NOT NULL CHECK (verdict IN ('pass', 'fail', 'error')),
    image_id TEXT,
    round INTEGER,
    created_at TEXT NOT NULL
);
CREATE INDEX check_attempt ON "check"(attempt);

CREATE TABLE finding (
    id INTEGER PRIMARY KEY,
    "check" INTEGER NOT NULL REFERENCES "check"(id),
    priority INTEGER NOT NULL,
    file TEXT,
    line INTEGER,
    category TEXT,
    body TEXT NOT NULL
);

CREATE TABLE approval (
    id INTEGER PRIMARY KEY,
    attempt INTEGER NOT NULL REFERENCES attempt(id),
    base TEXT NOT NULL,
    head TEXT NOT NULL,
    ticket_revision INTEGER NOT NULL,
    gate_digest TEXT NOT NULL,
    review_digest TEXT NOT NULL,
    checks TEXT NOT NULL,
    actor TEXT NOT NULL,
    text TEXT,
    state TEXT NOT NULL CHECK (state IN ('active', 'withdrawn', 'landed', 'retired')),
    created_at TEXT NOT NULL
);

CREATE TABLE attention (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('approval', 'proposal', 'stopped', 'red')),
    reason TEXT NOT NULL,
    ticket INTEGER REFERENCES ticket(id),
    attempt INTEGER REFERENCES attempt(id),
    execution INTEGER REFERENCES execution(id),
    payload TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('open', 'resolved')),
    resolution TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT
);
CREATE INDEX attention_open ON attention(state) WHERE state = 'open';

CREATE TABLE audit (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    at TEXT NOT NULL,
    event TEXT NOT NULL,
    ticket INTEGER,
    attempt INTEGER,
    execution INTEGER,
    attention INTEGER,
    text TEXT,
    data TEXT NOT NULL
);
"#,
    // Candidate identity grew a proof snapshot: the copy of the worker-written
    // proof directory a candidate was judged on.
    r#"
ALTER TABLE attempt ADD COLUMN proof TEXT;
ALTER TABLE execution ADD COLUMN proof TEXT;
ALTER TABLE "check" ADD COLUMN proof TEXT NOT NULL DEFAULT '';
ALTER TABLE approval ADD COLUMN proof TEXT NOT NULL DEFAULT '';
"#,
    // The harness's registration proof, once its reader yields a complete one:
    // `registered`, or `refused` when the reader refused it.
    "ALTER TABLE execution ADD COLUMN mcp TEXT;",
    // A check is its execution's input plus a verdict; the copies go.
    r#"
DROP INDEX check_attempt;
ALTER TABLE "check" DROP COLUMN attempt;
ALTER TABLE "check" DROP COLUMN name;
ALTER TABLE "check" DROP COLUMN base;
ALTER TABLE "check" DROP COLUMN head;
ALTER TABLE "check" DROP COLUMN proof;
ALTER TABLE "check" DROP COLUMN ticket_revision;
ALTER TABLE "check" DROP COLUMN digest;
ALTER TABLE "check" DROP COLUMN image_id;
ALTER TABLE "check" DROP COLUMN round;
"#,
    // A ticket edit steers a live attempt: nudges go, and the attempt keeps the
    // body its implementer last read so a resumed session gets the diff.
    r#"
ALTER TABLE attempt DROP COLUMN nudge;
ALTER TABLE attempt ADD COLUMN body_read TEXT;
"#,
    // The body an implementer execution's start transaction read.
    "ALTER TABLE execution ADD COLUMN body TEXT;",
    // The diff baseline is the latest launched execution's body.
    "ALTER TABLE attempt DROP COLUMN body_read;",
    // An approval given over a blocking review at the round limit says so.
    "ALTER TABLE approval ADD COLUMN overrode INTEGER NOT NULL DEFAULT 0;",
    "ALTER TABLE ticket ADD COLUMN target_branch TEXT;
     ALTER TABLE attempt ADD COLUMN target_branch TEXT;",
    "ALTER TABLE execution ADD COLUMN reserved INTEGER NOT NULL DEFAULT 0;
     UPDATE execution SET reserved = 1 WHERE status = 'running';",
    "ALTER TABLE execution ADD COLUMN lock_contract INTEGER NOT NULL DEFAULT 1;
     UPDATE execution SET lock_contract = 0;
     UPDATE execution SET reserved = 1 WHERE status = 'running' OR intent_state = 'open'
        OR (kind != 'cleanup' AND outcome IN ('error', 'failed', 'timeout', 'interrupted', 'refused', 'abandoned'))
        OR kind IN ('gate', 'review')
        OR (kind = 'landing' AND outcome IN ('red', 'conflict'));",
    "ALTER TABLE execution ADD COLUMN intent_ref TEXT;
     UPDATE execution SET intent_ref = 'refs/heads/' || (SELECT target_branch FROM attempt WHERE attempt.id = execution.attempt) WHERE intent_state IS NOT NULL;",
    // Historical admission evidence cannot be recovered from mutable bases.
    "ALTER TABLE attempt ADD COLUMN admission TEXT;",
    // Old results have no proved plan binding.
    "ALTER TABLE execution ADD COLUMN plan_revision INTEGER;\n     ALTER TABLE approval ADD COLUMN plan_revision INTEGER;",
    // Old zeros have no evidence of a measured bill; preserve them as legacy.
    r#"ALTER TABLE execution ADD COLUMN usage_fields TEXT;
       UPDATE execution SET usage_fields = '{"input":{"completeness":"unknown","provenance":"legacy"},"output":{"completeness":"unknown","provenance":"legacy"},"cost":{"completeness":"unknown","provenance":"legacy"}}';"#,
    // No node, result or teardown evidence is inferred for historical rows.
    "ALTER TABLE execution ADD COLUMN node TEXT;
     ALTER TABLE execution ADD COLUMN inputs TEXT;
     ALTER TABLE execution ADD COLUMN result TEXT;
     ALTER TABLE execution ADD COLUMN teardown INTEGER;",
    "ALTER TABLE execution ADD COLUMN report TEXT;",
    // No table references attention. Rebuild its closed kind constraint while
    // keeping every historical row and the open-item index in one transaction.
    r#"
CREATE TABLE attention_next (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('approval', 'proposal', 'stopped', 'red', 'candidate')),
    reason TEXT NOT NULL,
    ticket INTEGER REFERENCES ticket(id),
    attempt INTEGER REFERENCES attempt(id),
    execution INTEGER REFERENCES execution(id),
    payload TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('open', 'resolved')),
    resolution TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT
);
INSERT INTO attention_next
    (id, kind, reason, ticket, attempt, execution, payload, state, resolution, created_at, resolved_at)
SELECT id, kind, reason, ticket, attempt, execution, payload, state, resolution, created_at, resolved_at
FROM attention;
DROP TABLE attention;
ALTER TABLE attention_next RENAME TO attention;
CREATE INDEX attention_open ON attention(state) WHERE state = 'open';
"#,
    "ALTER TABLE approval ADD COLUMN accepted_skipped TEXT;",
    "ALTER TABLE execution ADD COLUMN prepared TEXT;",
    "CREATE TABLE plan_revision (
        attempt INTEGER NOT NULL REFERENCES attempt(id),
        revision INTEGER NOT NULL CHECK (revision > 1),
        evidence TEXT NOT NULL,
        created_at TEXT NOT NULL,
        PRIMARY KEY (attempt, revision)
     );",
    "ALTER TABLE execution ADD COLUMN verification TEXT;",
    // No historical scope is inferred during the nullable association rebuild.
    r#"
CREATE TABLE execution_next (
    id INTEGER PRIMARY KEY,
    attempt INTEGER REFERENCES attempt(id),
    parent INTEGER REFERENCES execution(id),
    kind TEXT NOT NULL CHECK (kind IN ('implementation', 'review', 'gate', 'landing', 'cleanup', 'command', 'report')),
    reason TEXT,
    status TEXT NOT NULL CHECK (status IN ('running', 'ended')),
    outcome TEXT,
    detail TEXT,
    base TEXT,
    head TEXT,
    ticket_revision INTEGER,
    digest TEXT,
    name TEXT,
    round INTEGER,
    handle TEXT,
    image_id TEXT,
    agent TEXT,
    harness TEXT,
    harness_version TEXT,
    provider TEXT,
    model TEXT,
    effort TEXT,
    resumed INTEGER REFERENCES execution(id),
    session_id TEXT,
    exit_cause TEXT,
    tokens_in INTEGER,
    tokens_out INTEGER,
    cost REAL,
    exit_code INTEGER,
    oom_kills INTEGER,
    progress TEXT,
    approval INTEGER REFERENCES approval(id),
    intent_old TEXT,
    intent_merged TEXT,
    intent_state TEXT CHECK (intent_state IN ('open', 'resolved')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    proof TEXT,
    mcp TEXT,
    body TEXT,
    reserved INTEGER NOT NULL DEFAULT 0,
    lock_contract INTEGER NOT NULL DEFAULT 1,
    intent_ref TEXT,
    plan_revision INTEGER,
    usage_fields TEXT,
    node TEXT,
    inputs TEXT,
    result TEXT,
    teardown INTEGER,
    report TEXT,
    prepared TEXT,
    verification TEXT,
    scope TEXT
);
INSERT INTO execution_next (id, attempt, parent, kind, reason, status, outcome, detail, base, head, ticket_revision, digest, name, round, handle, image_id, agent, harness, harness_version, provider, model, effort, resumed, session_id, exit_cause, tokens_in, tokens_out, cost, exit_code, oom_kills, progress, approval, intent_old, intent_merged, intent_state, started_at, ended_at, proof, mcp, body, reserved, lock_contract, intent_ref, plan_revision, usage_fields, node, inputs, result, teardown, report, prepared, verification)
SELECT id, attempt, parent, kind, reason, status, outcome, detail, base, head, ticket_revision, digest, name, round, handle, image_id, agent, harness, harness_version, provider, model, effort, resumed, session_id, exit_cause, tokens_in, tokens_out, cost, exit_code, oom_kills, progress, approval, intent_old, intent_merged, intent_state, started_at, ended_at, proof, mcp, body, reserved, lock_contract, intent_ref, plan_revision, usage_fields, node, inputs, result, teardown, report, prepared, verification FROM execution;
DROP TABLE execution;
ALTER TABLE execution_next RENAME TO execution;
CREATE INDEX execution_attempt ON execution(attempt);
CREATE INDEX execution_running ON execution(status) WHERE status = 'running';
"#,
    // Historical workspace and settings identity remains unknown.
    "ALTER TABLE execution ADD COLUMN workspace TEXT;
     ALTER TABLE execution ADD COLUMN settings TEXT;
     CREATE UNIQUE INDEX execution_workspace_owner ON execution(attempt, workspace)
       WHERE workspace IS NOT NULL AND (status = 'running' OR reserved = 1);",
    // Earlier stores have no picker provenance. Do not infer a choice from head.
    "ALTER TABLE attempt ADD COLUMN choice INTEGER REFERENCES execution(id);
     ALTER TABLE approval ADD COLUMN choice INTEGER REFERENCES execution(id);",
];
