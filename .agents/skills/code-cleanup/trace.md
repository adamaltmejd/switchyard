Read AGENTS.md and docs/ARCHITECTURE.md. Edit nothing. Use `rg` to trace:

1. configuration, CLI and worker-tool behavior to the spec. Report
   contradictions or unsupported features; the spec need not enumerate
   every implementation detail;
2. spec-named reasons, attention kinds and audit events to their emitted
   values;
3. every guarantee row's "Shown by" clauses to the tests that show them.

Report missing tests only for existing "Shown by" clauses. An untested
key, flag or combination alone is not a finding. Do not expand the spec or
suite to fill a coverage matrix.

Report only gaps, one per line:
`<file>:L<line>: <item>: no spec line | no test | no code.`
