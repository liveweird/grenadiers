# Testing and checks

`scripts/gates.sh` is the shared local/CI interface. Install both npm packages with `npm ci`
and use the repository's committed Cargo lockfile. Tests must exercise
behavior and observable contracts rather than merely repeat implementation constants.

## Rust

Use unit tests for pure validation/serialization and Tower oneshot tests for HTTP behavior.
Real WebSocket tests bind ephemeral listeners and connect over actual sockets with allowed
Origin headers. Cover readiness failure, invalid payloads, denied origins, connection limits,
and API/SPA separation. Keep tests independent and use bounded timeouts rather than sleeps.

`cargo test --workspace --locked` runs the debug suite; `cargo test --workspace --release --locked`
runs the same behavior in the production profile. The server gate requires both: debug assertions
must never perform required mutations. Clippy denies warnings. Coverage uses
`cargo-llvm-cov` **0.9.1** and an **80% line floor**, enforced in CI. Current measurements
and candidate evidence are recorded in [verification](verification.md).
Install with `cargo install cargo-llvm-cov --version 0.9.1 --locked`, then
`rustup component add llvm-tools-preview` and `scripts/gates.sh coverage`.
Stable branch coverage equivalent to Flow's Kover gate is not configured.

CI derives the exhaustive server item-mask partition matrix from the checked Rust catalog; the
workflow and JavaScript helpers do not duplicate item bits, cardinality, or ranges. Each partition
runs the complete debug, release, build, and 80%-floor coverage gates with a 20-minute limit and
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

`lint:api` checks the fixed OpenAPI file with Spectral's programmatic API and
`api-guidelines/spectral.yaml`; it retains the standard and custom rules without the CLI's
glob dependency. Parser/load errors and error-severity diagnostics fail the command.
`check:api`, `check:realtime`, and `check:gameplay` compare generated
contracts without modifying them. Gameplay generation/drift checks cover active v6 and historical
v2–v5; parity tests compare unchanged physical definitions. AJV validates runtime WebSocket payloads. EN/PL locale
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

The default suite uses one worker in CI and two locally; `E2E_WORKERS` explicitly overrides
that choice. Each multiplayer case already owns two browser contexts. Serial CI execution
keeps software WebGL within the two-player reference on two-core runners; specialized suites
also use one worker. Existing deadlines and assertions remain strict. Frozen-result checks
select retained snapshots by round and revision so automatic progression cannot change the
round being compared.

Install cargo-deny with `cargo install cargo-deny --version 0.20.2 --locked` for
`scripts/gates.sh security`. Rust advisory/license/source checks and full npm audits run
separately so external advisory outages do not obscure unit-test results. Document any
reviewed exception with its reason and removal condition; never add blanket suppressions.

All activated-v6 Playwright configurations disable framework trace capture. Raw traces can retain
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
remains pending until the predecessor-integrated candidate image exists.

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
execution remains pending until the predecessor-integrated candidate image exists.
