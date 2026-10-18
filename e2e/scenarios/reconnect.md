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

## Scenario: two offline tabs stop recovery after a late reserved-seat rejection

Both native transports close while fresh-session HTTP requests are held. After eleven real seconds, both resume attempts receive terminal reserved-seat rejection and return to a usable lobby without a retry loop. Exact599/600 expiry and abandonment precedence belong to deterministic core/server tests; this browser scenario verifies late network recovery behavior.
