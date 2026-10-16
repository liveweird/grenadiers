# Testing and checks

`scripts/gates.sh` is the shared local/CI interface. Install both npm packages with `npm ci`
and use the repository's committed Cargo lockfile. Tests must exercise
behavior and observable contracts rather than merely repeat implementation constants.

## Rust

Use unit tests for pure validation/serialization and Tower oneshot tests for HTTP behavior.
Real WebSocket tests bind ephemeral listeners and connect over actual sockets with allowed
Origin headers. Cover readiness failure, invalid payloads, denied origins, connection limits,
and API/SPA separation. Keep tests independent and use bounded timeouts rather than sleeps.

`cargo test --workspace --locked` runs the suite. Clippy denies warnings. Coverage uses
`cargo-llvm-cov` **0.9.1** and an initial **80% line floor**, enforced in CI. This initial floor
is a policy target pending the first runnable Rust measurement, not a measured claim.
Install with `cargo install cargo-llvm-cov --version 0.9.1 --locked`, then
`rustup component add llvm-tools-preview` and `scripts/gates.sh coverage`.
Stable branch coverage equivalent to Flow's Kover gate is not configured.

There is no durable schema yet, so no Testcontainers suite is needed. When persistence is
added, run migrations against an isolated real PostgreSQL instance and test query behavior,
constraints, and rollback; never use the developer's database as a test fixture.

## Web

Vitest, happy-dom, and Testing Library exercise UI and transport behavior. Co-locate tests
as `*.test.ts` or `*.test.tsx`. Mock the browser boundary for component tests; restore mocks,
listeners, and timers after each test. Keep default test isolation. The shared gate shuffles
test order and enforces coverage thresholds declared in `web/vite.config.ts`.

`lint:api` checks the fixed OpenAPI file with Spectral's programmatic API and
`api-guidelines/spectral.yaml`; it retains the standard and custom rules without the CLI's
glob dependency. Parser/load errors and error-severity diagnostics fail the command.
`check:api` and `check:realtime` compare generated
contracts without modifying them. AJV validates runtime WebSocket payloads. EN/PL locale
parity is tested. ESLint/SonarJS and Knip must report no findings.

Vitest also discovers `web/scripts/*.test.mjs`: generator tests compile representative
TypeScript assignments, API linter tests prove rule enforcement, and Kubernetes checker tests
exercise rendered YAML invariants.
These run in the shared web gate; the coverage floor measures application code in `src/`.

## Browser and security

`e2e/` is a separate Playwright package against the actual running stack. Install Chromium
with `npm --prefix e2e run install:browsers`, start Compose, then run
`scripts/gates.sh browser`. No mocked backend belongs in blackbox tests. Scenario documents
and spec titles must agree verbatim. E2E static checks run on every pull request; browser
smoke runs in CI against Compose.

Install cargo-deny with `cargo install cargo-deny --version 0.20.2 --locked` for
`scripts/gates.sh security`. Rust advisory/license/source checks and full npm audits run
separately so external advisory outages do not obscure unit-test results. Document any
reviewed exception with its reason and removal condition; never add blanket suppressions.
