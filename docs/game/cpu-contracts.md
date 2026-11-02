# CPU contracts

These Rust contracts support the [solo opponent](single-player.md) and the broader
[accepted design](single-player-cpu.md). The
[backlog](../../BACKLOG.md#single-player) owns delivery status.

## Controllers and ordinary inputs

`core::room::RoomMode` derives controller kinds independently of occupancy and socket state.
Human duels have two human controllers. Solo fixes the human at seat One and the CPU at seat
Two. Live selection uses `CORNER_REFUGES_V56_STANDARD` or `CORNER_REFUGES_V56_EASY` and
`HARD_V59`; private replay30 binds those identities. Archived replay29 retains CPU56/58. Named
archive constants preserve historical replay identities. Policy identity describes decision
semantics, not measured strength; seeds stay
private. Live owner construction accepts only those three exact selected profiles. Retaining a
historical replay identity does not re-enable that older policy for new matches.
Controllers carry an explicit seat identity through memory, recipient validation and ordinary
input submission. Foreign-recipient observations are rejected before state changes. Current
solo construction still authorizes only seat Two; an explicit identity does not grant a human
seat CPU input authority.

`core::cpu` accepts neutral observations and emits movement or Stop, with at most one Bomb or
Equipment intent. Intent carries no credentials, seat authority, readiness, settings mutation
or manual action ID. The server allocates those IDs and uses ordinary input
validation; tests exercise accepted actions through the existing runtime and replay path.

## Information boundary

The current-protocol adapter converts the already filtered v19 recipient projection. It must
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
Foreign unpaused timed-bomb motion may use the existing named stationary assumptions for short
recovery pulses. Every intersecting assumption is retained in the certificate, movement stays
capped at three updates, and observed displacement invalidates execution. This does not extend
origin grace beyond footprint exit or waive a rival bomb's actual collision.

The controller retains the record only through checked pulse or settled Stop observations.
Worker results must match its still-active origin before input or RNG can commit; late or cancelled
results cannot restore a cleared record. Queue lateness alone does not withdraw valid stopped
provenance. Recovery emits movement or Stop without another manual action. New information can
still withhold movement, and future enemy actions can invalidate a fresh pulse.

CPU44/45 add an opaque continuation certificate for this same confirmed-origin state. The worker
proves one complete nominal escape of at most eight legs, including planned Stops, and 96 movement updates, plus a
safe Stop after every nominal update. If continuation is ineligible or cannot be proved inside the
shared decision allowance, recovery falls back to the existing checked origin pulse. Non-origin
decisions delegate to CPU42/43 rather than changing their planner behavior.

For an exact stopped basis `B`, the Driver's actual publication counter supplies observations at
`B+3` and `B+6`; admission occurs at `B+6` and the first input is for `B+7`. A publication at `P`
can authorize only `P+1…P+3`, and a new checked publication is required before `P+4`. This phase is
not derived from a global tick modulus. Missing, duplicate, stale or mismatched publication, command
epoch, scene or pose withdraws authority. Each admission or renewal has one 8,192-work allowance and
performs no fresh path search or global forecast.

Renewal checks the expected actor motion and effects, bomb evolution, full footprint, hazards,
contact, short-window Teleporter obligations, spike pits and storm bounds. It carries residual
movement, pending turn assistance, reverse state and consumed origin grace across grants. Origin
grace cannot return after exit, and the first accepted continuation command makes the superseded
origin record unavailable. The current scene boundary requires completed running updates, visible
unpaused standard timed bombs, supported effects, and no initial finite flames, Oil, exposed
Teleporter or Diarrhea. Both lair-disabled and lair-enabled rooms use the corresponding finite
contact mode; this is not broader item support.

The owner commits each grant only after its physical mailbox submission succeeds. Rejection,
renewal failure or changed public facts fail closed to an ordinary Stop submitted through the same
mailbox, followed by a physical observation fence before ordinary planning resumes. A late result
cannot restore revoked authority, and a lease cannot extend a grant beyond its three updates.

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

## Rules9 occupancy and respawn profile

CPU25 receives checked v17 facts through `new_rules_v9`, with no lair overlay when disabled.
Visible living players are blockers through the same fixed-point cell/footprint geometry as
physics. Fresh next-segment checks supplement pose/epoch/scene guards without requiring every
unrelated rival to stand still. Hidden positions remain unknown. Corner alignment and forward
motion are checked as separate axis segments; future unexpected blocking withdraws authority.

`plan_respawn_v24_with_origin` keeps finite movement/contact and planting proofs, including with
lairs disabled. Occupied lairs contribute possible replacement envelopes after earliest death
plus grace; empty lairs use independent due times. Combat goals seek reachable firing positions
against all visible rivals with shared weapon/effect blast geometry. The real-State regression
includes a human elimination, CPU-versus-CPU lethal planting and the planter's escape.

## Extra mine observations

Rules9 ordinary-slot counts exclude Contact mines, while forecast ownership/removal history keeps
all visible mines. Distinct bomb cells bound observations to the board (at most 195). Work budgets
and per-cell interval limits are unchanged: an excessively uncertain scene fails closed rather
than accepting an unproved route. Archived observations and slot accounting remain closed.

## Rules10 spike-pit profile

Archived CPU26 receives checked gameplay18 facts through `new_rules_v10`. Its
`SPIKE_PITS_STANDARD` and `SPIKE_PITS_EASY` profiles model every public pit as a path-dependent
clock: a route can trigger an idle pit, leave it and encounter its later warning or raised phase.
Forecasts use the same contact geometry and checked phase transitions as simulation.

Committed execution carries a guard for the observation tick, exact public pit configuration
and every expected phase/countdown. A changed pit state withdraws authority and forces a fresh
observation. Raised contact is rejected unless the recipient's published damage grace or active Shield
still covers it. This does not introduce a separate immunity or expand the existing work,
movement, planting, stopped-suffix or survival budgets. Rules9/CPU25 remains archived unchanged.

## Live difficulty profiles

Gameplay19 retains the same rules10 observation boundary. Easy/Standard select CPU56 and Hard
selects CPU59. They retain CPU52/53's ordinary planning and pit avoidance, adding the bounded
blocked-away refuge hints described below. Multi-Teleporter selection and confirmed-origin
shared preparation remain unchanged. Archived CPU54/55 delegate their non-origin planning
exactly to CPU52/53. Archived CPU50/51
compute their ordinary fallback first and preserve exactly CPU44/45's result,
memory and RNG; only the typed Teleporter selector consumes additional remaining work. Confirmed-
origin continuation for those archived profiles delegates unchanged to CPU44/45. The selected profiles retain the same shared
65,536-work budget and full planting, route, mob-contact and admission proofs. Hard keeps CPU45's
range-aware firing positions against visible rivals and does not receive hidden positions,
concealed loot, private seeds or faster physics.

Hard pressure uses an optional shared allowance of at most 4,096 work units and one eighth of
remaining work, with at most 32 origins per rival. Failure falls back to existing collect/farm
guidance. Strong useful pickups retain their values; attack pressure outranks routine farming.
Retained goals keep commitment/progress deadlines. Admission rechecks the exact target: a rival
moving out of the certified blast withdraws planting usefulness.

CPU29 permits clear-arena counter-pressure only from completed-update observations: no finite
flames or exposed Teleporter, visible unpaused timed bombs and no soft blocks. Empty-bomb
farming retains its prior support. This changes a Teleporter support precondition, not the
route proof, public observation facts or work limit; all conservative motion checks remain.

CPU50/51 inherit CPU44/45 and CPU42/43's one-update pickup handoff on rules10 even though
ordinary clear navigation is disabled there. Full forecast, spike/storm, contact, confirmed Stop
and fresh observation requirements remain. The longer ordinary pulse still rejects physics-
changing collection; Teleporter and Diarrhea remain outside this handoff. Archived CPU27–29 are
unchanged.

CPU50/51 retain CPU44/45 and CPU42/43's refinement of a failed mob-contact envelope for an
ordinary plant when public barriers separate every source from the complete certified escape.
Potentially destroyed blocks are treated as open throughout. The opaque plant-bound certificate
retains scene, bomb-presence, source and barrier checks at admission and each publication;
unsupported changes withdraw it.
When every original source is a possible birth blocked by a retained soft wall, the new profiles
can prove an empty reachable source set. Every birth barrier must remain intact; any observed mob
withdraws that authority. Archived CPU40/41 reject this additional certificate capability.
It does not authorize recovery around an already observed bomb. See the
[contact topology contract](single-player-cpu.md#plant-bound-contact-topology).
Private replay30 binds CPU56/59; archived replay29 binds CPU56/58; replay28 binds CPU56/57;
replay27 binds CPU54/55; replay26 binds CPU52/53; replay25 binds CPU50/51; replay24 retains
CPU48/49; and replay23 retains CPU46/47. CPU44/45 and private replay22 are archived alongside
CPU42/43 and private replay21; replay20 retains CPU40/41. CPU50/51 ordinary fallback delegates exactly to
CPU44/45, including confirmed-origin continuation, while its new selector is inactive for an
origin record. During continuation, lairs-disabled and lairs-enabled scenes use distinct checked
contact modes. Older policy/replay contracts remain isolated.

## Shared escape preparation

CPU54/55 and CPU56/57/58/59 validate the actual retained plant origin and build one unchanged default forecast
against the shared remaining work before attempting optional continuation. A forecast that
exceeds the old half allowance can therefore still support a checked recovery pulse.

Let incoming remaining work be `A`, its old half allowance `H = floor(A/2)`, and common
preparation cost `C`. When `C <= H`, optional continuation receives exactly `H-C`, preserving
the old successful-preparation choices and work. Otherwise it receives at most half the
post-preparation remainder. Generation retains its existing cap and two frozen proof slots.
Charge all failed attempts, then reuse the same forecast for the existing pit-safe pulse
fallback. No work is refunded, no second forecast or RNG draw is introduced, and the total
decision ceiling remains 65,536.

Common preparation or fallback exhaustion fails closed. Actual Stop, scene, bomb, contact,
Oil, pit and publication checks remain unchanged; Oil still excludes full continuation where
it did previously. CPU44–53 retain their original allocation and results. This does not relax
the separate foreign-bomb planting/future-loot support guard.

A completed rules10 pulse waits for a publication at its actual admission tick before
the controller attempts strict admission. Waiting retains the pending result and authorizes no
input. The original six-tick freshness limit, selected start window and world/command checks
still apply; changed pit state, pose or contact facts can reject the plan. This shared controller
correction also applies to archived policies. Planning results and playback of existing accepted-
input journals retain their contracts; newly played matches can accept input at different ticks.

## Farming when the usual retreat is blocked

CPU56/57/58/59 can use a lateral refuge when the cell directly away from an adjacent farming target
is an indestructible wall, lair or outside the board. This requires the CPU to occupy its current
committed Farm approach, visible empty bombs and no flames. Two deterministic candidates move
sideways first, then use refuges offset by the observed bomb range both sideways and toward the
target. The geometry is only a hint: complete planting, route, target-usefulness and mob-contact
proofs still authorize the action.

Selection and proof share the existing `min(8,192, remaining / 8)` child allowance. Failed work
is charged to the parent, no RNG is consumed, and legacy hint order/work remain intact for
archived policies. This does not relax the separate foreign-bomb/future-loot support guard.
Safe complete planting routes may still cross pits under their existing time-dependent proof.

## Voluntary pit avoidance

Current profiles inherit CPU52/53's exclusion of every observed pit site, regardless of phase, from ordinary search maps before
reachability. Pickup, wall-farming and rival-pressure approaches also reject pit cells, including
retained goals and routes. An already-on-pit search root cannot become a new pit approach merely
because reachability seeds the current cell. No-pit scenes retain their corresponding tactical
base: farm38/39 with lairs, spike26/hard28 without them.

Before accepting an ordinary short pulse, pickup handoff, opening move or origin-recovery pulse,
check its complete swept footprint envelope for new pit contact. Refusal happens while selecting
candidates so safe alternatives remain available. All checks consume the existing shared work
allowance. Fully certified planting and continuation routes retain their complete time-dependent
pit proofs and can cross pits safely; this restriction does not replace them. A CPU already on a
pit retains checked recovery behavior, with no promise that forced placement is escapable.

The multi-Teleporter selector and physical executor are retained. Their existing proof already
excludes pit sites from movement prefixes and landing regions. Foreign-bomb planting support,
future-loot and contact obligations remain unchanged. Archived policies and replay formats retain
their earlier behavior.

## Typed Teleporter recovery

Multiple exposed Teleporters are supported only when every unselected pickup is beyond the
maximum full-window travel bound of every visible living actor’s basis position, including the
root. Swapping permutes occupied positions and resets movement residual; checking all origins
therefore also excludes another activation after a partner lands at the root’s former position.
Other reachable pickups, contested collection and unsafe landings remain unsupported.

Guidance builds one map/BFS, scans charged neighboring approaches and keeps a valid prior target.
Otherwise it chooses reachable approach distance, then target row/column. It freezes one target
and two movement candidates before the single forecast/proof attempt; failure does not try more
targets. Unselected pickups must remain unchanged at admission and completion. CPU48/49 retain
their exact-one-target contract; CPU50/51 reuse their physical executor.


Planting, ordinary pulse and contact-topology guards continue to reject exposed Teleporters.
CPU50/51 may instead remove one selected exposed Teleporter with a separate `TeleporterStep` after
the ordinary CPU44/45 fallback has already been computed. Existing `SearchMap`/BFS navigation
approaches a stable neighboring center without giving curses positive global value or using a
Teleporter as ordinary transit. Target-specific Approach steps permit footprint overlap only with
the named target while the center remains outside; Collect enters its center cell.

Each certificate covers one directional update, unconditional Stop on the next physical update,
and a fresh physical stopped observation before any further movement or planning. For stopped
basis `B`, every start in `[B+1, B+8)` is proved through exclusive `B+18` using the retained
physical Stop tick `S <= B`; a direction admission clears `S` even if collision blocks movement.
The executor never synthesizes stopped provenance from unchanged positions or a global clock.

An already-dead visible roster member must retain identical public actor facts; resurrection or
any changed facts reject the step. A recorded physical Stop still occurs at admission `A+2`.
Its fresh confirmation may be observed later: the retained confirmation tick must lie between
`A+2` and the current observation, within the original exclusive safety deadline. This commits
strategic progress without an unnecessary invalidation fence. CPU46/47 retain their older
all-alive admission and exact Stop-confirmation timestamp checks.

The proof covers every possible living swap partner, including humans, plus blocked, partial,
no-move, no-swap, consumed and bounded moving-partner outcomes. A swap binds unchanged public
markers and opposite endpoint poses. It stops the root while preserving the partner's held input.
Both endpoint footprints retain mob, mine, pit and storm checks. Every pit site is excluded from
prefixes and landings; fresh pit layout/settings remain bound while irrelevant phases may change.
Sealed marker-aware contact containment permits natural births and deaths without inventing
observations. The guarantee is root safety under observed supported facts, not the partner's future
self-survival or protection from unseen future enemy input.

Eligibility requires the selected Teleporter, the complete visible living roster, visible
empty bombs, no flames or Oil, and no Bracelet, unsupported effects, immediate Remote or Landmine,
active origin, or reachable invalidating pickup. Both lair modes are checked honestly. New bomb
blocking and other changed facts can still invalidate the proof.

The selector uses only remaining shared work, capped at 32,768, one forecast and at most two frozen
candidate slots; every success and failure path is charged. Admission and Stop validation are
each fail-closed within an 8,192-work cap, with no new search or forecast. Actual mailbox success
precedes memory, RNG and token commit. Missing observations, expiry, rejection, world/epoch
mismatch or changed facts revoke authority and cause a physical queue fence.

The stable high-level goal keeps its deadlines unless observed BFS-cell or subcell progress occurs.
Expiry tries one alternate approach; an invalid or stalled alternate ends that recovery episode.
Fallback or later world facts can make a later episode eligible. Unsupported scenes and general
passivity remain unsolved; broader farm/contact stalls and further Hard tactics remain in the
[backlog](../../BACKLOG.md#single-player).

## Equal-distance opponent exit coverage

Hard58 first retains every visible rival’s existing nearest useful firing-position candidate;
Hard59 retains this behavior unchanged, using the previous seat order, sequential budget shares
and coordinate tie-break. Only afterward
may it compare other origins at exactly that navigation distance. A strictly larger count of
currently open adjacent terrain branches covered by the exact blast can replace the baseline.
Equal coverage keeps the coordinate choice; a farther origin never wins this tie-break.
This is present branch coverage, not a guarantee about rival movement or survival.

Optional ranking requires a visible, alive, unfrozen and exactly centered rival, visible-empty
bombs, no flames/Oil and no exposed Teleporter. Hidden living actors make comparison unsupported.
Branch walkability uses the rival’s enabled WallPass, never the CPU’s equipment: floor is open,
intact soft blocks require that rival’s enabled WallPass, and walls/lairs are closed. Adjacent
pickups, pits, mobs or overlapping visible actors defer to the existing baseline. Lairs or storm
elsewhere do not add knowledge or relax complete contact and escape proofs.

Placement-time range/modifiers and Standard, Piercing or Power geometry use the existing shared
blast visitor. Every classification and traversal consumes the original shared pressure budget:
at most4,096 units and one eighth of incoming remaining work. Optional work also stays within
each rival’s original unused share; baseline plus optional origin probes remain capped at32 per
rival. Incomplete or unsupported comparisons retain the baseline. No extra RNG, hidden inputs,
commitment change, faster physics or larger decision budget is introduced. The selected approach
still requires the full useful planting, absent/present escape, contact and fresh-publication
checks before any bomb action is admitted. Archived Hard57 retains its original choice and work,
while archived replay29 freezes Hard58's exit-coverage output.

## Equal-distance visible block yield

Hard59 starts with the existing selected visible block target and baseline adjacent farming
approach. It may compare only other eligible adjacent approaches for that same target and at
exactly the same reachable navigation distance. A candidate replaces the baseline only when the
existing placement-time blast geometry reaches strictly more currently visible soft blocks.
Equal yield keeps the baseline, a farther origin never wins, and target ordering is unchanged.

Eligibility retains the existing transit, pit and failed-approach exclusions and rejects a pickup
on the candidate cell. Ranking is unsupported with unavailable or nonempty bomb knowledge,
flames, Oil or any exposed Teleporter. Yield uses the current timed range and Standard, Piercing
or Power blast visitor; it counts visible soft blocks, not concealed contents or possible future
drops. Unsupported or incomplete geometry and work exhaustion retain the baseline.

The comparison consumes only the existing pressure remainder: at most 4,096 work units and one
eighth of the incoming remaining work. It introduces no larger decision budget, new RNG draw,
hidden observation, target switch or planting certificate. A retained same-target approach stays
stable while eligible and preserves its commitment/progress deadlines; if it becomes invalid,
progress is measured against the replacement rather than the stale origin. Every actual bomb
still requires the unchanged usefulness, absent/present escape, contact, admission and fresh-
publication checks. Archived Hard58/replay29 keep the coordinate-selected farming approach.
