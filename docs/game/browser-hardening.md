# Browser visibility hardening

The first M7 browser-hardening slice exercises a real hidden and visible Chromium tab transition.
It uses the existing v4 client, generated match fixture, native keyboard input, authoritative server
ticks, and public opponent snapshots. It does not replace visibility APIs, dispatch synthetic DOM
events, clear controls from the test, alter the game clock, or inject state.

The dedicated Playwright project runs headed with retries disabled and removes Chromium defaults
that disable background timer and renderer throttling. Playwright normally applies focus emulation
to each attached page, which makes every automation target appear visible. This fixture launches
the pinned Chromium binary and reconnects with Playwright's supported `noDefaults` option in the
default context, restoring native tab visibility without setting document state. On Linux, run it
under Xvfb:

```sh
E2E_MATCH_FIXTURE=arena-v1-matches \
  xvfb-run -a npm --prefix e2e run test:hardening
```

The test holds a native movement key before activating a cover tab in the same browser context.
The opponent must observe an explicit stop and a stable position across fresh snapshots. During at
least one second hidden, authoritative ticks continue and no replacement gameplay socket appears.
After returning, the old key cannot replay movement or a bomb; a fresh native key must move and stop,
with current snapshots and HUD visible to the opponent.

This result covers the exact pinned Chromium, operating system, renderer, and profile used for the
run. A one-second background interval establishes focus and visibility recovery only. It does not
claim long-term intensive throttling, OS suspension, mobile eviction, delayed-network behavior, or
cross-browser support. V4 traces remain disabled because raw admission frames contain the private
resume capability. Only bounded public command, room, and actor summaries are attached on failure.
