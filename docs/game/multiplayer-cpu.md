# Multiplayer CPU foundation

The live environment CPU implementation uses policy 18 on rules 7 for every live two-to-four-seat
room. The legacy multiplayer profiles remain policy 17 for archived/reference paths. See the
[board profile table](board-sizes.md). The
[CPU contract guide](cpu-contracts.md) describes planner boundaries; the
[roster runtime](multiplayer-runtime.md) supplies the certified publication seam.

## Observations and compatibility

Rules 1–5 require two canonical actors and retain the 16-bomb observation/forecast cap.
Rules 6 accepts two to four canonical actors and 32 bombs. Forecast evolution, history and
hypothetical placement use the selected rules' limit rather than raising a global legacy cap.

`protocol::v15::observe_cpu_recipient` derives observations from the same checked recipient
projection used by humans. Cloaked actors, unavailable bomb knowledge, timed effects and
individual Teleporter markers remain private. Planning never receives the complete simulation
or a private arena/simulation random stream.

## Policies and controller ownership

`CpuProfile::ENVIRONMENT_STANDARD` and `ENVIRONMENT_EASY` select Utility v18. Existing
`MULTIPLAYER_STANDARD`, `MULTIPLAYER_EASY`, `CURRENT`, `CURRENT_EASY` and historical replay
profile checks retain their legacy behavior. The live environment uses
the established utility and bounded forecasting machinery with explicit rules-7 branches.

Each controller owns its participant/world identity, memory, RNG, pending work, route and
cadence. Cross-seat results cannot commit plans or random draws. Cancellation of one controller
does not cancel another. The shared planner retains two workers, a 32-job queue and the
existing per-job work allowance. Forecast identity includes board dimensions, and movement,
hazards, Power traversal and Punch routes use the selected board without changing 15×13 policy
ordering or cadence.

Punch and Bracelet proofs inspect all relevant visible rivals and third-party blockers.
Unknown hidden rivals withdraw proofs that cannot be certified. Bracelet also rejects
third-party Bracelet threats that could counter-shove the CPU off its certified approach.
These additional multiplayer guards do not change rules-5 admission semantics.

## Environment profile

Rules7 has an explicit `ENVIRONMENT_STANDARD`/`ENVIRONMENT_EASY` pair selecting policy18,
including when the cloud is disabled. Its observation overlay carries only the published
cloud leg and recipient-visible storm timers. Legacy observation layouts and constructors
remain unchanged; the checked v15 adapter creates the new profile.

Ordinary manual-free navigation retains the existing clear-board path, including supported
pickups. It carries no escape certificate: known bombs, flames, burning Oil and unsupported
control effects prevent this path. Cloud contact alone is nonlethal, and the driver retains
bounded holds and confirmed stops. Planting and survival never use this navigation shortcut.

Each certified execution lane tracks its own slowdown deadline interval. Routing models movement,
delayed starts and stopped suffixes at physical ticks. After the published straight leg,
planning uses a bounded possible-contact envelope. A certificate requires contact to be
impossible or an existing guaranteed slowdown to cover the remaining movement; uncertain
speed changes cannot be assumed safe. Deadline intervals participate in route dominance and
lease validation. Legacy policies retain their original bulk checks and work accounting.

Live controller selection follows the profile table above.

## Lair profile boundaries

`LAIRS_STANDARD` and `LAIRS_EASY` identify Utility19 on rules8. Its checked v16 observation
adds public lair attempt timers and mob poses without targets, paths or hidden freeze deadlines.
Use `survival::plan_lairs`; historical public policy, routing and survival entry points reject
rules8 rather than returning movement without a mob-contact proof.

The planner returns a separate finite contact lease beside the existing bomb/escape proof.
Navigation covers movement and a stopped handoff; an optional plant must also cover its complete
fuse/flame escape. It computes useful movement before trying optional tactics, charges failed
work to the same allowance and returns that fallback when a plant cannot be certified. Pickups
on the actual path shorten authorization to the collection update; unrelated pickups do not
slow every command to one update. Changed movement physics require a fresh stopped observation.

Mob envelopes conservatively include current mobs and earliest possible births, without assuming
walls, future freezing or deaths make contact impossible. Teleporter landing checks cover full
pickup-cell destinations and retained movement, including pending contents exposed by expiring
fire. Unresolved Teleporter scenes can withhold optional planting. These are bounded proofs of
the observed scene, rather than guarantees against arbitrary later rival actions.

Lair controller integration must preserve ordinary admission identity, age, pose, effects,
command epoch and stopped authority. Contact checks supplement those fences; they do not replace
them or rebase a result to a newer observation. Hold deadlines remain unconditional even when
workers or observations are late. See [CPU contracts](cpu-contracts.md).

## Focused checks

Run `cargo test -p grenadiers-core cpu:: --lib` and
`cargo test -p grenadiers-server game::rooms::cpu:: --lib` for planner/controller regressions.
The v15 adapter tests compare CPU facts with human recipient projections. Additional cases
cover forecast capacity, rival interference, both difficulties, independent controllers and
player four. Live multi-CPU scheduling is owned by the room owner/socket integration rather than
this projection API.
