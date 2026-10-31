# Multiplayer CPU foundation

The live environment CPU implementation uses policy 23 on rules 9 for every live two-to-four-seat
room. The legacy multiplayer profiles remain policy 17 for archived/reference paths. See the
[board profile table](board-sizes.md). The
[CPU contract guide](cpu-contracts.md) describes planner boundaries; the
[roster runtime](multiplayer-runtime.md) supplies the certified publication seam.

## Observations and compatibility

Rules 1–5 require two canonical actors and retain the 16-bomb observation/forecast cap.
Rules 6 accepts two to four canonical actors and 32 bombs. Forecast evolution, history and
hypothetical placement use the selected rules' limit rather than raising a global legacy cap.

`protocol::v17::observe_cpu_recipient` derives observations from the same checked recipient
projection used by humans. Cloaked actors, unavailable bomb knowledge, timed effects and
individual Teleporter markers remain private. Planning never receives the complete simulation
or a private arena/simulation random stream.

## Policies and controller ownership

`CpuProfile::RESPAWN_LAIRS_STANDARD` and `RESPAWN_LAIRS_EASY` select Utility23 for live rules9 rooms.
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

Live controller selection uses the lair profile below, including when both environment features are disabled.

## Lair profile boundaries

`RESPAWN_LAIRS_STANDARD` and `RESPAWN_LAIRS_EASY` identify Utility23 on rules9. The checked
v17 observation adds independent nullable lair deadlines and mob poses without targets, paths
or hidden thaw deadlines. Use `survival::plan_respawn_v23_with_origin`; historical entry points
remain closed to rules9. Disabled lairs use the same solid-player profile.

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

Every visible living rival is an enemy, whether human or CPU. CPU23 seeks reachable firing
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
replay8 executes matching CPU21–23 inputs with archived mob navigation; current replay9
binds CPU23 and mob navigation2. Neither reruns the planner.

## Productive goals and bomb opportunities

CPU23 retains a reachable pickup, crate-clearing or combat goal across short movement pulses.
It revalidates the goal's current kind and value after collection or board changes, ranks
reachable goals, and uses a bounded history of settled cell transitions to avoid unproductive
reversals. An unreachable high-value item cannot suppress a reachable lower-value objective.
Escape planning can still reverse immediately when danger requires it.

Outside combat, reachable firing positions beside destructible blocks offer upgrade opportunities;
the CPU does not know hidden drops before a block is destroyed. At a useful firing position it
tries planting before leaving for another position. Unrelated exposed pickups no longer bypass
that decision. A bounded set of distinct first movements, including Stop, can yield a certified
placement even when the ordinary navigation direction cannot.

Optional bomb attempts share a charged budget after a safe movement fallback has been computed.
Each accepted placement still requires the existing escape and mob-contact proofs; failed or
exhausted tactics return the saved fallback. These improvements apply with lairs enabled or
disabled. Easy retains its slower cadence; Standard gains stronger decisions without additional
visibility, reaction privileges or larger planning allowances. Strategic memory commits only
with accepted inputs and resets with the world. Archived CPU21/22 retain their original paths.

## Focused checks

Run `cargo test -p grenadiers-core cpu:: --lib` and
`cargo test -p grenadiers-server game::rooms::cpu:: --lib` for planner/controller regressions.
The v17 adapter tests compare CPU facts with human recipient projections. Additional cases
cover forecast capacity, rival interference, both difficulties, independent controllers and
player four. Live multi-CPU scheduling is owned by the room owner/socket integration rather than
this projection API.
