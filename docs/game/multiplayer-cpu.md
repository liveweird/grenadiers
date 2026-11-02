# Multiplayer CPU foundation

The live environment CPU implementation uses policy60 on Easy/Standard and policy61 on Hard for every live
two-to-four-seat gameplay20/rules10 room. Policy56/59 remains the archived replay30 pair and policy58
remains the archived Hard identity for replay29, while
the legacy multiplayer profiles remain policy17 for archived/reference paths. See the
[board profile table](board-sizes.md). The
[CPU contract guide](cpu-contracts.md) describes planner boundaries; the
[roster runtime](multiplayer-runtime.md) supplies the certified publication seam.

## Observations and compatibility

Rules 1–5 require two canonical actors and retain the 16-bomb observation/forecast cap.
Rules 6 accepts two to four canonical actors and 32 bombs. Forecast evolution, history and
hypothetical placement use the selected rules' limit rather than raising a global legacy cap.

`protocol::v20::observe_cpu_recipient` derives observations from the same checked recipient
projection used by humans. Cloaked actors, unavailable bomb knowledge, timed effects and
individual Teleporter markers remain private. Planning never receives the complete simulation
or a private arena/simulation random stream.

## Policies and controller ownership

`CpuProfile::CONTACT_PUNCH_V60_STANDARD` and `CONTACT_PUNCH_V60_EASY` select Utility60
for live rules10 Easy/Standard rooms. `CpuProfile::HARD_V61` selects Utility61 for Hard rooms
with the same observation boundary. Private replay31 binds this exact current pair and ContactV2;
replay30 retains CPU56/59 with ManualV1.
`RELIABLE_V27_STANDARD`/`RELIABLE_V27_EASY` and `HARD_V29` retain their archived behavior.
`SPIKE_PITS_STANDARD`/`SPIKE_PITS_EASY` retain Utility26 for archived/reference paths.
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

## Contact Punch routing

CPU60/61 do not intentionally emit the archived manual Equipment action or use an intentional
contact-Punch tactic. Because ContactV2 can relocate a bomb on movement contact, every certified
forward segment checks the immediate swept contact cells for a visible bomb or a forecasted
punchable moving bomb. Any possible unmodelled relocation rejects that route. Exact retained
origin grace permits departure from an existing overlap without activating Punch. A fresh
observation may make another route certifiable.

The guard uses the existing hazard forecasts and their indexed per-cell bomb-motion facts. Its
actor scan and candidate-by-forecast checks debit the same shared work allowance; it adds no search,
forecast, worker, RNG draw or planting proof. The simulation remains authoritative about cooldown,
trigger type, landing conflicts and the contact latch. Archived CPU56/59 keep their ManualV1
behavior and archived manual Punch tactic unchanged.

## Persistent burning Oil

CPU27 can certify a useful bomb while static burning Oil exists. With Teleporter enabled,
planting still requires visible, empty bomb knowledge, no finite flames and no exposed
Teleporter. Oil remains a persistent lethal forecast cell; full escape, selected-target
usefulness, mob-contact and the 65,536-work limit still apply. CPU26 keeps its original refusal.

This removes a global planting prohibition after an Oil barrel burns; it does not authorize
bombing in every Teleporter scene or guarantee safety against arbitrary later rival actions.
Changed hazards invalidate a committed route. Newly revealed items require fresh observation.
Private replay12 binds CPU27/navigation3; earlier readers retain their original identities.

## Hard pressure

Hard examines bounded, reachable firing origins using its current range and supported weapon.
A position earns opponent pressure only when its exact blast reaches a living, visible rival.
It prefers that pressure to routine wall farming; valuable drops retain their existing priority.
Targets keep their existing commitment and progress deadlines instead of switching every update.

This is tactical guidance, not permission to plant. Full escape, contact and target-usefulness
proofs remain required, and admission rechecks a rival that moved or disappeared. Optional
pressure analysis shares a small bounded allowance across rivals; exhausted or unsupported
analysis falls back to collect/farm guidance. Hard has no hidden information, faster movement,
larger bomb inventory, extra planning workers or immunity. Easy keeps its navigation pause and
Standard remains the default. See [Solo difficulty](single-player.md#difficulty).

For one selected visible block target, Hard61 retains Hard59's comparison of eligible adjacent
farming approaches at the baseline navigation distance and prefers a strictly larger visible soft-block yield under
the existing timed blast geometry. Ties keep the stable approach and farther origins cannot win.
This reuses the unchanged optional pressure allowance and does not reveal concealed drops, reorder
block targets, add RNG, or authorize a bomb. The ordinary complete planting, escape, contact and
fresh-publication proofs remain mandatory. Hard58 retains the earlier coordinate-selected result.

## Clear-arena counter-pressure

With Teleporter enabled, CPU29 requires at least one completed running update, no finite flames,
no exposed Teleporter and complete visible bomb knowledge before planting. An empty bomb set
retains supported farming around blocks and Burning Oil. With existing bombs, every bomb must
be timed and unpaused and the entire soft-block set must be empty. This prevents hidden item
reveals, including when an existing blast ignites the proposed bomb early.

These public preconditions authorize only attempting the unchanged full escape/contact/usefulness
proof. Bomb motion remains uncertain. A bomb's conservative motion envelope can still make a
nearby or same-axis plant unprovable; the CPU keeps its certified fallback. Its active own-escape
route retains its existing hold. Replay14 binds CPU29; replay13 keeps CPU28 unchanged.

## Rules10 pickup progress

Current profiles allow a certified one-update approach that can collect a pickup, then require
Stop and a fresh observation before further movement. This prevents oscillation between drops
when an ordinary longer movement pulse must stop before collection. The short handoff uses the
complete hazard forecast, spike/storm checks and mob-contact proof; it does not enable ordinary
uncertified movement on rules10. Teleporter and Diarrhea retain their existing exclusions.
Easy keeps its normal clear-navigation cadence. Old policies27–32 remain unchanged; replay15
retains Easy/Standard31 and Hard32, while replay16 binds Easy/Standard33 and Hard34; replay17 binds
Easy/Standard33 and Hard35. Replay18 binds Easy/Standard36 and Hard37; earlier replay versions retain
their exact policy identities. Replay19 binds archived Easy/Standard38 and Hard39, without
changing gameplay19. Later private formats advance closed CPU identities; current replay31 binds
Easy/Standard60 and Hard61 with ContactV2.

## Alternate farming refuges

When already at a committed adjacent block-farming approach, current profiles can try up to two
deterministic refuge candidates before the general route search. Candidate geometry follows the
target, blast range and public monster/lair positions. Movement lengths can include an interior
turn point instead of only the turn-band boundary or maximum pulse length.

Each candidate must pass the complete absent/present bomb-route proof, exact selected-block
usefulness check and unchanged monster-contact proof. Failed candidates consume the same bounded
allowance: the optional search receives at most 8,192 work units and one eighth of the remaining
job budget, leaving the existing fallback available. Unsupported items, uncertain hazards and
unsafe routes still prevent planting. With lairs disabled, the prior planner is reused unchanged.
This expands supported safe routes; it does not guarantee that every farming approach has an escape
or that a bot can automatically recover from every long-match stall.

## Refused farming approaches

With lairs enabled, current profiles remember one eligible refused farming position for 120 ticks
(two seconds). They exclude that cell before choosing the best approach for each block, so
switching wall targets cannot immediately select the same failed position. Pickup and opponent
candidates keep their existing selection rules. The memory belongs to the CPU's world/seat,
expires normally, and starts empty in a new world.

The position is recorded only when the bot is stopped at an adjacent farming approach with
visible bomb knowledge, spare capacity, an ordinary bomb and supported planting conditions,
and every complete planting attempt has failed. A successful plant does not record a failure.
Capacity exhaustion, unsupported effects and queue/admission delays do not create this memory.
All movement and later planting still need their existing complete proofs; this is guidance,
not permission to move or plant. With lairs disabled, the prior planner remains unchanged.

The captured refused (3,9) position now leads to guidance toward (3,8) for block (3,7). That
guidance decision and the safe plant from (3,8) are separately tested; they do not establish a
continuous restoration or replay of the original captured match.

## Safe second bombs

After a Hard bot reaches a confirmed stopped refuge from its first bomb and has spare capacity, it may
queue one optional attempt to replace its stationary hold with a useful full new Stop+Bomb plant
certificate. The old route and contact proof remain installed while planning; the attempt uses job-local
Reengage memory and RNG and commits live state only after the owner accepts Stop+Bomb. A shared absent/present
survival horizon must cover the old deadline. Queue refusal, stale or failed proof, non-plant result,
displacement or submission refusal retain the old hold. Optional jobs follow every seat's urgent jobs.
The existing two workers, 32-job queue, 65,536-work limit and all forecast, motion, contact and usefulness
proofs remain unchanged. Hard35 reuses the Core34 planner; archived Hard39 adds the bounded
farming refuges above, and current Hard61 retains them. Teleporter counter-pressure still requires
globally clear soft blocks and its prior support rules.

## Footprint-boundary recovery

Escape guidance targets the centre of the current safe traversable cell while the actor's full footprint
still overlaps a threat from an adjacent cell. This prevents reversal when only the centre has crossed the
boundary. Guidance requires a centred full-footprint position with no definite lethal forecast; it does not
change pulse, contact, storm, pit, static-obstacle or admission proofs. ConditionalStationary remains the
existing fallback. This is guidance only: it grants no hidden facts, faster physics or proof waiver, and
`origin_target` is unchanged.

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
stationary assumptions on checked pulses of at most three updates. Repeated Stop/replanning
cycles can still make an interrupted escape too slow, especially with Short Fuse; these pulses
do not guarantee completion of the original full escape. See the
[controlled recovery investigation](../history/cpu-short-fuse-investigation-2026-11-01.md).

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
replay8 executes matching CPU21–25 inputs with archived mob navigation; archived replay9
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
The v18/v19/v20 adapter tests compare CPU facts with human recipient projections. Additional cases
cover forecast capacity, rival interference, all three difficulties, independent controllers and
player four. Live multi-CPU scheduling is owned by the room owner/socket integration rather than
this projection API.
