# Gameplay v17: respawn lairs and solid players

Gameplay17 is the live contract on `/api/v1/game/ws`. Its source is
[`api/gameplay-v17.schema.json`](../../api/gameplay-v17.schema.json), Rust
`protocol::v17` and generated `web/src/game/v17/schema.ts`. Authentication accepts17;
older gameplay clients receive `incompatibleProtocol` and must refresh the paired web/server
application. Realtime/session API versions are independent.

## Profile and settings

The room binds rules9, CPU22, roster lifecycle3, lair layout1 and unchanged simulation/environment
randomness versions. Generator3 covers 15×13; generator4 covers 13×11 and11×9.

Environment settings retain `stormCloud` and `lairs { enabled, spawnSeconds, maxLivingMobs }`.
`spawnSeconds` is the first-spawn and post-death delay, 5–60 seconds. `maxLivingMobs` is required
and fixed at1, including while disabled; the UI has no capacity editor. Settings changes remain
atomic, revision-bound and restricted to the existing creator/readiness policy.

## Lair state

Each enabled room publishes two canonical wall lairs and at most two mobs. For each lair:

- `livingMobs: 1` requires `spawnRemainingTicks: null` and exactly one associated mob.
- `livingMobs: 0` requires an integer countdown between0 and its configured delay, with no mob.
- Zero means eligible now; an obstructed lair retries next tick.

Lairs have independent deadlines. A death starts that lair's full delay; no timer runs while
its mob lives. Mob IDs remain globally unique and carry no targets, paths or thaw deadlines.
Stopwatch/frozen-state validation and recipient privacy remain intact.

## Occupancy and projection

Every living player has a distinct center cell and a nonoverlapping 640×640 footprint.
The server validates all actors before privacy filtering. A recipient validates only visible
living pairs; hidden positions never enter a client or CPU observation. Cloaking changes
visibility, not authoritative collisions. Bracelet face contacts and atomic Teleporter swaps
follow [movement rules](movement.md).

Projection, state epoch, scores, terminal phases, settings revision and personal admission/action
IDs retain their existing checked bindings. Native v17 validators consume v17 messages directly;
messages are not relabelled through the closed v16 validator. The Rust-generated
[server corpus](../../api/fixtures/gameplay-v17-server-conformance.json) covers independent
cooldowns, occupied/null timers, blocked-zero eligibility, lifecycle and privacy.

Rules8/gameplay16/replay7 remain closed historical profiles. Test-only replay8 records accepted
rules9/CPU22 (or archived CPU21) inputs and reconstructs lair births/deaths, player collision and randomness exactly;
production replay recording stays disabled. Upgrade/restart ends process-local matches.

## Mines and bounded publication

`liveBombCount` counts ordinary capacity slots; Contact mines are additional consumables. All
physical bombs remain in the recipient-visible list, bounded by distinct board cells (at most 195).
Authoritative capture checks exact owned ordinary counts before privacy filtering. Bomb-blind
recipients receive no bomb list and gain no separate mine-count metadata.

Gameplay17 allows up to 216 events per simulation report, 256 per snapshot and event indices 0–215.
Its bounded outgoing frame is 196,608 UTF-8 bytes; incoming commands retain the 4,096-byte cap.
Latest-value publication prevents a slow client from accumulating snapshot queues. Archived
protocol bounds remain unchanged.
