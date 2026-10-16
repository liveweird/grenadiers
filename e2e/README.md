# Grenadiers browser tests

These Playwright tests treat the deployed app as a black box. They require a complete stack at `http://127.0.0.1:8085`, or at `E2E_BASE_URL`; the test command never starts or mocks a server.

Run `npm ci`, `npm run install:browsers`, then `npm test`. Every `test(...)` title must have an exact `## Scenario:` counterpart under `scenarios/`; `npm run check:scenarios` enforces the pairing. Also run `npm run typecheck`, `npm run lint`, and `npm run knip` after editing the suite.
