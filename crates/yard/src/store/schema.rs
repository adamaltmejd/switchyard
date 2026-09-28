//! The schema, one forward migration per entry. A store whose
//! `user_version` is past the last entry is from a newer Yard and refused.

pub const MIGRATIONS: &[&str] = &[r#"
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
    started_once INTEGER NOT NULL DEFAULT 0,
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
    branch TEXT NOT NULL,
    base TEXT NOT NULL,
    head TEXT,
    target TEXT,
    state TEXT NOT NULL CHECK (state IN ('live', 'ended')),
    outcome TEXT,
    lane INTEGER NOT NULL DEFAULT 1,
    lane_since TEXT,
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
"#];
