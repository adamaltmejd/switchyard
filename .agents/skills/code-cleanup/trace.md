Edit nothing. Use `rg` over crates/ and docs/ARCHITECTURE.md to trace:

1. every key `config.rs` reads to its row under `## Projects`, and then to
   a test that sets it;
2. every verb and flag `cli.rs` parses to `## Daemon and CLI`, and then to
   a test;
3. every worker tool `mcp.rs` serves to `## Worker tools`, and then to a
   test;
4. every reason, attention kind and audit event name the binary emits to
   the spec line that names it;
5. every clause of every guarantee row's "Shown by" to the test function
   that shows it.

Report only gaps, one per line:
`<file>:L<line>: <item>: no spec line | no test | no code.`
