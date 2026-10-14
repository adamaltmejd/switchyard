# Qualification and host budget, 2026-10-14

This is evidence for the recorded candidates, not release acceptance.
The installed operational Yard and Pinfold binaries were unchanged.
The host qualification used Pinfold 0.2.2 and the isolated shifting-profile
store with fuse-overlayfs. It does not resolve the original native-overlay
LXC failure in Pinfold #89.

## Completed host gates

| Ticket | Exact candidate | Gate | Tests | Suite seconds |
| --- | --- | --- | --- | --- |
| Y136 | 96b7d72b5bd817097d526be6db752a775ae0087f | 2954 | 79/79 | 753.04 |
| Y109 | 1fc4e9c1c9379a8354bff3d304c76abaf9e96c0e | 2965 | 79/79 | 753.35 |
| Y62 | dec84892b120fe9060bb5c669890e8e308965ca1 | 2976 | 79/79 | 764.36 |

The present warm host suite takes about 13 minutes. This revises the
original five-minute estimate; no scenario was deleted to meet it.
Y62 historical fixture builds reported 40.70, 23.47 and 23.95 seconds.
Their sum is not necessarily removable wall time because suite work overlaps.

A sequential fixture-build experiment measured 93.682 seconds for separate
build targets and 55.103 seconds for shared dependencies with Yard cleaned
and rebuilt before copying each distinct binary. This saves 38.579 seconds
of measured setup. Sharing without cleaning produced identical binaries and
was rejected. Y152 still requires actual scenario and full host validation.

## Scoped fault evidence

Y136 caught six independent source faults: injected null effort, required
inner effort, accepted unsupported old effort, injected explicit effort into
a six-field old package, removed recorded capability, and removed exact
package capability. Restored G8 passed in 349.55 seconds.

The first two faults fail at the earlier corrected-publication assertion.
They do not separately qualify the later omission terminal observer.
The old-package injection directly qualifies the early terminal observer.
The unsupported-effort fault fails at refusal before later no-write checks.
Metadata and exact-package fallback faults independently change the actual
provider request. Neither implies universal historical package compatibility.

Y109's gate-sync overwrite fault directly failed the new frozen-plan
comparison before adoption. Restored G2 passed in 75.34 seconds. Later
staging and image comparisons passed in the full suite, but this fault did
not independently qualify them.

The full manifests, source patches, terminal results and hashed raw logs are:

- /home/yard/validation/y136-96b7d7-host-controls-20261014-evidence/qualified-manifest.json
- /home/yard/validation/y109-1fc4e9-host-controls-20261014-evidence/scoped-manifest.json
- /home/yard/validation/y111-operator-evidence-20261013/approved-y62-dec848-gate-capture/

Earlier failed Y136 gates 2927 and 2944 remain retained. Later passes do
not erase them. Actual historical migration, final platform validation,
CI restart holder identification and release acceptance remain separate.
