# Multiplayer CPU foundation

The live environment CPU implementation uses policy26 on rules10 for every live two-to-four-seat
room. The legacy multiplayer profiles remain policy17 for archived/reference paths. See the
[board profile table](board-sizes.md). The
[CPU contract guide](cpu-contracts.md) describes planner boundaries; the
[roster runtime](multiplayer-runtime.md) supplies the certified publication seam.

## Observations and compatibility

Rules 1–5 require two canonical actors and retain the 16-bomb observation/forecast cap.
Rules 6 accepts two to four canonical actors and 32 bombs. Forecast evolution, history and
hypothetical placement use the selected rules' limit rather than raising a global legacy cap.

`protocol::v18::observe_cpu_recipient` derives observations from the same checked recipient
projection used by humans. Cloaked actors, unavailable bomb knowledge, timed effects and
individual Teleporter markers remain private. Planning never receives the complete simulation
or a private arena/simulation random stream.

## Policies and controller ownership

`CpuProfile::SPIKE_PITS_STANDARD` and `SPIKE_PITS_EASY` select Utility26 for live rules10 rooms.
`RESPAWN_LAIRS_STANDARD` and `RESPAWN_LAIRS_EASY` retain Utility25 for archived rules9 rooms.
`RESPAWN_V23_STANDARD`/`RESPAWN_V23_EASY` retain Utility23 with goal-cell persistence.
`RESPAWN_V22_STANDARD`/`RESPAWN_V22_EASY` retain Utility22 with certified pickup handoffs.
`RESPAWN_V21_STANDARD`/`RESPAWN_V21_EASY` retain Utility21 for archived rules9 planning.
`LAIRS_STANDARD`/`LAIRS_EASY` retain Utility20 on historical rules8.
`ENVIRONMENT_STANDARD`/`ENVIRONMENT_EASY` retain Utility18 on rules7; multiplayer, current
legacy and historical replay profiles keep their previous identities. Planning uses the shared
utility and bounded forecasting machinery with explicit profile boundaries.

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

## Retained storm profile

Rules7 retains its `ENVIRONMENT_STANDARD`/`ENVIRONMENT_EASY` pair selecting policy18,
including when the cloud is disabled. Its observation overlay carries only the published
cloud leg and recipient-visible storm timers. Legacy observation layouts and constructors
remain unchanged; the checked v15 adapter constructs this retained profile.

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

Live controller selection uses the spike-pit profile below, including when every environment
feature is disabled.

## Lair profile boundaries

`RESPAWN_LAIRS_STANDARD` and `RESPAWN_LAIRS_EASY` identify archived Utility25 on rules9. The
checked v17 observation adds independent nullable lair deadlines and mob poses without targets, paths
or hidden thaw deadlines. Use `survival::plan_respawn_v24_with_origin`; historical entry points
remain closed to rules9. Disabled lairs use the same solid-player profile.

Gameplay18/CPU26 inherits these lair facts and adds the checked public spike-pit overlay. Its
route forecast advances idle contacts and warning/raised deadlines, then guards execution against
the exact observed phase state. Ordinary published damage protection is modeled; work limits and
the existing movement, planting and stopped-suffix safety budgets are unchanged.

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

## Combat pressure and player blocking

Every visible living rival is an enemy, whether human or CPU. CPU26 retains CPU25's reachable firing
positions whose actual blast can threaten a rival instead of walking into its occupied cell.
Useful pickups remain valuable; combat pressure outranks routine crate clearing. Planting still
requires a complete escape certificate, so increased aggression does not authorize suicide.

Routing uses the shared player cell/footprint clipper for visible living rivals. Admission and
continued execution check the next authorized movement against fresh blockers; unrelated rival
movement does not invalidate the whole scene. Alignment/forward corner ticks retain their two
axis segments. Hidden positions stay private; unexpected physical blocking is detected through
the CPU's own pose and invalidates movement before fresh Stop/recovery planning.

An occupied lair still has a possible future birth: its mob can die on the next update and the
replacement appears only after the full grace. Empty lairs use their independent published
eligibility ticks. These finite envelopes supplement bomb/storm proofs even when spawning is
currently suspended.

## Interrupted planting

A rival bomb can invalidate a CPU's certified escape, including when several CPUs plant on
one update. The controller stops and retains only independently checked origin-exit grace for
its own matching bomb. After a confirmed Stop, fresh planning uses the complete current bomb
scene and certifies a new movement pulse together with its mob-contact lease. The old route
never authorizes further movement. Origin grace ends when the footprint leaves the bomb cell.
Escape guidance excludes visible bomb cells. Foreign unpaused timed-bomb motion uses only named
stationary assumptions on checked pulses of at most three updates, so distant bombs do not
force waiting until the CPU's own fuse is almost spent.

Scheduled stops between route legs retain their contact lease. Contact invalidation stops the
route or pulse while preserving valid origin provenance for a fresh proof; completed commitments
release their leases. Historical rules8 controllers retain policy20; archived policy19 replay records
remain readable without rerunning their planner.

## Pickup handoff

CPU22 retains both movement and optional mob-contact certificates when approaching a pickup.
A clear scene with no observed bombs, flames or burning Oil permits one manual-free movement
update, followed by a confirmed Stop and fresh planning. The lease covers every admitted start
tick and the stopped suffix; changed inventory, timers or controls invalidate the old proof.
This also applies with lairs disabled. An eligible pickup under the current center prevents a
delayed start; a neighboring pickup touched only by the footprint does not block approach.

Teleporter and Diarrhea collection remain outside this handoff because they can relocate the
actor or spawn a bomb during the stopped suffix. Other hazard/contact checks and moving-plan
admission requirements remain unchanged. Archived CPU21 retains its original entry points;
replay8 executes matching CPU21–25 inputs with archived mob navigation; current replay9
binds CPU23/24/25 and mob navigation2. Neither reruns the planner.

## Persistent strategic intent

CPU26 retains CPU25's exact item, block or opponent separately from its approach cell and execution route.
Commitment, material utility differences and observed progress govern switching. Safety overrides
revoke execution authority while retaining the subject for resumption. Optional bombing serves
that subject; pickup pursuit does not opportunistically bomb unrelated objects. Explicit phases
and switch reasons make the behavior inspectable through an operator-enabled, recipient-filtered
developer view. See [intent and diagnostics](cpu-intent.md) for lifecycle, thresholds and usage.
CPU21–24 retain their historical selection and memory-reset paths.

## Focused checks

Run `cargo test -p grenadiers-core cpu:: --lib` and
`cargo test -p grenadiers-server game::rooms::cpu:: --lib` for planner/controller regressions.
The v18 adapter tests compare CPU facts with human recipient projections. Additional cases
cover forecast capacity, rival interference, both difficulties, independent controllers and
player four. Live multi-CPU scheduling is owned by the room owner/socket integration rather than
this projection API.
