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
`cargo-llvm-cov` **0.9.1** and an **80% line floor**, enforced in CI. Current measurements
and candidate evidence are recorded in [verification](verification.md).
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
`check:api`, `check:realtime`, and `check:gameplay` compare generated
contracts without modifying them. Gameplay generation/drift checks cover active v6 and historical
v2/v3; parity tests compare unchanged physical definitions. AJV validates runtime WebSocket payloads. EN/PL locale
parity is tested. ESLint/SonarJS and Knip must report no findings.

Vitest also discovers `web/scripts/*.test.mjs`: generator tests compile representative
TypeScript assignments, API linter tests prove rule enforcement, and Kubernetes checker tests
exercise rendered YAML invariants.
These run in the shared web gate; the coverage floor measures application code in `src/`.

The realtime generator supports explicit string `minLength`/`maxLength`/`pattern` and array
`minItems`/`maxItems`/`uniqueItems`. Bounds must be ordered nonnegative safe integers; patterns
must compile as ECMAScript Unicode regular expressions. Unknown and misplaced keywords fail
closed. Generated TypeScript remains ordinary string/array types; AJV enforces runtime bounds
and patterns. Keep generator tests and actual schema validation alongside contract changes.

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

All activated-v6 Playwright configurations disable framework trace capture. Raw traces can retain
personal `admitted` frames and private resume capabilities before test-side redaction. Failure
screenshots, public reports, and bounded redacted command/snapshot summaries remain available.

### Generated pickup acceptance

The default six scenarios run against random generated maps and rely only on safe openings.
A separate `test:pickups` command uses the explicit read-only operator fixture setup in
[deployment.md](deployment.md#reproduce-a-generated-pickup-round). It fails without
`E2E_PICKUP_FIXTURE=arena-v1-pickups`; it never silently skips. Run both modes against the same
image. CI performs default smoke, recreates the image with the fixture override, then exercises
native keyboard collection, both HUDs, and coexisting old/new bomb ranges. The public fixture
also has a deterministic core simulation trace; concealed data never enters browser assets. The
pickup runner retains bounded test-side action/source summaries but disables per-key DOM/compositor
capture, whose overhead distorts real-time escape on software-rendered Linux. On failure it attaches
redacted command/snapshot summaries; failure screenshots and explicit visual checkpoints remain.

### Complete-match acceptance

`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:matches` runs separately after
the documented read-only fixture setup. It fails without acknowledgement. Two real browsers
walk the core-proven corridor, draw with one shared bomb, complete three round wins, verify
fresh worlds/assignments, withdraw a vote, and consent to rematch over the same sockets.
The same image runs default, pickup, and full-match modes in CI. Retained reports include
redacted public histories; screenshots and Axe cover desktop and 320-pixel final results.

### Reconnect acceptance

`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect` runs two reconnect
scenarios against the same match fixture and uses zero retries. It covers transport restore and
connected takeover with real browsers and sockets. Together with six default, one pickup, and one
complete-match scenario, scenario parity covers ten documented cases. These checks do not by
themselves claim exact-image or hosted-environment acceptance.

## Ordered WebSocket delay

With the documented matches fixture running, use
`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:latency`.
The fixture keeps text/binary frames unchanged, bounds both FIFO queues and tests real native
movement/release through actual page delivery. The [method and reference](game/websocket-latency.md)
define the fixed budget, CPU-affinity profile and evidence limits; twenty observations do not
establish WAN behavior or a tail guarantee. Gameplay traces remain disabled.

## Procedural sound

`xvfb-run -a npm --prefix e2e run test:audio` runs the natural headed Chromium sound scenario
on Linux; macOS needs no Xvfb wrapper. The [sound guide](game/audio.md) records gesture, mute,
visibility and cleanup evidence. CDP context/node observations establish browser processing,
not hardware audibility or listening quality; final context disposal is unit-tested separately.
