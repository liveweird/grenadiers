# Historical browser acceptance notes

> Historical record: the statuses, source IDs, counts, candidate labels, and execution claims below
> describe their original checkpoints. They are not the current acceptance status. Use the
> [current testing index](../testing.md#browser-acceptance-index), [verification record](../verification.md),
> and [release guide](../releases.md) for current procedures and evidence.

### Generated pickup acceptance

The default eight scenarios run against random generated maps and rely only on safe openings.
A separate `test:pickups` command uses the explicit read-only operator fixture setup in
[deployment.md](../deployment.md#reproduce-a-generated-pickup-round). It fails without
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
and prove protection ends before a later lethal blast. See [deployment](../deployment.md).

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
movement/release through actual page delivery. The [method and reference](../game/websocket-latency.md)
define the fixed budget, CPU-affinity profile and evidence limits; twenty observations do not
establish WAN behavior or a tail guarantee. Gameplay traces remain disabled.

## Procedural sound

`xvfb-run -a npm --prefix e2e run test:audio` runs the natural headed Chromium sound scenario
on Linux; macOS needs no Xvfb wrapper. The [sound guide](../game/audio.md) records gesture, mute,
visibility and cleanup evidence. CDP context/node observations establish browser processing,
not hardware audibility or listening quality; final context disposal is unit-tested separately.

Golden Flame has a separate production-seed scenario:
`E2E_GOLDEN_FLAME_FIXTURE=arena-v2-golden-flame npm --prefix e2e run test:golden-flame`.
Use `docker-compose.golden-flame.yaml` with the same image and wait for readiness after
recreating the service. It checks generated collection, captured maximum range, exact deadline,
reconnect and restored base range. See [deployment](../deployment.md#reproduce-opt-in-golden-flame-range).

Short Fuse has a separate production-seed scenario:
`E2E_SHORT_FUSE_FIXTURE=arena-v2-short-fuse npm --prefix e2e run test:short-fuse`.
Use `docker-compose.short-fuse.yaml` with the same image and wait for readiness after
recreation. The scenario selects a five-second base fuse, verifies a captured 150-tick
shortened fuse on both peers and natural detonation, reconnects without extending the
effect, and verifies a later ordinary 300-tick bomb after expiry.
See [deployment](../deployment.md#reproduce-opt-in-short-fuse-timing).

Short Flame has a separate production-seed scenario:
`E2E_SHORT_FLAME_FIXTURE=arena-v2-short-flame npm --prefix e2e run test:short-flame`.
Use `docker-compose.short-flame.yaml` with the same image and wait for readiness after
recreation. It verifies range-one captures and actual fire stopping before the second cell
of a known clear ray, independent ordinary fuse timing, reconnect, exact expiry and later
base-range damage. See [deployment](../deployment.md#reproduce-opt-in-short-flame-range).

Long Fuse has a separate production-seed scenario:
`E2E_LONG_FUSE_FIXTURE=arena-v2-long-fuse npm --prefix e2e run test:long-fuse`.
Use `docker-compose.long-fuse.yaml` with the same image. It verifies generated collection,
captured 240-tick timing from a 120-tick base, exact expiry, and restored ordinary timing.
See [deployment](../deployment.md#reproduce-opt-in-long-fuse-timing).

Reverse Controls has a separate production-seed scenario:
`E2E_REVERSE_CONTROLS_FIXTURE=arena-v2-reverse-controls npm --prefix e2e run test:reverse-controls`.
Use `docker-compose.reverse-controls.yaml` with the same image. It holds one raw east input across
collection and the exact 600-tick expiry, checks both peers, and captures screenshots only outside
the timed window. See [deployment](../deployment.md#reproduce-opt-in-reverse-controls-movement).

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
the [v0.3.42 published delivery](../releases.md#v0342-published-prerelease--2026-10-23).

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
[verification](../verification.md#v0345-punch-native-accepted-checkpoint--2026-10-24--27-cases).

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
execution is part of the [32-case checkpoint](../verification.md#current-v0347-oil-barrel-acceptance--2026-10-24).
Unlit and burning checks assert recognizable material palettes on both views; they accept valid
zero-based entity IDs and check room-scoped Polish rules after admission.
