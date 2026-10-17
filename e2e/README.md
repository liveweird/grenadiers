# Grenadiers browser tests

These Playwright tests treat the deployed app as a black box. They require a complete stack at `http://127.0.0.1:8085`, or at `E2E_BASE_URL`; the test command never starts or mocks a server.

Run `npm ci`, `npm run install:browsers`, then `npm test`. Every `test(...)` title must have an exact `## Scenario:` counterpart under `scenarios/`; `npm run check:scenarios` enforces the pairing. Also run `npm run typecheck`, `npm run lint`, and `npm run knip` after editing the suite.

The suite covers independent diagnostic sockets and real browser session issuance, including
Origin and cache policy. A gameplay socket scenario proves compatibility-before-claim,
duplicate-owner protection, and sequence replay rejection. Rendered duel scenarios follow
with their implemented milestone.

The generated pickup and full-match fixtures run separately from random-map smoke tests.
Run the dedicated configurable-controls acceptance with `npm run test:controls`; it needs no
fixture and verifies persisted UI remapping with native keys against a live round.
The reconnect suite reuses the full-match fixture; run it with
`E2E_MATCH_FIXTURE=arena-v1-matches npm run test:reconnect` from this directory.
Every suite disables native tracing because v4 personal frames contain stable private seat
capabilities. Keep failure screenshots, public HTML reports, explicit compositor checkpoints,
and bounded redacted summaries; never print or attach authentication/resume payloads. The
reconnect probe closes real native transports and authenticates normally without changing
the authoritative simulation clock, state, or outcomes.

## Natural headed visibility

With the matches fixture ready, run `E2E_MATCH_FIXTURE=arena-v1-matches xvfb-run -a npm --prefix e2e run test:hardening`
on Linux. The dedicated harness launches fresh Chromium and uses the existing default context
with `connectOverCDP({ noDefaults: true })`; default Playwright focus emulation forces all targets
visible and cannot establish a genuine hidden tab. The scenario observes actual visibility events.
Owned processes/profiles have bounded cleanup; private CDP endpoints and capabilities are not attached.
