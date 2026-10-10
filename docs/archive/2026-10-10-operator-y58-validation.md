# Y-58 operator validation — 2026-10-10

This record covers trusted operator stdio MCP. Release validation,
publication and operational switchover remain separate work.

## Candidate corrections

Initial cfd73a1f12cdb8b4d4e9a8a8b108d2850445f6b9 passed 42 real host
scenarios in 226.00 seconds. Source and assertion audits found three gaps:
the disconnect test killed the client instead of exercising stdin EOF,
attention's poll deadline excluded readiness Git/config reads, and fatal
transport errors could drop accepted mutation forwards.

Repaired 280c5bf1584aa7aaa9944cc0632d356627b76ef4 preserved forwards
through transport errors and bounded the complete poll. Its normal EOF
proof passed. The full host suite passed 42 and failed one in 211.40 seconds:
the new allowed-sync control compared a missing isError with false because
its helpers had already unwrapped the MCP response. Static audit also found
expiry could advance the cursor past attention it had not returned.
The operator edited Y-58 to revision 5 with these concrete findings and
an error-exit assertion correction. No candidate was approved over them.

## Latest validated candidate

b549805a627abef7d5c38c209323814d6e7d1758 observes raw MCP tool success
for the control, checks a normal non-success exit, and preserves the caller's
attention cursor on incomplete expiry. Binding architecture changes match.
Read-only source and assertion audits close the named gaps.

All 43 real host scenarios passed in 189.77 seconds. The new G7 transport
scenario holds a real sync Git effect while receiving a fatal oversized
frame, observes the refusal before release, then proves the accepted mutation
completion and canonical head from host Git. Normal EOF while a real worker
is held exits successfully and allows the worker to reach approval and land.
Worker/operator grants, stale identities and schema refusal retain controls.

Logs:

- /tmp/switchyard-y58-host.log
- /tmp/switchyard-y58-repair-host.log
- /tmp/switchyard-y58-repair2-host.log

DeepSeek execution 1165/check 620 passed with no findings. Candidate fmt,
locked all-targets clippy and build checks 611–613 passed. Approval 70
bound the exact head and empty proof digest
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.
Landing 1175 passed fmt, clippy, build and all 43 host scenarios in
195.11 seconds. The intent was event 4057; landing.recorded was 4059,
ticket completion 4060 and attempt completion 4061. Cleanup 1180 ended
removed at 4063. Sync consumed the recorded head.

The operational executable retains SHA256
05ae25832964703c81525e7c5f5be7a8148954517cfddf69ef9846953bf1ebb2.
No binary was installed and no operational daemon was replaced.
Y-83 and the remaining release scope continue; Y-81 remains parked.
