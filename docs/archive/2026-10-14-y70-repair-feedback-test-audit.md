# Y-70 selected producer repair feedback

Keep the extension to the existing G2/G8 selected-candidate scenario. After
the original five concurrent producers, same frozen reviewer, failed-picker
retry, operator choice controls and source replacement, the replacement B
fails the real candidate gate. The fixture requires a repair file for the new
version and emits selected-B-needs-repair-marker with exit 94 when it is absent.
The real B harness must receive that marker in its new user prompt before the
script writes the required repair file, commits and holds the resulting head.
Git in B's retained clone supplies both the failed and repaired heads; fixture
bytes supply the failure marker, repair bytes and exit code.

Store observations while that harness is held bind one repair execution to B's
actual producer model/workspace and failed head. Its feedback retains the exact
failed gate execution/result source, gate identity, verdict and fixture detail.
The current choice is absent after repair admission. Releasing the worker
requires a new immutable B publication, fresh own review, a new read-only picker
choice, successful native checking and exact approval before the original
held-intent landing control. The failed gate row remains byte-for-byte unchanged
after successful continuation. All native checks still name B versions only.

Sabotage: drop Start.feedback in writer-graph binding, or invalidate the choice
before binding the repair execution. The worker then lacks the failure marker
or repair selects the wrong producer. The scenario's earlier passing checks
cannot expose lost failure provenance. Ordinary linear repair tests declare a
feedback input and do not traverse the writer-graph binding branch.

The implementation also snapshots pending rejection and landing-return notes
before start_writer clears next. Existing structured feedback is copied without
changing its source identity. A plain pending instruction gets an external
feedback envelope; no check, result, acceptance or authority is fabricated.
Requalification reads the same saved input, using the existing execution
lifecycle. Prior source, choice, raw review and failure rows remain history.

Existing assertion blocks removed or weakened: 0. The fixture now expects the
repaired B version to land and includes its externally observed head among the
allowed B-only check heads. Five producer concurrency, collision-shaped names,
same reviewer, picker failure retry, stale/foreign choice refusals, source-change
invalidation and open-intent refusal/allowed control remain intact. No duplicate
matrix, generic harness, mock, sleep or binary barrier is added. Host e2e is not
run in this worker box; it remains the reviewed landing gate.

The local approval observer now waits for the next event instead of reusing the
first matching historical event. Each continuation must earn its own approval;
the existing exact choice and head assertions remain unchanged.
