# Browser visibility hardening

Run `E2E_MATCH_FIXTURE=arena-v1-matches npm run test:hardening` against the documented complete-match
fixture. Chromium must be headed; Linux runs require Xvfb. The dedicated project restores Chromium's
ordinary background throttling and renderer behavior, disables retries and traces, and retains only
public command, room, and actor summaries on failure. Its default-context CDP connection disables
Playwright's focus-emulation defaults so cover-tab activation exposes native visibility state.

## Scenario: a hidden gameplay tab clears held input and recovers on a fresh native action

Two real clients start a generated match. Player one holds a native movement key, then a cover tab
in the same browser context comes to the front. The gameplay document must report an actual hidden
transition. Player two observes the authoritative actor stop across fresh snapshots while the server
clock advances for more than one second and the gameplay socket count remains stable. On the actual
visible transition, releasing the old key does not replay input or place a bomb. A new native key
moves the actor, stops normally, and the opponent's snapshots and HUD remain current.
