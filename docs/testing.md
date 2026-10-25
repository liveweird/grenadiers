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

Parallelize independent checks and gameplay groups using isolated processes, build directories,
Compose projects, ports and Kubernetes namespaces. Reuse compiled dependencies and tooling.
Keep latency tests isolated from competing workloads. Scheduling and caching must not reduce
assertions, exhaustive mask coverage, coverage floors or failure visibility, or introduce retries
that hide failures. Avoid repeated evidence packaging and review of unchanged results.

The historical verification records below describe the workflow used at the time; they do not
make repeated remote runs a current release requirement.

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

There is no durable schema yet, so no Testcontainers suite is needed. When persistence is
added, run migrations against an isolated real PostgreSQL instance and test query behavior,
constraints, and rollback; never use the developer's database as a test fixture.

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
`web/bundle-budget.json`: 400,000 bytes of JavaScript and 40,000 bytes of CSS. These initial
ceilings come from a measured build; review bundle contributors before changing them. Images,
source maps and diagnostics/workbench builds are outside this budget. The checker reports
the largest contributors on failure; Vite's existing per-chunk warning remains visible.

`lint:api` checks the fixed OpenAPI file with Spectral's programmatic API and
`api-guidelines/spectral.yaml`; it retains the standard and custom rules without the CLI's
glob dependency. Parser/load errors and error-severity diagnostics fail the command.
`check:api`, `check:realtime`, and `check:gameplay` compare generated
contracts without modifying them. Gameplay generation/drift checks cover active v11 and historical
v2–v10; parity tests compare unchanged physical definitions. AJV validates runtime WebSocket payloads. EN/PL locale
parity is tested. ESLint/SonarJS and Knip must report no findings.

Live gameplay-v11 and realtime browser consumers use generated standalone Ajv validators;
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

### Generated pickup acceptance

The default eight scenarios run against random generated maps and rely only on safe openings.
A separate `test:pickups` command uses the explicit read-only operator fixture setup in
[deployment.md](deployment.md#reproduce-a-generated-pickup-round). It fails without
`E2E_PICKUP_FIXTURE=arena-v1-pickups`; it never silently skips. Run both modes against the same
image. CI performs default smoke, recreates the image with the fixture override, then exercises
native keyboard collection, both HUDs, and coexisting old/new bomb ranges. The public fixture
also has a deterministic core simulation trace; concealed data never enters browser assets. The
pickup runner retains bounded test-side action/source summaries but disables per-key DOM/compositor
capture, whose overhead distorts real-time escape on software-rendered Linux. On failure it attaches
redacted command/snapshot summaries; failure screenshots and explicit visual checkpoints remain.

### Expanded pickup acceptance

Heart and Shield each have a separate production-seed scenario with ordinary movement/bombs:
`E2E_HEART_FIXTURE=arena-v2-heart npm --prefix e2e run test:heart` and
`E2E_SHIELD_FIXTURE=arena-v2-shield npm --prefix e2e run test:shield`.
Use the matching Compose override and external mode-0444 fixture; wait for readiness after
recreating the service. Heart retains defaults; Shield explicitly enables only Shield through
the lobby. Tests observe both perspectives, capture desktop/Polish narrow-screen artwork,
and prove protection ends before a later lethal blast. See [deployment](deployment.md).

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
connected takeover with real browsers and sockets. Scenario parity covers every documented case
across all Playwright suites, not only these. These checks do not by
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

Golden Flame has a separate production-seed scenario:
`E2E_GOLDEN_FLAME_FIXTURE=arena-v2-golden-flame npm --prefix e2e run test:golden-flame`.
Use `docker-compose.golden-flame.yaml` with the same image and wait for readiness after
recreating the service. It checks generated collection, captured maximum range, exact deadline,
reconnect and restored base range. See [deployment](deployment.md#reproduce-opt-in-golden-flame-range).

Short Fuse has a separate production-seed scenario:
`E2E_SHORT_FUSE_FIXTURE=arena-v2-short-fuse npm --prefix e2e run test:short-fuse`.
Use `docker-compose.short-fuse.yaml` with the same image and wait for readiness after
recreation. The scenario selects a five-second base fuse, verifies a captured 150-tick
shortened fuse on both peers and natural detonation, reconnects without extending the
effect, and verifies a later ordinary 300-tick bomb after expiry.
See [deployment](deployment.md#reproduce-opt-in-short-fuse-timing).

Short Flame has a separate production-seed scenario:
`E2E_SHORT_FLAME_FIXTURE=arena-v2-short-flame npm --prefix e2e run test:short-flame`.
Use `docker-compose.short-flame.yaml` with the same image and wait for readiness after
recreation. It verifies range-one captures and actual fire stopping before the second cell
of a known clear ray, independent ordinary fuse timing, reconnect, exact expiry and later
base-range damage. See [deployment](deployment.md#reproduce-opt-in-short-flame-range).

Long Fuse has a separate production-seed scenario:
`E2E_LONG_FUSE_FIXTURE=arena-v2-long-fuse npm --prefix e2e run test:long-fuse`.
Use `docker-compose.long-fuse.yaml` with the same image. It verifies generated collection,
captured 240-tick timing from a 120-tick base, exact expiry, and restored ordinary timing.
See [deployment](deployment.md#reproduce-opt-in-long-fuse-timing).

Reverse Controls has a separate production-seed scenario:
`E2E_REVERSE_CONTROLS_FIXTURE=arena-v2-reverse-controls npm --prefix e2e run test:reverse-controls`.
Use `docker-compose.reverse-controls.yaml` with the same image. It holds one raw east input across
collection and the exact 600-tick expiry, checks both peers, and captures screenshots only outside
the timed window. See [deployment](deployment.md#reproduce-opt-in-reverse-controls-movement).

## Diarrhea production-seed scenario

Run `E2E_DIARRHEA_FIXTURE=arena-v2-diarrhea npm --prefix e2e run test:diarrhea` with
`docker-compose.diarrhea.yaml`. The scenario enables only Diarrhea, reveals and collects the
seeded cell (3,1) pickup, verifies the server-owned next-tick ordinary bomb without another
browser bomb action, and observes natural death and a clean next round. Screenshots are taken
before collection so capture time cannot change the timed behavior.

## Slow-foot production-seed scenario

Run `E2E_SLOW_FOOT_FIXTURE=arena-v2-slow-foot npm --prefix e2e run test:slow-foot` with
`docker-compose.slow-foot.yaml`. The scenario enables only Slow foot and captures both localized
views before collection. It measures one stack's half-rate movement and restored base-rate
movement from authoritative snapshots in unclamped corridor windows, checks exact timers on both
peers, and verifies key release across later frames. Native execution remains a candidate gate.

## Piercing Bomb production-seed scenario

Use `docker-compose.piercing-bomb.yaml` with the v2 seed fixture and
`E2E_PIERCING_BOMB_FIXTURE=arena-v2-piercing-bomb npm --prefix e2e run test:piercing-bomb`.
The two-peer scenario verifies natural collection, localized armed presentation, captured Piercing
placement, two consecutive burned blocks, charge consumption, and the following ordinary bomb.
It is a native browser acceptance check and remains pending for the local candidate.

## Power Bomb production-seed scenario

Use `docker-compose.power-bomb.yaml` with the v2 seed fixture and
`E2E_POWER_BOMB_FIXTURE=arena-v2-power-bomb npm --prefix e2e run test:power-bomb`.
The two-peer scenario verifies natural collection, localized armed presentation, a captured
Power placement, fire and block destruction beyond a retained hard wall, a safe cell outside the
Manhattan footprint, charge consumption, and a fresh ordinary placement fenced from both peers'
earlier records. It is a native browser acceptance check and remains pending for the candidate.

## Remote Bomb production-seed scenario

Use `docker-compose.remote-bomb.yaml` with the v2 seed fixture and
`E2E_REMOTE_BOMB_FIXTURE=arena-v2-remote-bomb npm --prefix e2e run test:remote-bomb`.
The two-peer scenario reveals and collects the charge at (3,1), plants it there, captures safe
localized armed views, and proves the same Remote ID survives beyond the ordinary 180-tick fuse.
One fresh bomb action detonates it without placement and leaves the observed empty weapon slot
empty. Focused core regressions prove that a held Piercing or Power charge survives the same
trigger path. A fresh-ID Timed Standard bomb then proves ordinary placement and actual escape.
Native execution remains a candidate gate.

## Landmine production-seed scenario

Use `docker-compose.landmine.yaml` with the v2 seed fixture and
`E2E_LANDMINE_FIXTURE=arena-v2-landmine npm --prefix e2e run test:landmine`.
The two-peer scenario reveals and collects the charge at (3,1), places the Contact Standard mine
there, and captures localized armed views before contact. It proves that the same ID survives
beyond the ordinary fuse, that leaving the initial overlap is safe, and that ordinary keyboard
re-entry detonates the mine without another manual action. Both streams must publish the same
detonation and resulting flames. Native execution remains a candidate gate.

## Invisibility Cloak production-seed scenario

Use `docker-compose.invisibility-cloak.yaml` with the v2 seed fixture and
`E2E_INVISIBILITY_CLOAK_FIXTURE=arena-v2-invisibility-cloak npm --prefix e2e run test:invisibility-cloak`.
The two-peer scenario reveals and collects the cloak at (3,1), proves the owner keeps a complete
actor while the opponent receives an exact hidden actor and no collection cue, and compares actual
compositor pixels at (2,1). Opponent reconnect preserves privacy without replay or deadline refresh;
the actor appears at the current cell at collection tick plus 600. Native execution remains a
candidate gate.

## Stopwatch production-seed scenario

Use `docker-compose.stopwatch.yaml` with the v2 seed fixture and
`E2E_STOPWATCH_FIXTURE=arena-v2-stopwatch npm --prefix e2e run test:stopwatch`.
The two-peer scenario reveals both mirrored pickups with ordinary bombs and exercises two
sequential 180-tick holder windows. It proves blocked movement and consumed manual actions,
held-input thaw, one pause per physical update, full-fuse placement during a pause, and natural
detonation after the exact accumulated pause. Both recipient streams must retain the same bomb IDs,
pause facts and deadlines. Native execution remains a candidate gate.

## Invisible Bombs production-seed scenario

Use `docker-compose.invisible-bombs.yaml` with the v2 seed fixture and
`E2E_INVISIBLE_BOMBS_FIXTURE=arena-v2-invisible-bombs npm --prefix e2e run test:invisible-bombs`.
The two-browser route proves recipient-local live-bomb and event privacy, truthful capacity and
public flames, reconnect without replay, exact 600-tick expiry, and current-state reveal of one
surviving bomb. Native acceptance remains pending until this candidate is built and run.

## Teleporter production-seed scenario

Use `docker-compose.teleporter.yaml` with the mode-0444 v2 seed fixture and
`E2E_TELEPORTER_FIXTURE=arena-v2-teleporter npm --prefix e2e run test:teleporter`.
The keyboard-only two-browser route reveals both pickups with ordinary movement and bombs. It
proves a safe atomic swap, a shared new `lastTeleportTick`, position and marker retention from a
strictly newer reconnect admission, and unsafe consumption beside a live bomb without movement or
marker change. The scenario has no seed search, state injection or retries. Native execution
passed fresh 54 Compose plus 54 Kubernetes cases; the factual publication status is recorded in
the [v0.3.42 published delivery](releases.md#v0342-published-prerelease--2026-10-23).

## Wall Pass production-seed scenario

Use `docker-compose.wall-pass.yaml` with the mode-0444 v2 seed fixture and
`E2E_WALL_PASS_FIXTURE=arena-v2-wall-pass npm --prefix e2e run test:wall-pass`.
The keyboard-only two-browser route enables only Wall Pass, reveals and collects the pickup at
`(3,1)`, enters the intact soft block at `(3,2)`, and proves the authoritative block remains opaque
while the actor and equipment cue remain legible. It rejects planting without consuming a bomb ID,
capacity or weapon, clamps west against hard wall `(2,2)`, resumes from a strictly newer admission
inside the block, then exits north without re-entry. A legal post-exit bomb has a later
entity ID, identical on both clients. The rejected action creates no bomb or event and leaves
resources unchanged; focused core tests verify no entity allocation.
English desktop and Polish 320-pixel
reduced-motion views produce the two named full-page captures. The scenario uses no state
injection, direct WebSocket commands, seed search or retry. The sole-item fixture cannot prove
Wall-Pass-to-Heart replacement grace; core and protocol tests provide that coverage. Native
execution completed as a sealed 55 Compose plus 55 Kubernetes matrix on H2 source
`11d0dc7e39e8e2cafcf70c1548be339f52c740ab`. Published source
`c086f719ea470b9fe24963adef84b5e3bdf3e17a` later passed candidate, master and tag checks;
[v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43) publication and exact-image
local promotion are complete. The native result remains attributed to H2 rather than relabeled to
the published source.

### Kick source scenario

Use `docker-compose.kick.yaml` with the mode-0444 v2 seed fixture and
`E2E_KICK_FIXTURE=arena-v2-kick npm --prefix e2e run test:kick`. The scenario uses ordinary UI and
keys, observes public streams, and does not inject state, search seeds, retry, or extend budgets.
At the 2026-10-24 source-gates checkpoint, source `cb70cba891ed972380f30f52133622ce36fd9ee8`
passes all 524,288 masks in debug, release and coverage, 763 core and 210 server tests per profile,
1,059 web tests across 66 files, and 57 static scenarios. Fresh image execution subsequently passes
all 56 Compose plus 56 Kubernetes cases with zero retries or carried results. Audit, independent
result/binding review, durable sealing and post-seal review are clear. Hosted release checks remain
historical checkpoint evidence. Published source `f32fd86d8f0c7c4bebba8fa60d1c66c65d1a174a`
is available as [v0.3.44](https://github.com/liveweird/grenadiers/releases/tag/v0.3.44), and the
accepted image is promoted locally.

### Punch source scenario

Use `docker-compose.punch.yaml` with the mode-0444 repeating-byte-31 v2 seed fixture and
`E2E_PUNCH_FIXTURE=arena-v2-punch npm --prefix e2e run test:punch`. The two-browser route remaps
Equipment to F, enables only Punch, reveals the natural pickup, clears the approved corridor and
moves one ordinary bomb from `(4,1)` to wrapped `(1,1)`. It proves the shared action-ID successor,
same public bomb identity and fuse, no placement/detonation event, recipient parity, safe escape,
and natural detonation using ordinary UI and native keys. It does not inject state, search seeds,
retry, send direct gameplay commands or extend budgets. Native execution is accepted in the scoped
27-case checkpoint (26 Compose plus one Kubernetes case, zero retries); the full 114-case matrix
was not run. Natural-fuse, reconnect and screenshot checks are recorded in
[verification](verification.md#v0345-punch-native-accepted-checkpoint--2026-10-24--27-cases).

### Bracelet source scenario

Use `docker-compose.bracelet.yaml` with the mode-0444 repeating-byte-31 v2 seed fixture and
`E2E_BRACELET_FIXTURE=arena-v2-bracelet npm --prefix e2e run test:bracelet`. Two browser peers use
ordinary movement and Bomb input to clear opposite perimeter routes, collect Bracelet naturally,
and stage the players at `(10,1)` and `(11,1)`. Forward movement shoves the stationary second player
exactly one tile to `(12,1)` in both public streams. The scenario asserts no victim command,
Equipment action, public event or private cooldown/report field, then captures English desktop and
Polish 320-pixel reduced-motion views. It does not inject state, search seeds, retry, send direct
gameplay commands or extend budgets. Bracelet acceptance passed in v0.3.46; the same scenario
also passes against the Oil v11 image.

### Oil Barrel production-seed scenario

Use `docker-compose.oil-barrel.yaml` with the mode-0444 repeating-byte-31 v2 seed fixture and
`E2E_OIL_BARREL_FIXTURE=arena-v2-oil-barrel npm --prefix e2e run test:oil-barrel`. The 90-second,
retry-free two-browser route enables only Oil Barrel and uses ordinary movement and Bomb input. It
proves pending privacy while finite fire remains, safe revelation at `(3,1)`, the exact unlit east
clamp at `x=2752`, legal adjacent ignition and escape, unchanged `(4,1)` and `(3,2)` soft blocks,
finite-flame expiry after at least 60 ticks, persistent burning, survival, localized rendering and
reconnect retention without a pickup event or synthesized audio cue. It does not inject state,
search seeds, send direct gameplay commands or extend the timeout. Accepted Compose/Kubernetes
execution is part of the [32-case checkpoint](verification.md#current-v0347-oil-barrel-acceptance--2026-10-24).
Unlit and burning checks assert recognizable material palettes on both views; they accept valid
zero-based entity IDs and check room-scoped Polish rules after admission.
