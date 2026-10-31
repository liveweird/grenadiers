# Grenadiers browser tests

These Playwright tests treat the deployed app as a black box. They require a complete stack at `http://127.0.0.1:8085`, or at `E2E_BASE_URL`; the test command never starts or mocks a server.

## Arena graphics acceptance

`npm run test:three-arena` builds and serves the separate credential-free Three.js diagnostic.
It checks actual WebGL colour output, all board sizes, billboard variants, visibility resets,
context recovery and teardown at desktop and narrow reduced-motion sizes. The artist atlas
and retained Pixi parity suites validate source artwork separately. Run software WebGL suites
serially; macOS and CI use an explicit shared ANGLE backend and default to one worker.
See the [scenario](scenarios/three-arena.md) and [testing policy](../docs/testing.md).

## Content security policy acceptance

`npm run test:csp` checks real enforced headers, live HTTP/WS and HTTPS/WSS, and blocked
execution/connections/framing. From `e2e/`, build and start an isolated complete app at
`http://127.0.0.1:8086`, leaving the normal game on 8085:

```sh
docker build -t grenadiers-csp-check:local ..
docker run -d --rm --name grenadiers-csp-check --init --read-only --cap-drop ALL \
  --security-opt no-new-privileges:true -p 127.0.0.1:8086:8085 \
  -e ALLOWED_ORIGINS=http://127.0.0.1:8086,https://127.0.0.1:8443 grenadiers-csp-check:local
```

Generate a short-lived local test certificate (OpenSSL is a test prerequisite):

```sh
mkdir -p .playwright/csp-tls
openssl req -x509 -newkey rsa:2048 -nodes -days 2 -subj /CN=localhost \
  -addext subjectAltName=DNS:localhost,IP:127.0.0.1 \
  -keyout .playwright/csp-tls/key.pem -out .playwright/csp-tls/cert.pem
node scripts/local-browser-proxy.mjs --upstream http://127.0.0.1:8086 \
  --https-port 8443 --cert .playwright/csp-tls/cert.pem --key .playwright/csp-tls/key.pem
```

In another terminal, run:

```sh
E2E_CSP_HTTP_URL=http://127.0.0.1:8086 \
E2E_CSP_HTTPS_URL=https://127.0.0.1:8443 npm run test:csp
```

The loopback-only relay preserves Origin and WebSocket upgrades; it is test tooling, not a
public deployment proxy. Only the HTTPS project accepts the local test certificate. Actual
execution fails if `E2E_CSP_HTTPS_URL` is missing. The optional `--policy-file` relay flag is
for pre-header development trials; omit it for final acceptance of the server's own header.
Stop the relay afterward, run `docker stop grenadiers-csp-check`, and remove
`.playwright/csp-tls/` and the test image when no longer needed.
Never commit test keys or record credential-bearing WebSocket frames.

## General suite

Run `npm ci`, `npm run install:browsers`, then `npm test`. Every Playwright test title must
have an exact `## Scenario:` counterpart in the document assigned by
`scripts/scenario-map.mjs`; one document may explicitly cover multiple specs. The
`npm run check:scenarios` gate tests the checker, rejects unmapped documents/specs and duplicate
or computed titles, then enforces the mapped pairing. Also run `npm run typecheck`, `npm run lint`,
and `npm run knip` after editing the suite.
Two-browser responsive scenarios should use `createResponsiveBrowserContexts` from
`tests/support/browser-contexts.ts`. It creates native desktop 1440x1050 and reduced-motion
mobile 320x844 contexts concurrently; pass `{ mobileWidth: 390 }` only for the wider pickup
acceptance layout. Keep specialized context options local to their scenario.
Simple gameplay-suite Playwright wrappers should use `gameplaySuiteConfig` from
`playwright-gameplay-suite.ts` with only their test directory, artifact stem, and any non-default
timeout. Keep specialized projects, servers, launch behavior, and chained fixture configs local.
Conditional `test.skip`, `test.fixme`, and `test.fail` annotations are statically recognized when
their reason is a string literal or an identifier declared as a string; ambiguous overloads fail
with a source location instead of being omitted from parity checks.

The suite covers independent diagnostic sockets and real browser session issuance, including
Origin and cache policy. A gameplay socket scenario proves compatibility-before-claim,
duplicate-owner protection, and sequence replay rejection.

The generated pickup and full-match fixtures run separately from random-map smoke tests.
Start every deterministic fixture from the repository root with the base file first and the exact
seed from the [fixture index](../docs/deployment.md#fixture-index):

```sh
GRENADIERS_FIXTURE_SEED_FILE=./core/tests/fixtures/<exact-name>.seed \
  docker compose -f docker-compose.yaml -f e2e/compose/fixture.yaml \
  up -d --no-build --force-recreate app
```

`GRENADIERS_FIXTURE_SEED_FILE` is required host-side Compose interpolation; it is not passed into
the container or browser. Supply an absolute host file path or a relative path beginning with `./`;
the latter is resolved from the first, root Compose file. An unset or empty selector fails Compose
configuration. The bind disables host-path creation, so a missing source is rejected before the
application starts and is not created as a directory. Keep each suite's separate
`E2E_*_FIXTURE` acknowledgement. Tear down with `docker compose -f docker-compose.yaml down` so
cleanup does not require the selector.
Run the dedicated configurable-controls acceptance with `npm run test:controls`; it needs no
fixture and verifies persisted UI remapping with native keys against a live round.
The reconnect suite reuses the full-match fixture; run it with
`E2E_MATCH_FIXTURE=arena-v1-matches npm run test:reconnect` from this directory.
Every suite disables native tracing because personal gameplay frames contain stable private seat
capabilities. Keep failure screenshots, public HTML reports, explicit compositor checkpoints,
and bounded redacted summaries; never print or attach authentication/resume payloads. The
reconnect probe closes real native transports and authenticates normally without changing
the authoritative simulation clock, state, or outcomes.

The Wall Pass production route runs separately with the mode-0444 seed fixture:
`E2E_WALL_PASS_FIXTURE=arena-v3-wall-pass npm run test:wall-pass`. Start the same application image
with the shared override and `core/tests/fixtures/arena-v2-wall-pass.seed` first. The route uses only
browser controls and read-only socket observations; it does not inject state or send gameplay
commands over the probe.

## Natural headed visibility

With the matches fixture ready, run `E2E_MATCH_FIXTURE=arena-v1-matches xvfb-run -a npm --prefix e2e run test:hardening`
on Linux. The dedicated harness launches fresh Chromium and uses the existing default context
with `connectOverCDP({ noDefaults: true })`; default Playwright focus emulation forces all targets
visible and cannot establish a genuine hidden tab. The scenario observes actual visibility events.
Owned processes/profiles have bounded cleanup; private CDP endpoints and capabilities are not attached.

The Kick source route uses the shared override with the mode-0444
`core/tests/fixtures/arena-v2-kick.seed`:
`E2E_KICK_FIXTURE=arena-v3-kick npm run test:kick`. It uses native controls and read-only public
observations; source checks do not claim native acceptance.

The Punch source route uses the shared override with the mode-0444
`core/tests/fixtures/arena-v2-punch.seed`:
`E2E_PUNCH_FIXTURE=arena-v3-punch npm run test:punch`. It remaps and uses the actual Equipment key,
uses native movement/Bomb input and observes both public recipient streams without injection or
direct gameplay messages. Source checks do not claim native acceptance.

The Bracelet source route uses the shared override with the mode-0444
`core/tests/fixtures/arena-v2-bracelet.seed`:
`E2E_BRACELET_FIXTURE=arena-v3-bracelet npm run test:bracelet`. Two browsers clear opposite
perimeters with ordinary Bomb and movement input, then prove the stationary rival moves exactly
one tile in both recipient streams without an Equipment action, public event or private marker.
Source checks do not claim native acceptance.

The Oil Barrel source route uses the shared override with the read-only
`core/tests/fixtures/arena-v2-oil-barrel.seed`:
`E2E_OIL_BARREL_FIXTURE=arena-v3-oil-barrel npm run test:oil-barrel`. Ordinary Bomb and movement
input proves delayed reveal, the unlit collision edge, safe ignition and persistent burning in both
recipient streams, then reconnects normally. Source checks do not claim native acceptance.
