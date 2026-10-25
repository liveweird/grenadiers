# Reserved-seat recovery

Use `E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect` against the same read-only generated fixture as the full-match suite. Retries are disabled. Native WebSocket transport closes and fresh normal authentication drive recovery; no game state, input command, simulation clock, or outcome is injected. Private capabilities remain inside browser memory; traces are disabled for this fixture and only public summaries/screenshots leave it.

## Scenario: a browser restores its full live world and action floor before a fenced takeover

Two actual UI clients start a match. One plants a real bomb, loses its native transport, and returns to the same seat/world/live bomb with its stable private capability and action floor. It escapes using the real fuse, proves focus-loss input clearing, and sends action2. Invalid capability and room use the same rejection. A separately authenticated native socket takes over; the old UI stops automatic recovery and can create a fresh room. Shared frames contain no private capability or seed. Desktop/mobile Axe and containment checks pass.

The connected-takeover authority check follows a second real bomb placement (action ID 2).
Escape to the already traversed (5,1) and wait for bombs/flames to clear while the player remains
alive in Running. This keeps a three-second fuse from racing browser teardown. The earlier
disconnected restore still requires a live bomb in its full baseline. After connected takeover,
a scoped movement from the replacement socket must change the peer's authoritative position;
stop cleanup must preserve any original movement failure. Admission listeners detach once the
private resume result is settled. Watchdogs and retry policy are unchanged.
The same initialized canvas remains connected across disconnected restore and terminal
takeover, with exactly one canvas after restore and a hidden canvas in the waiting state.
This verifies bounded renderer reuse while the existing live-world and authority checks run.

The reconnect accessibility audits use guarded single-frame Axe legacy mode as a bounded
experiment against post-audit browser-control stalls. The helper checks for one frame before
and after analysis and fails on any frame attachment during analysis. It retains the same
rules, zero-violations assertion, width check, screenshots and deadline. A separate controlled
fixture proves legacy/current rule-result parity and detection of an unlabeled button; no
audit fixture or game state is injected into this gameplay scenario. Screenshot diagnostics
use version-checked instance-local hooks to retain bounded fixed stage timings for the original
capture. They issue no extra browser commands and retain no arguments, results or image bytes.

## Scenario: two offline tabs stop at the recovery deadline while session requests are held

Both native transports close while fresh-session HTTP requests remain held. Each browser's real
ten-second recovery deadline returns it to a usable lobby with a connection failure, without
opening another socket or sending a resume command. Releasing the held requests afterward must
not revive recovery. No browser/game clock or authoritative state is replaced. Exact599/600
reservation expiry and abandonment precedence belong to deterministic core/server tests.

The safe second-bomb route ends in cell(5,1) using the movement helper's documented
128-unit center tolerance. Connected takeover must preserve the exact stationary position
from that route; no artificial one-sided center bound is imposed.
