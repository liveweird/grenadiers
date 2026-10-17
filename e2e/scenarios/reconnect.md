# Reserved-seat recovery

Use `E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect` against the same read-only generated fixture as the full-match suite. Retries are disabled. Native WebSocket transport closes and fresh normal authentication drive recovery; no game state, input command, simulation clock, or outcome is injected. Private capabilities remain inside browser memory; traces are disabled for this fixture and only public summaries/screenshots leave it.

## Scenario: a browser restores its full live world and action floor before a fenced takeover

Two actual UI clients start a match. One plants a real bomb, loses its native transport, and returns to the same seat/world/live bomb with its stable private capability and action floor. It escapes using the real fuse, proves focus-loss input clearing, and sends action2. Invalid capability and room use the same rejection. A separately authenticated native socket takes over; the old UI stops automatic recovery and can create a fresh room. Shared frames contain no private capability or seed. Desktop/mobile Axe and containment checks pass.

## Scenario: two offline tabs stop recovery after a late reserved-seat rejection

Both native transports close while fresh-session HTTP requests are held. After eleven real seconds, both resume attempts receive terminal reserved-seat rejection and return to a usable lobby without a retry loop. Exact599/600 expiry and abandonment precedence belong to deterministic core/server tests; this browser scenario verifies late network recovery behavior.
