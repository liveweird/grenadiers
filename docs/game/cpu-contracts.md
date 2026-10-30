# CPU contracts

These Rust contracts support the [solo opponent](single-player.md) and the broader
[accepted design](single-player-cpu.md). The
[backlog](../../BACKLOG.md#single-player) owns delivery status.

## Controllers and ordinary inputs

`core::room::RoomMode` derives controller kinds independently of occupancy and socket state.
Human duels have two human controllers. Solo fixes the human at seat One and the CPU at seat
Two. `CpuProfile::CURRENT` selects Standard and `CURRENT_EASY` selects Easy in the current
policy family. Named archive constants preserve historical Standard replay identities. Policy
identity describes decision semantics, not measured strength; seeds stay private. Live owner
construction accepts only the two exact current profiles. Retaining a historical replay identity
does not re-enable that older policy for new matches.
Controllers carry an explicit seat identity through memory, recipient validation and ordinary
input submission. Foreign-recipient observations are rejected before state changes. Current
solo construction still authorizes only seat Two; an explicit identity does not grant a human
seat CPU input authority.

`core::cpu` accepts neutral observations and emits movement or Stop, with at most one Bomb or
Equipment intent. Intent carries no credentials, seat authority, readiness, settings mutation
or manual action ID. The server allocates those IDs and uses ordinary input
validation; tests exercise accepted actions through the existing runtime and replay path.

## Information boundary

The current-protocol adapter converts the already filtered v11 recipient projection. It must
never feed the policy `State::public_view` or reconstruct an authoritative `State`. A cloaked
rival exposes only identity, seat and alive status. Invisible Bombs makes all bomb knowledge
unavailable, including the observer's bombs; unavailable differs from an empty visible list.

Observations retain public terrain, settings, effects, exposed pickups/Oil, flames and visible
bomb triggers and original placement ticks. Private movement residuals, collision grace, action cooldown histories and bomb
motion timing remain explicitly unknown. Concealed content, seeds and opponent intent never
enter observations. The adapter receives server-owned world identity separately because a
public command epoch is not a physical-round identity.

## Bounded decisions and replay

Memory belongs to a room instance, physical round and CPU seat. Decision admission separately
checks the current command epoch, observation tick and request serial. Reconnect preserves
memory; a new physical world resets it. The local rejection fallback is Stop without a manual
action; it grants no submission authority. Discard stale or wrong-world completions instead of
injecting their fallback into a newer request. Only the current controller's missing/exhausted
request may submit its ordinary Stop. History has bounded, deterministic retention, and work is
counted in logical units rather than hardware milliseconds.

Tie-breaking uses its own seeded ChaCha stream and never draws from arena generation. Small
fixture policies remain test-only; production uses the current policy family. Replays consume recorded accepted
actions rather than invoking a policy again. Production journal allocation remains disabled;
explicit solo recording adds format-3 header provenance while preserving formats 1/2 and
accepted-input record capacity.
Historical policy identities accept only their matching Standard profile; the current family
supports both levels. New recordings retain the actual room profile. The live v13 room shape
publishes the chosen difficulty; archived v12 remains Standard-only.

## Difficulty cadence

Standard does not add delays. Easy arms one bounded pause after accepted ordinary navigation,
starting its 12-tick deadline once the stopped pose is confirmed. Repeated observations cannot
restart it. Deferral happens before a fresh request, preserving six-tick result admission and
exact manual execution times. Active escapes/routes/pulses and unavailable or newly dangerous
observations bypass the pause. Reconnect preserves cadence; world/command-epoch changes reset
it. Goal scores, RNG, worker budgets and known-hazard mechanics are shared.

## Goal guidance

A single bounded static flood-fill records reachable distance and the first step toward useful
pickups, block openings and a visible rival. Pickup value depends on observed upgrades, current
equipment and remaining buff duration. Useful upgrades, Heart and beneficial effects can be goals;
capped upgrades, duplicates and unnecessary refreshes have no collection priority.

Known curses, Oil, Teleporter and unsupported stored weapons are excluded from deliberate goals
and ordinary navigation transit. Harmless capped upgrades or duplicate/long-duration buffs remain
transitable. Equipment shares one replaceable slot: navigation preserves held Heart rather than
replacing it with an unused capability. Remembered ordinary next steps use the same eligibility
check. Supported Piercing/Power charges can be goals for an empty slot or replacement of an
unsupported charge. Duplicate charges remain transitable without collection priority; navigation
does not seek a replacement for a usable charge, while both supported alternatives remain
transitable. The weapon slot is separate from equipment.
Stable ordering and the independent RNG resolve genuine ties; exhausted work yields Stop.

This is static guidance. Existing hazard checks, planting certificates and pickup-changing-path
restrictions retain authority, and hidden contents never enter goal selection.

## Planning and committed execution

Two shared workers accept at most 32 queued jobs; a controller has one outstanding request and
one bounded result slot. Planning uses neutral observations and a cloned independent RNG. The
owner never waits for it. Results must still match the issued request and be at most six logical
ticks old; the observation tick identifies that request, while runtime age is checked separately.
Only accepted current input commits prospective memory/RNG.

Plans hold ordinary movement for 1–12 updates, then the owner submits ordinary Stop at its
committed deadline. The narrow CPU1 planting fallback uses `follow_checked_escape` to execute only
the already planned route of at most four cells from fresh checked snapshots. It performs no
goal/path search, random draws or manual action and does not wait for another worker. This
keeps worker contention outside the route's scheduling bound. Unsupported effects, unknown
bombs, blocked or displaced route geometry stop the executor rather than inventing safe facts.
New hazards may still invalidate its game-world assumptions.
The executor distinguishes continuing, waiting, completed and invalidated routes. Invalidation
clears committed mode so planning can resume after ordinary Stop executes.

## Hazard forecasts and executable pulses

The authoritative game and forecast share immutable blast geometry. Forecasts retain absolute
half-open lethal/blocking intervals, separate safe gaps and persistent Oil fire. Same-tick
chains see unchanged blockers; later waves see destruction. Destroyed soft-block cells remain
blocked for traversal until newly revealed contents are observed.
Neutral motion clips only against permanent terrain and arena boundaries. Sweeps touching
soft blocks without usable Wall Pass, or touching known unlit Oil, are rejected rather than
relying on temporary collision. Equipped Wall Pass uses the intact-block rules below.

Nominal stationary hazards are separate from conservative motion/trigger/pause uncertainty.
Unknown motion can advance immediately and then every ten updates. Uncertain chain outcomes
may extend through the 840-tick horizon; conservative uncertainty can prevent an otherwise
possible move. History retains at most 16 bomb identities and two observations each, with a
sticky displacement flag. Capacity, clock overflow, hidden bomb vision and exhausted work
return failure rather than a truncated safe forecast. Each decision has 65,536 logical units;
each cell layer holds at most 32 intervals.

`certify_pulse` checks every first-execution update in its admission window, exact movement,
modifier expiry and swept footprint. A known executed Stop establishes zero residual and no
pending turn before planning; merely admitting Stop does not establish that fact. Certified
pulses hold at most 12 updates; conditional pulses hold at most three and may waive only named
motion assumptions for bombs never observed moving. Trigger/pause uncertainty is never waived.
Every possible endpoint must remain safe through the exclusive stopped-lease deadline,
including publication and accepted-worker-age delays. Known pickup-changing paths are withheld.

The executor admits one pulse, checks intermediate snapshots at their own timestamps and
submits ordinary Stop at its deadline. Fresh hazards, displaced bombs, changed effects/geometry
or mismatched poses invalidate it. Missing workers cannot extend certification. Static steering
is guidance only; no guidance suffix or conditional pulse authorizes planting or certifies a
complete multi-leg escape. Clear-board goals and narrow CPU1 planting retain their own guards.
Clear-board ordinary movement has no hazard lease: it retains its accepted hold through
intermediate publications and can meet newly introduced hazards before the next decision.
The lease's intermediate-pose and hazard-invalidation guarantees apply only to leased actions.

## Committed timed routes

`plan_route` checks one common program against up to seven exact initial execution times.
Each lane retains its own positions, headings and absolute effect clocks; aggregated bounds
never seed later movement. The planner reuses the pulse motion kernel and returns only a
complete certificate, including a safe terminal stop through the requested survival deadline.
Search has at most eight steps and 2,048 expansions within the shared decision work allowance.

Each movement is followed by exactly three stopped physics updates. The owner admits the next
scheduled input before its boundary, after a checked post-Stop observation. It continues the
committed program without another worker result. Proposed commands advance the executor only
after ordinary input acceptance; missing observations, rejected commands or missed deadlines
invalidate the schedule rather than shifting it. Explicit waits permit travel through a later
safe interval. Conditional routes retain their named assumptions and never authorize planting.

A bounded expected-scene ledger distinguishes predicted bomb removals, destruction, known Oil
ignition and exact flame-expiry changes from unexpected facts. Normal predicted explosions do
not cancel the route. New bombs, displaced bombs, revealed items or other unmodeled changes do.
The owner performs bounded observation/deadline checks; forecasting and search stay on workers.
If complete route planning fails, the independently checked one-pulse fallback remains available.

## Certified ordinary planting

`plan_ordinary_plant` derives an ordinary timed bomb and proves one complete escape program in
both its placed and absent futures. Acceptance of a Bomb input does not prove physical placement;
absence can change blast shielding and chain timing. Both forecasts, search and replay consume
the same decision work allowance. A failed speculative proof preserves the checked fallback.
Only unconditional certificates authorize this path.

The first execution is exactly seven updates after the basis observation. Early completed results
stay in the controller's one pending slot until admission at basis + 6; waiting commits no RNG or
memory. Recheck the scene, identity, epoch and stopped pose then. Late or stale results are discarded.
First movement executes before placement, so the origin comes from its resulting position. The
new bomb affects same-tick explosions and blocks movement from the next update. Placement-derived
origin grace permits departure and is permanently consumed when the footprint leaves the cell.

The first move lasts one update, followed by three stopped updates; an initial Stop waits four.
A fresh publication must confirm exactly one new matching own bomb by public placement tick,
cell, range, standard blast and timed trigger, or match the absent future. Confirmation binds its
actual identity. Manual rejection chooses the already-proved absent branch without retrying Bomb.
Unexpected facts invalidate execution; continuation uses the existing committed route schedule.

Range and fuse use shared game rules at the absolute placement tick, including Golden Flame,
Short Flame, Short Fuse and Long Fuse. Slow Foot and Reverse Controls expiry use shared motion.
Stored special weapons or a live own Remote, active Diarrhea/own Stopwatch, hidden bomb vision,
insufficient capacity,
unsupported uncertainty, an unobservable early chain or an incomplete horizon cause refusal.
The full blast expiry and final stopped margin must fit; no fuse or escape deadline is shortened
to fit the forecast. The guarded CPU1 planting fallback remains available where applicable.

## Certified timed weapons

`plan_timed_weapon_plant` accepts enabled, held Piercing or Power and shares the ordinary planting
proof. The ordinary-only entry point still rejects stored weapons. Both scenarios retain the
same exact input program, movement-before-placement timing, full blast-expiry horizon and work
allowance. The certificate records the actual blast kind; confirmation rejects a different kind.

Awaiting placement and confirmed absence require the captured weapon to remain held. Confirmed
successful placement requires exactly that charge to be consumed. Presence is confirmed before
checking this transition, and every other actor invariant uses the same shared guard. Unexpected
replacement, loss, recollection or a changed certificate basis invalidates execution. A confirmed
Present outcome cannot be downgraded to Absent. Ordinary rejection or physical placement veto
preserves the charge and continues the absent route without an immediate Bomb retry.

Usefulness uses the shared blast geometry: Piercing can affect blocks beyond another soft block,
and Power can threaten a diagonal cell within its diamond. Field construction and full traversal
are charged before accepting utility; exhaustion preserves the checked fallback. Remote, Landmine
and any live own Remote remain unsupported by timed placement because their triggers differ.
Certified movement still rejects pickup-changing paths and never assumes hidden facts.

## Finite-hazard responsiveness

Unlit Oil is a static obstacle, so its presence elsewhere does not prevent ordinary movement
when bomb knowledge is visibly empty and no flames or burning Oil remain. Existing transit
and collision checks still exclude the Oil cell.

For non-plant routes with visibly empty bombs and no burning Oil, the survival deadline covers
the latest current flame expiry and at least the latest admission start plus the 12-update
movement cap and 10-tick stopped margin, clipped to the forecast horizon. The existing seven
admission lanes therefore require at least 29 ticks from the observation. The full 840-tick
forecast remains intact; only the request deadline changes. Active bombs and persistent Oil
fire retain the full horizon. Additional scans consume the same bounded work budget, and
checked arithmetic failure withholds the plan. Planting proofs and execution guards are unchanged.

A stationary non-plant route can finish before its original deadline after a real public
hazard gap. This requires an originally visible nonempty bomb set, a current visible empty
bomb set, no flames or burning Oil, only Wait steps, no pending transition and an executed
Stop confirmed at or after first execution. Normal scene, actor and exact pose validation
still runs first. Hidden bombs, unconfirmed Stop, planting, movement, legacy escape and origin
recovery do not use this completion. Completion authorizes no input; the controller requests
a fresh ordinary decision, which can resume existing clear-board pickup navigation. Forecasts,
proof horizons, work limits and pickup-changing pulse restrictions remain unchanged.

## Interrupted confirmed planting

A newly observed bomb still invalidates the old escape route. For a confirmed timed placement,
the controller can retain an opaque origin record from the route's exact current pose and matching
live bomb, then submit ordinary Stop. Fresh planning uses the current checked observation and
unchanged forecast to prove short escape pulses. All seven admission lanes, their stopped waits,
movement and stopped suffix retain the normal lethal/uncertainty checks. Conditional motion keeps
the three-update cap and named stationary assumptions. Guidance selects a legal exit from the
fixed origin through the shared refuge search. Every admitted pulse must improve its worst-case
endpoint distance toward that exit; necessary turn alignment counts as progress. The selected
guidance never authorizes movement without a fresh pulse proof.

The origin exception applies only where floor, Oil, soft-block and bomb checks prove that the
matching sole bomb contributes the blocking interval. Every lane permanently withdraws the
exception at footprint exit. Actor/loadout/effect, pose, own-bomb signature, world or command
changes withdraw provenance; unknown bomb vision also prevents its use. A confirmed route is the
source of this record, rather than arbitrary overlap or private simulation state. Legacy planting does not issue it. An Awaiting certificate can issue origin only when a
unique newly observed bomb matches its exact certified birth, actor and pose. Missing, ambiguous,
mismatched or explicitly absent placement grants no record. Whole-scene confirmation and route
invalidation remain strict even when that independent birth check succeeds.

Rules8 uses the same recovery with a fresh combined bomb-hazard and mob-contact pulse proof.
Concurrent rival planting does not strand a CPU on its own bomb. Scheduled route stops retain
contact validation, and an invalidated contact lease cannot authorize the old route or pulse.
Foreign bomb-motion uncertainty may use the existing named stationary assumptions for short
recovery pulses. Every intersecting assumption is retained in the certificate, movement stays
capped at three updates, and observed displacement invalidates execution. This does not extend
origin grace beyond footprint exit or waive a rival bomb's actual collision.

The controller retains the record only through checked pulse or settled Stop observations.
Worker results must match its still-active origin before input or RNG can commit; late or cancelled
results cannot restore a cleared record. Queue lateness alone does not withdraw valid stopped
provenance. Recovery emits movement or Stop without another manual action. New information can
still withhold movement, and future enemy actions can invalidate a fresh pulse.

## Held Wall Pass

One capability check requires rules v5, enabled Wall Pass and a visible living recipient holding
it. Static guidance and certified motion may cross known intact soft blocks or begin stopped
inside one. Hard walls, bounds, bombs, mines, fire, uncertainty and known unlit Oil remain strict;
the opponent conservatively avoids Oil even though the game capability permits passage.

Forecast construction omits only intact-block collision for that recipient. A predicted block
destruction adds blocking from its destruction tick through the horizon, including after finite
flames expire: concealed pickups or Oil cannot be assumed absent. Combined hazard checks and
blast geometry are unchanged. Crossing an intact block neither reveals nor collects its contents.
Equipment changes invalidate execution; the policy never invents private exit grace.

Opening goals remain legal empty floor. Wall Pass holders preserve the capability instead of
seeking or crossing replacement equipment; Heart holders likewise retain Heart. Duplicate
Wall Pass remains harmless ordinary transit. Weapons use their independent slot policy.
Planting inside a block is refused. The first certified move may leave a block and plant on free
floor; active Wall Pass uses complete timed planting proof rather than the legacy shortcut.

## Focused verification

Rules8 planning additionally requires the finite mob-contact lease described in
[multiplayer CPU boundaries](multiplayer-cpu.md#lair-profile-boundaries). A lease binds the
recipient/world and Teleporter marker, models public spawn deadlines and treats frozen mobs as
capable of moving without a disclosed thaw deadline. Ordinary movement and a plant's longer
escape retain separate horizons; speculative tactic exhaustion cannot discard the precomputed
movement fallback. Teleporter guards include full landing cells and maximum retained travel,
with work charged to the same fixed allowance. A changed marker withdraws execution authority.

Future rival bombs, Bracelet actions and other unexpected scene changes remain reasons to
invalidate and reobserve. Finite contact certification does not claim universal survival against
unobserved future enemy inputs.

Run `cargo test -p grenadiers-core cpu:: --lib` for bounded contracts,
`cargo test -p grenadiers-core cpu_observation --lib` for recipient privacy and public facts,
and `cargo test -p grenadiers-server cpu_contract --lib` for accepted-input replay. These
focused commands supplement the unchanged full Rust/coverage release gates.
