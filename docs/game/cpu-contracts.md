# CPU contracts

These Rust contracts support the [solo opponent](single-player.md) and the broader
[accepted design](single-player-cpu.md). The
[backlog](../../BACKLOG.md#single-player) owns delivery status.

## Controllers and ordinary inputs

`core::room::RoomMode` derives controller kinds independently of occupancy and socket state.
Human duels have two human controllers. Solo fixes the human at seat One and the CPU at seat
Two. `CpuProfile::CURRENT` identifies `UtilityV5` with `Standard` difficulty;
`INITIAL`, `PULSE_V2`, `ROUTE_V3` and `PLANT_V4` preserve the V1–V4 replay identities. These are
policy identities, not claims about measured strength. Seeds are outside participant metadata.
Live owner construction accepts only the current profile; retaining a historical replay
identity does not re-enable that older policy for new matches.

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
fixture policies remain test-only; production uses UtilityV5. Replays consume recorded accepted
actions rather than invoking a policy again. Production journal allocation remains disabled;
explicit solo recording adds format-3 header provenance while preserving formats 1/2 and
accepted-input record capacity.
Policy versions 1/2/3/4/5 are accepted only with their matching profile; new recordings retain the
actual room profile. All five policies share the unchanged v12 room shape.

## Goal guidance

A single bounded static flood-fill records reachable distance and the first step toward useful
pickups, block openings and a visible rival. Pickup value depends on observed upgrades, current
equipment and remaining buff duration. Useful upgrades, Heart and beneficial effects can be goals;
capped upgrades, duplicates and unnecessary refreshes have no collection priority.

Known curses, Oil, Teleporter and unsupported stored weapons are excluded from deliberate goals
and ordinary navigation transit. Harmless capped upgrades or duplicate/long-duration buffs remain
transitable. Equipment shares one replaceable slot: navigation preserves held Heart rather than
replacing it with an unused capability. Remembered ordinary next steps use the same eligibility
check. Stable ordering and the independent RNG resolve genuine ties; exhausted work yields Stop.

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
soft blocks or unlit Oil are rejected instead of relying on temporary collision; this remains
conservative when WallPass or a predicted explosion changes passability during a held input.

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

## Focused verification

Run `cargo test -p grenadiers-core cpu:: --lib` for bounded contracts,
`cargo test -p grenadiers-core cpu_observation --lib` for recipient privacy and public facts,
and `cargo test -p grenadiers-server cpu_contract --lib` for accepted-input replay. These
focused commands supplement the unchanged full Rust/coverage release gates.
