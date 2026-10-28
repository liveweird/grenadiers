# Testing and checks

`scripts/gates.sh` is the shared local/CI interface. Install both npm packages with `npm ci`
and use the repository's committed Cargo lockfile. Tests must exercise
behavior and observable contracts rather than merely repeat implementation constants.

The web, E2E-static and browser/atlas gates check the installed direct dependency versions of
the workspaces they use before executing tools. After a manifest/lock update, rerun `npm ci`;
matching committed lockfiles alone do not prove an existing `node_modules` is current.

## Verification policy

Local checks are the primary gate for this single-maintainer repository. A passing local check
does not need a second remote run merely because a branch was pushed, a PR was opened, a merge
was made or a tag was created. Remote checks must add coverage that was not exercised locally,
such as a different operating system or a clean build environment. A Linux container run already
counts as Linux coverage for the behavior it actually executes; a container build alone does not
prove that Rust unit tests ran on Linux.

Use the relevant checks during development and complete the applicable suite before shipping.
Keep a concise record of the tested commit, commands, results, environment and artifact digest.
Reuse successful results only while their tested inputs remain applicable; rerun affected checks
after relevant changes or failures. Documentation-only changes do not require repeating gameplay
acceptance when application inputs are unchanged.

### Focused gameplay feedback

For cached, focused feedback while editing the gameplay client or shared contract corpora, run:

```sh
npm --prefix web test -- src/game/validation/expanded-client-conformance.test.ts src/game/validation/expanded-server-conformance.test.ts src/game/client/client.test.ts src/game/client/client-settings.test.ts src/game/client/locale.test.tsx src/App.test.tsx src/game/arena/GameArena.test.tsx
cargo test -p grenadiers-core --locked 'protocol::expanded_client_'
cargo test -p grenadiers-core --locked 'protocol::expanded_server_contract_tests'
cargo test -p grenadiers-core --locked 'admission_tests'
```

The Rust filters intentionally select only `grenadiers-core`; the `admission_tests` filter covers
the protocol admission module, not server match-owner admission. These commands are an early
feedback loop, not a replacement for the applicable acceptance gates, exhaustive profiles or
coverage floors. See the [historical fast-feedback measurement](history/fast-feedback.md) for the
measured environment, cache conditions and fault-detection evidence.

Parallelize independent checks and gameplay groups using isolated processes, build directories,
Compose projects, ports and Kubernetes namespaces. Reuse compiled dependencies and tooling.
Keep latency tests isolated from competing workloads. Scheduling and caching must not reduce
assertions, exhaustive mask coverage, coverage floors or failure visibility, or introduce retries
that hide failures. Avoid repeated evidence packaging and review of unchanged results.

Concurrent native stacks must use distinct host ports as well as distinct projects/namespaces.
Refuse an occupied port. Bind `kubectl port-forward --address=127.0.0.1` when browser URLs use
IPv4 loopback, and confirm that the intended forwarding process reports that exact listener
before checking HTTP readiness. An IPv6-only forward cannot validate traffic sent to IPv4;
otherwise a test can accidentally exercise another running stack. Preserve pod state and logs
before failure cleanup so routing failures can be distinguished from application failures.
Keep helper scripts unchanged while their acceptance command is running.

The historical verification records below describe the workflow used at the time; they do not
make repeated remote runs a current release requirement.

### Deterministic Solo evaluation

The [CPU evaluation guide](game/cpu-evaluation.md) describes the test-only reference workloads.
Routine Rust tests cover deterministic queued planning and ordinary admission. For isolated
release measurements, run `scripts/measure-game.sh --cpu`; it writes environment, source/binary
identity and structured results under `target/cpu-measurements/`. This mode runs only the CPU
reference matrix, keeping the existing runtime/owner measurements available through the default
command. Fixed input scripts provide regression evidence; they do not replace human playtesting
or establish balanced difficulty. `scripts/measure-game.sh --cpu-rounds` runs the separate
full-round matrix and verifies generated accepted-input replays. Keep timing runs separate
from other checks. Easy has separately labeled `--cpu-easy` and `--cpu-easy-rounds` modes
with the same workloads and independent output directories.

### Hosted workflows

The full `checks` workflow is manual: `gh workflow run ci.yml --ref <branch>`. It remains available
for a clean hosted reproduction, including all exhaustive Rust profiles and browser scenarios.
It does not run automatically for pushes, PRs, merges or tags.

`platform.yml` runs debug and release Rust unit tests and doctests on Ubuntu for relevant PRs.
This covers the platform gap when local Rust tests ran on macOS. It skips only the expensive
exhaustive replay test; local full acceptance and manual full CI retain that test in all three
profiles. The cheap exhaustive catalog/protocol tests remain in the Linux suite. Compiled Cargo
dependencies are cached; test outcomes are always executed. The workflow can also be dispatched
manually when Linux coverage is needed. Equivalent local Linux test execution satisfies the same
requirement without a remote rerun.

The manual full workflow also caches compiled dependencies and its pinned `cargo-llvm-cov` and
`cargo-deny` binaries. Cache restoration never skips a check, and each restored binary must report
its exact required version before use.

The optional `scripts/gates.sh tooling` gate requires ShellCheck 0.11.0 and actionlint 1.7.12 to
already be installed. It installs nothing and checks the repository Bash entry points plus all
GitHub workflows. It intentionally remains outside `all`, `repository`, server, and automatic
platform checks so these developer tools do not become application or platform prerequisites.

## Artist atlas

`scripts/gates.sh art-atlas` runs CLI resource/path checks and the actual Chromium artist
workbench suite. Install both npm workspaces and Playwright Chromium first. The Playwright
server builds the separate diagnostics target before previewing it, including on a clean
checkout. Set `RENDERER_DIAGNOSTICS_BASE_URL` only when deliberately testing an already-built
diagnostics server. The manual full CI includes this gate; `scripts/gates.sh all` retains its
existing Rust/web/E2E-static scope and does not require an installed browser.

The suite checks canonical PNG/JSON import, decoded frame pixels, deterministic export data,
invalid sources, concurrent operations, context restoration, one live preview, texture budgets,
EN/PL errors and desktop/320px reduced-motion layouts. Core unit tests cover frame inventory,
metadata, every extrusion edge/corner, padding, gutters, transparency and texture disposal.
Generated browser proofs live under ignored `e2e/.playwright/art-atlas-test-results/`. The
[artist handoff](../assets/art/README.md) documents template/export commands. The dedicated
`scripts/gates.sh atlas-gameplay` checks the gameplay atlas backend against procedural reference
pixels, by packing the procedural baseline sheet rather than the adopted artist source, and real
WebGL lifecycle behavior with the bundled source. It uses the same fresh diagnostics build and
stores proofs in `e2e/.playwright/atlas-gameplay-test-results/`. See the
[rendering guide](game/art-atlas.md). Artist redraws therefore change no parity expectation.
The same item-shard step runs the DPR 1/2 terrain-pixel suite and retains failures under
`e2e/.playwright/terrain-pixel-test-results/`; the separate six-cell renderer lifecycle suite
remains opt-in.

## Rust

Use unit tests for pure validation/serialization and Tower oneshot tests for HTTP behavior.
Real WebSocket tests bind ephemeral listeners and connect over actual sockets with allowed
Origin headers. Cover readiness failure, invalid payloads, denied origins, connection limits,
and API/SPA separation. Keep tests independent and use bounded timeouts rather than sleeps.

Register simulation child tests in `core/src/game/simulation_test_modules.rs`. The test-only
include keeps the original simulation namespace, private access and full test names; production
module declarations stay in `simulation.rs`. Repository checks also cover every immediate
`e2e/*-tests` tree, including nested typed sources. `scripts/source-size.test.mjs` exercises the
400-line boundary, future-directory discovery and unchanged exclusions through the real checker.

`cargo test --workspace --locked` runs the debug suite; `cargo test --workspace --release --locked`
runs the same behavior in the production profile. The server gate requires both: debug assertions
must never perform required mutations. The test profile uses optimization level 1 while retaining
debug assertions and integer overflow checks; exhaustive debug and instrumented coverage runs use
that same assertion-enabled profile. Clippy denies warnings. Coverage uses
`cargo-llvm-cov` **0.9.1** and a **90% line floor**, enforced in CI. Current measurements
and candidate evidence are recorded in [verification](verification.md).
Install with `cargo install cargo-llvm-cov --version 0.9.1 --locked`, then
`rustup component add llvm-tools-preview` and `scripts/gates.sh coverage`.
Stable branch coverage equivalent to Flow's Kover gate is not configured.

Cargo coverage excludes test-only `test_support.rs` and `*_test_support.rs` helpers from the
denominator using `scripts/coverage-exclusions.txt`. Standalone and parallel reporting share
that policy; helper code still executes and the 90% floor and exhaustive acceptance remain.

`scripts/gates.sh server-parallel` combines the server and coverage checks with four local
workers by default. It builds each profile once, runs ordinary tests once per profile, and
executes the exhaustive replay ranges using isolated workers. Set
`GRENADIERS_LOCAL_SERVER_WORKERS=2` to adjust concurrency (supported range: 1–8). It requires a
clean committed worktree and retains the same assertions, complete mask catalog and coverage
floor. The separate `server` and `coverage` commands remain available as the serial fallback.
Do not run latency acceptance alongside these CPU-intensive checks.

The parallel runner checks Cargo-owned coverage-cache cleanup before expensive execution and
fails immediately if Cargo rejects an existing cache. Preserve the complete cache metadata when
restoring a cache; do not bypass an invalid ownership marker. Instrumented workspace executables
come from one Cargo JSON build stream. Reporting copies that exact complete set into a fresh
private target and uses only the ordinary-plus-exhaustive profiles from this run. The official
reporter, exclusions and 90% floor remain unchanged; retained scratch includes execution evidence
before reporting so a report failure does not erase completed-test provenance.

`api/fixtures/gameplay-expanded-server-conformance.json` is checked against actual Rust-produced
v8–v11 admissions/snapshot pairs and the corresponding browser validators. It covers retained
freeze/teleport/Oil differences and recipient identity/privacy, alongside structural/semantic
negative controls; it does not replace existing control/error-frame conformance tests.
Run `cargo test -p grenadiers-core --locked expanded_server_contract_tests` and
`npm --prefix web test -- src/game/validation/expanded-server-conformance.test.ts` for focused feedback.

The repository gate also runs the hosted-workflow contract tests. Before the parallel server
runner starts, its regression suite checks scheduling, failure propagation and real
`cargo-llvm-cov` binary/profile isolation using a disposable crate. Run that suite separately
with `python3 -I -B scripts/local-server-parallel.test.py`; it requires the coverage tooling
listed above. Coverage tools are not prerequisites for repository, web or E2E-static checks.

The optional full CI derives the exhaustive server item-mask partition matrix from the checked
Rust catalog; the
workflow and JavaScript helpers do not duplicate item bits, cardinality, or ranges. Each partition
runs the complete debug, release, build, and 90%-floor coverage gates with a 20-minute limit and
emits source-bound debug, release, and coverage summaries. The stable `server` job accepts only the
complete, disjoint set of successful partition artifacts. With both partition variables absent,
local debug, release, and coverage commands still execute every mask. Clean Git worktrees report
their HEAD; dirty worktrees and archives without `.git` honestly report a null source. CI requires
and verifies an exact clean checked-out 40-hex source.

The CI-only selector pair is
`GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX` and
`GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT`. Supplying only one, a malformed value, or a
count that differs from the checked plan fails before mask execution. Profile and source context
are assigned by `scripts/gates.sh`; callers must not infer or relabel coverage output.

The shared expanded client corpus is `api/fixtures/gameplay-expanded-client-conformance.json`.
Rust parsers and schemas for v8–v11 and the browser consume the same valid/invalid cases.
`expanded_client_contract_tests` and `expanded_client_mutation_tests` add deterministic malformed
field, byte-boundary and rejection checks; these are bounded regressions, not a general fuzzing
engine. Run `cargo test -p grenadiers-core --locked expanded_client_` and
`npm --prefix web test -- src/game/validation/expanded-client-conformance.test.ts` for focused feedback.

There is no durable schema yet, so no Testcontainers suite is needed. When persistence is
added, run migrations against an isolated real PostgreSQL instance and test query behavior,
constraints, and rollback; never use the developer's database as a test fixture.

The application/tooling projects retain strict mode and exact optional properties. The
[compiler-option assessment](history/typescript-policy.md) explains the indexed-access and
E2E migration disposition; diagnostic totals are not defect counts.

## Web

Vitest, happy-dom, and Testing Library exercise UI and transport behavior. Co-locate tests
as `*.test.ts` or `*.test.tsx`. Mock the browser boundary for component tests; restore mocks,
listeners, and timers after each test. Keep default test isolation. The shared gate shuffles
test order and enforces coverage thresholds declared in `web/vite.config.ts`.
Test-only `src/**/testSupport.ts` fixtures execute with the tests but are excluded from the
application coverage denominator.
After a fresh measurement with those helpers excluded, the web floors are 90% lines and
88% statements, functions and branches. Thresholds must not be lowered to accommodate changes.

Ordinary `npm --prefix web run build` also checks aggregate gzip sizes against
`web/bundle-budget.json`: 400,000 bytes of JavaScript and 20,000 bytes of CSS. The CSS ceiling follows the
[measured stylesheet reduction](history/mantine-css.md); review bundle contributors before changing either budget. Images,
source maps and diagnostics/workbench builds are outside this budget. The checker reports
the largest contributors on failure; Vite's existing per-chunk warning remains visible.

`lint:api` checks the fixed OpenAPI file with Spectral's programmatic API and
`api-guidelines/spectral.yaml`; it retains the standard and custom rules without the CLI's
glob dependency. Parser/load errors and error-severity diagnostics fail the command.
`check:api`, `check:realtime`, and `check:gameplay` compare generated
contracts without modifying them. Gameplay generation/drift checks cover active v14 and historical
v2–v13; parity tests compare unchanged physical definitions. AJV validates runtime WebSocket payloads. EN/PL locale
parity is tested. ESLint/SonarJS and Knip must report no findings.

Live gameplay-v14 and realtime browser consumers use generated standalone Ajv validators;
archived validators retain their own compiler/schema checks. Run
`npm --prefix web run gen:validators` after changing a live schema or its generation tools.
`check:validators` regenerates in memory and checks both ESM and declarations before production
builds. Parity tests compare complete ordered errors with the original compilation routes;
generated output rejects executable imports/evaluation while allowing inert helper metadata.
Generated MJS is excluded narrowly from handwritten lint rules; its adjacent declarations are
checked by TypeScript/drift checks and ignored only by Knip's unused-file discovery. Handwritten
tests and helpers remain in scope; see [ADR 0022](decisions/0022-standalone-browser-validation.md).
The compatible enforced browser policy uses Pixi's static implementations and retains Mantine's
inline styles. Run the separate HTTP/WS and HTTPS/WSS `test:csp` acceptance with the documented
[isolated backend, certificate and relay setup](../e2e/README.md#content-security-policy-acceptance).
That suite requires the real server header without trial injection; diagnostic suites also observe
policy violations before navigation through drawing, context restoration and artist export.

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
and spec titles must agree verbatim. Run E2E static checks locally before release. The optional manual full workflow also runs
those checks and Compose scenarios; automatic PR checks add distinct Linux Rust coverage.

The default suite uses one worker in CI and two locally; `E2E_WORKERS` explicitly overrides
that choice. `E2E_CAPTURE_PROFILE=1` enables the opt-in capture workload profiler and turns off
the default configuration's failure screenshots; see the
[capture profile](game/renderer-diagnostics.md#original-reconnect-capture-profile). Each multiplayer case already owns two browser contexts. Serial CI execution
keeps software WebGL within the two-player reference on two-core runners; specialized suites
also use one worker. Existing deadlines and assertions remain strict. Frozen-result checks
select retained snapshots by round and revision so automatic progression cannot change the
round being compared.

Install cargo-deny with `cargo install cargo-deny --version 0.20.2 --locked` for
`scripts/gates.sh security`. Rust advisory/license/source checks and full npm audits run
separately so external advisory outages do not obscure unit-test results. Document any
reviewed exception with its reason and removal condition; never add blanket suppressions.

All Playwright configurations disable framework trace capture. Raw traces can retain
personal `admitted` frames and private resume capabilities before test-side redaction. Failure
screenshots, public reports, and bounded redacted command/snapshot summaries remain available.

## Browser acceptance index

This index describes the currently configured suites, not whether a particular candidate, image,
commit, or release has passed them. Record current execution evidence in
[verification](verification.md) and release status in [releases](releases.md); do not carry a
historical “pending” or “passed” label forward without a new run. The former narrative is retained
unchanged as a [historical testing record](history/testing.md). Its deleted per-feature Compose
filenames and commands describe the v0.3.63-era procedure; use the current deployment guide and
shared override for present-day runs.

The package scripts and setup procedures in the [E2E guide](../e2e/README.md) and
[deployment guide](deployment.md) are authoritative. The linked scenario documents define the
assertions and evidence for each suite.

### Browser evidence and safety rules

- Run black-box gameplay suites against the actual stack; do not replace the backend with mocks.
  Fixture-backed suites require their exact acknowledgement variable and documented read-only
  fixture or Compose override, and must fail rather than silently skip when setup is absent. Wait
  for readiness after recreating a service. Use the same image when a verification plan compares
  default and fixture-backed modes. Concealed fixture data remains server-side and must not enter
  browser assets.
- Production-seed item scenarios use ordinary UI/native input and their documented public or
  read-only observations. Do not add state injection, direct gameplay messages, seed searches,
  retries, longer budgets, or private markers merely to make acceptance pass. Any deliberately
  different diagnostic mechanism belongs in its scenario document.
- Framework traces stay disabled in every Playwright configuration because raw admitted frames
  can contain private seat/resume capabilities. Never attach or print those capabilities. Retain
  only the suite's allowed failure screenshots, public HTML reports, explicit visual checkpoints,
  and bounded redacted command/snapshot or diagnostic summaries. Preserve suite-specific capture
  restrictions where compositor, DOM, screenshot, or trace overhead would disturb timing.
- Preserve recipient-specific privacy checks through reconnect: hidden actors, bombs, events,
  cues, and capabilities must not leak through observations or retained artifacts. Reconnect must
  authenticate normally and must not synthesize replay or refresh an effect deadline.
- Where a scenario requires visual/localization coverage, keep the English desktop and Polish
  narrow-screen view, including the documented 320px (or explicit 390px) viewport and reduced
  motion. Do not replace pixel/compositor assertions with source inspection. Keep accessibility,
  horizontal-fit, screenshot naming, and timing placement defined by the scenario.

### Configured suites

| Suite or command | Fixture or deployment | Unique scope |
| --- | --- | --- |
| `npm --prefix e2e test` | [Default Compose stack](deployment.md#docker-compose) | [Board presets](../e2e/scenarios/board-sizes.md), default generated-map smoke, session, room, realtime, gameplay, settings, turning, and rendered-duel contracts; see the mapped [scenario index](../e2e/scenarios/). |
| `npm --prefix e2e run test:csp` | [Isolated HTTP/HTTPS relay setup](../e2e/README.md#content-security-policy-acceptance) | Enforced server CSP over HTTP/WS and HTTPS/WSS, including blocked execution, connections, and framing. |
| <a id="generated-pickup-acceptance"></a>`npm --prefix e2e run test:pickups` | [Generated pickup fixture](deployment.md#reproduce-a-generated-pickup-round) | Native collection, both HUDs, and captured old/new bomb ranges; retains the [pickup scenario's](../e2e/scenarios/pickups.md) timing-safe redacted evidence. |
| <a id="expanded-pickup-acceptance"></a>`npm --prefix e2e run test:heart` | [Heart fixture](deployment.md#reproduce-heart-collection-and-protection) | Natural collection, one protected hit, later lethal damage, and both localized perspectives. |
| `npm --prefix e2e run test:shield` | [Shield fixture](deployment.md#reproduce-opt-in-shield-protection) | Shield-only lobby configuration, protection expiry, later lethal damage, and both localized perspectives. |
| `npm --prefix e2e run test:controls` | [Live-stack setup](../e2e/README.md#general-suite) | Persisted UI remapping exercised with native keys in a real round. |
| <a id="complete-match-acceptance"></a>`npm --prefix e2e run test:matches` | [Complete-match fixture](deployment.md#reproduce-a-complete-classic-match) | Three round wins, fresh worlds/assignments, vote withdrawal, same-socket rematch, Axe, and localized final results. |
| <a id="reconnect-acceptance"></a>`npm --prefix e2e run test:reconnect` | [Complete-match fixture](deployment.md#reproduce-a-complete-classic-match) | Transport restoration and connected takeover with normal authentication, real browsers/sockets, and zero retries. |
| `npm --prefix e2e run test:hardening` | [Natural headed visibility setup](../e2e/README.md#natural-headed-visibility) | Real visibility events in owned headed Chromium without Playwright focus emulation. |
| <a id="ordered-websocket-delay"></a>`npm --prefix e2e run test:latency` | [Complete-match fixture](deployment.md#reproduce-a-complete-classic-match) | Bounded ordered text/binary frame delay and native movement/release; the [method and limits](../e2e/scenarios/websocket-latency.md) do not claim WAN behavior or a tail guarantee. |
| <a id="procedural-sound"></a>`npm --prefix e2e run test:audio` | Linux needs `xvfb-run -a` before this command; macOS needs no wrapper. See the [sound guide](game/audio.md) and [scenario](../e2e/scenarios/audio.md). | Gesture, mute, visibility, and cleanup through browser processing; it does not claim hardware audibility or listening quality. |
| `npm --prefix e2e run test:golden-flame` | [Golden Flame fixture](deployment.md#reproduce-opt-in-golden-flame-range) | Captured maximum range, exact deadline, reconnect, and restored base range. |
| `npm --prefix e2e run test:short-fuse` | [Short Fuse fixture](deployment.md#reproduce-opt-in-short-fuse-timing) | Captured 150-tick fuse, natural detonation, reconnect without extension, exact expiry, and later base fuse. |
| `npm --prefix e2e run test:short-flame` | [Short Flame fixture](deployment.md#reproduce-opt-in-short-flame-range) | Range-one fire boundary, independent fuse timing, reconnect, exact expiry, and later base-range damage. |
| `npm --prefix e2e run test:long-fuse` | [Long Fuse fixture](deployment.md#reproduce-opt-in-long-fuse-timing) | Captured 240-tick timing from a 120-tick base, exact expiry, and restored ordinary timing. |
| `npm --prefix e2e run test:reverse-controls` | [Reverse Controls fixture](deployment.md#reproduce-opt-in-reverse-controls-movement) | One held raw input across collection and exact expiry, with captures outside the timed window. |
| <a id="diarrhea-production-seed-scenario"></a>`npm --prefix e2e run test:diarrhea` | [Diarrhea fixture](deployment.md#reproduce-opt-in-diarrhea-automatic-placement) | Server-owned next-tick placement without another browser action, natural death, and clean next round. |
| <a id="slow-foot-production-seed-scenario"></a>`npm --prefix e2e run test:slow-foot` | [Slow-foot fixture](deployment.md#reproduce-opt-in-slow-foot-movement) | Authoritative unclamped half-rate/base-rate movement windows, exact timers, and key release. |
| <a id="piercing-bomb-production-seed-scenario"></a>`npm --prefix e2e run test:piercing-bomb` | [Piercing Bomb fixture](deployment.md#reproduce-opt-in-piercing-bomb-penetration) | Captured placement, two consecutive burned blocks, charge consumption, and a following ordinary bomb. |
| <a id="power-bomb-production-seed-scenario"></a>`npm --prefix e2e run test:power-bomb` | [Power Bomb fixture](deployment.md#reproduce-opt-in-power-bomb-hard-wall-traversal) | Hard-wall traversal, Manhattan-footprint boundary, charge consumption, and a fresh ordinary placement. |
| <a id="remote-bomb-production-seed-scenario"></a>`npm --prefix e2e run test:remote-bomb` | [Remote Bomb fixture](deployment.md#reproduce-opt-in-remote-bomb-manual-detonation) | Same-ID survival beyond the ordinary fuse, manual detonation without placement, and later ordinary placement. |
| <a id="landmine-production-seed-scenario"></a>`npm --prefix e2e run test:landmine` | [Landmine fixture](deployment.md#reproduce-opt-in-landmine-contact-detonation) | Same-ID contact mine persistence, safe overlap exit, keyboard re-entry detonation, and recipient parity. |
| <a id="invisibility-cloak-production-seed-scenario"></a>`npm --prefix e2e run test:invisibility-cloak` | [Invisibility Cloak fixture](deployment.md#reproduce-opt-in-invisibility-cloak-recipient-privacy) | Exact recipient-local actor privacy, compositor pixels, reconnect without replay/refresh, and exact reveal. |
| <a id="stopwatch-production-seed-scenario"></a>`npm --prefix e2e run test:stopwatch` | [Stopwatch fixture](deployment.md#reproduce-opt-in-stopwatch-action-and-fuse-pausing) | Two pause windows, blocked/consumed actions, held-input thaw, accumulated fuse pause, and shared deadlines. |
| <a id="invisible-bombs-production-seed-scenario"></a>`npm --prefix e2e run test:invisible-bombs` | [Invisible Bombs fixture](deployment.md#reproduce-opt-in-invisible-bombs-privacy) | Recipient-local live-bomb/event privacy, truthful capacity/flames, reconnect without replay, expiry, and reveal. |
| <a id="teleporter-production-seed-scenario"></a>`npm --prefix e2e run test:teleporter` | [Teleporter fixture](deployment.md#reproduce-opt-in-teleporter-swaps) | Safe atomic swap, shared marker, newer reconnect admission, and unsafe consumption without movement. |
| <a id="wall-pass-production-seed-scenario"></a>`npm --prefix e2e run test:wall-pass` | [Wall Pass fixture](deployment.md#reproduce-opt-in-wall-pass-movement) | Intact-block traversal/rendering, rejected planting without allocation, reconnect inside, legal exit/placement, and localized captures. |
| <a id="cpu-item-source-scenario"></a>`npm --prefix e2e run test:cpu-item` | [Shared Compose fixture](../e2e/compose/fixture.yaml), [`arena-v2-cpu-wall-pass.seed`](../core/tests/fixtures/arena-v2-cpu-wall-pass.seed), `E2E_CPU_ITEM_FIXTURE=arena-v2-cpu-wall-pass`, and [scenario](../e2e/scenarios/cpu-item.md) | Real CPU Wall Pass collection with all items enabled, public inventory and rendered gear; private AI randomness stays independent. |
| <a id="kick-source-scenario"></a>`npm --prefix e2e run test:kick` | [Shared Compose fixture](../e2e/compose/fixture.yaml), exact [`arena-v2-kick.seed`](../core/tests/fixtures/arena-v2-kick.seed), and [scenario](../e2e/scenarios/kick.md) | Same bomb identity/fuse across native kick movement and the retained pre-input timeline. |
| <a id="punch-source-scenario"></a>`npm --prefix e2e run test:punch` | [Shared Compose fixture](../e2e/compose/fixture.yaml), exact [`arena-v2-punch.seed`](../core/tests/fixtures/arena-v2-punch.seed), and [scenario](../e2e/scenarios/punch.md) | Remapped Equipment input, wrapped bomb movement, action-ID successor, public parity, and natural detonation. |
| <a id="bracelet-source-scenario"></a>`npm --prefix e2e run test:bracelet` | [Shared Compose fixture](../e2e/compose/fixture.yaml), exact [`arena-v2-bracelet.seed`](../core/tests/fixtures/arena-v2-bracelet.seed), and [scenario](../e2e/scenarios/bracelet.md) | One-tile stationary-rival shove with no victim command, equipment action, public event, or private marker. |
| <a id="oil-barrel-production-seed-scenario"></a>`npm --prefix e2e run test:oil-barrel` | [Oil Barrel fixture](deployment.md#reproduce-opt-in-oil-barrel-reveal-and-persistent-fire) | Pending privacy, unlit collision, legal ignition/escape, finite and persistent fire, rendering, and reconnect retention. |
| `npm --prefix e2e run test:renderer-diagnostics` | [Renderer lifecycle scenario](../e2e/scenarios/renderer-diagnostics.md) | Six real Pixi lifecycle cells, cross-origin blocking, accessibility/fit, capture stability, and bounded cleanup diagnostics. |
| `npm --prefix e2e run test:terrain-pixels` | [Terrain pixel scenario](../e2e/scenarios/terrain-pixels.md) | Exact cached/uncached Pixi terrain parity across DPRs, repaint/reset, and context restoration. |
| `scripts/gates.sh art-atlas` / `npm --prefix e2e run test:art-atlas` | [Artist atlas](#artist-atlas) | Canonical import/export, decoded pixels, texture lifecycle/budgets, localization, and responsive reduced-motion layouts. |
| `scripts/gates.sh atlas-gameplay` / `npm --prefix e2e run test:atlas-gameplay` | [Atlas gameplay scenario](../e2e/scenarios/atlas-gameplay.md) | Procedural-reference atlas parity and real bundled WebGL lifecycle behavior. |
