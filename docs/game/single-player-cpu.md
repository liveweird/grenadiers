# Single-player: human versus CPU

This is the accepted design boundary for the staged hybrid chosen on 2026-10-25 after comparing
an independent proposal with [Claude's original proposal](../history/single-player-cpu-proposal.md).
It defines invariants and acceptance, not delivery status. Remaining sequencing and open work live in the
[product backlog](../../BACKLOG.md#single-player).

## Chosen approach

Build a deterministic Rust bot with scored goals and time-aware routing. A private match has
one human and one server-controlled CPU, using the existing two-seat simulation and ordinary
input/action validation. Keep campaign mobs, larger matches and learned policies outside this
initial implementation.

The bot policy is a pure component: observable state and bounded memory go in; ordinary input
intent comes out. A server adapter owns its lifecycle, timing, controller identity and input
submission. It does not mint a fake human session or bypass bomb limits, cooldowns, curses,
collision, damage or equipment rules.

## Comparison and decision

Both proposals favored a server-side deterministic bot, fair observations, danger prediction,
goal scoring and incremental development. The selected design keeps that common foundation.

| Area | Original proposal | Selected refinement |
| --- | --- | --- |
| Prediction | Rebuild a sandbox `State` from the recipient view | First define a restricted observation/forecast API; prove which rule kernels can be reused without private state. |
| Hazards | Next burn tick and duration per cell | Preserve all relevant danger intervals, moving occupancy, safe gaps and persistent fire. |
| Difficulty | Different prediction depth, reaction and mistakes | Share correct known-hazard mechanics; vary reaction, goals and bounded search. |
| Scheduling | Inline or separate task left open; example millisecond target | Keep planning off the authoritative tick's critical path, with bounded queues/work and stale-result rejection; measure time separately. |
| Delivery | Product integration after the main AI/item phases | Connect a real solo room early, then improve the same playable slice. |
| Verification | Fixture tests, tournaments and timing checks | Add observation-privacy, replay-input and work-bound guarantees; keep expensive balance/benchmarks separate from routine feedback. |

The original already forbids cloning the authoritative state. The concern is its reconstruction
assumption: recipient snapshots omit movement residuals, grace/cooldown histories and other
simulation bookkeeping. Existing `State::new*` constructors reset internal fields; they are not
lossless snapshot restoration APIs. A sandbox needs a deliberate partial-information model.

## Observation and prediction boundary

- Observe only what the CPU seat may know under the current recipient/privacy rules. Avoid
  coupling the bot to an archived protocol adapter; use a neutral bot observation and a server
  adapter for the current live projection.
- Never expose concealed pickups/Oil, hidden bombs, a cloaked rival's private details, arena
  seeds or opponent intent. Bounded memory may retain previously observed facts and the bot's
  own actions; reset it at round/room identity boundaries.
- Reuse pure movement/explosion rule kernels where their inputs are sufficient. Represent
  unknowns explicitly; never silently fill a full simulation with invented authoritative facts.
- Forecast lethal intervals and occupancy over time, including chains, flame expiry, moving
  bombs, Oil ignition and persistent fire, modifiers and pauses. A tile may burn more than once.
- Routing must account for waiting, effective speed, continuous actor footprint and the actual
  movement/turn rules. Use a search suited to timed transitions; a cell-only shortest path is
  insufficient.
- Plan a bomb only when an escape exists under the observed facts and chosen forecast. This is
  a planning guarantee, not a promise that an opponent or newly revealed hazard cannot kill it.

## Decisions, scheduling and replay

Score survival, useful pickups, opening terrain and offensive pressure. Survival has priority;
tactical search extends this policy rather than creating a separate rules implementation.
Use each item's existing semantics: Shield and Teleporter apply on collection, stored weapons
change Bomb behavior, and Equipment invokes the existing equipment action.

Give decisions a fixed work allowance, such as node expansions and bounded horizon. Use a
bounded planning queue/worker arrangement so the room tick never waits for an unfinished
search. Results carry room, round and observation identity; discard stale results and avoid
unbounded catch-up queues. Planning performs no external I/O and participates in room shutdown.
Choose concrete work limits through measurement, not an assumed sub-millisecond CI budget.

Use a separate deterministic RNG stream for tie-breaking/imperfection; never consume arena
randomness. Record the accepted bot inputs through the existing replay path. Store policy,
difficulty and seed metadata compatibly when needed; replay should consume recorded actions,
not rerun whichever bot version is installed later.

All difficulty levels obey the same physics and knowledge boundary. Tune reaction cadence,
goal quality, aggression and search allowance. Balance tables and win-rate targets need
measured fixtures and playtesting; they are not correctness guarantees.

## Room and product contract

- Add an explicit solo room mode and CPU label/difficulty. The human occupies seat one, CPU
  seat two; retain the current two-seat core and global room/admission/resource limits.
- The CPU's controller is server-owned. A remote client cannot claim its seat or forge its
  actions. Update public mode/seat metadata with the repository's contract-first compatibility
  process rather than disguising the CPU as a network participant.
- Define creator settings, start, rematch, transient reconnect and permanent departure in the
  first vertical slice. Preserve bot state across human reconnect; close the CPU room when its
  human permanently leaves. Record the exact disconnect/pause policy before implementation.
- Mark future history/stats as CPU matches. Durable stats/storage are not prerequisites.
- Develop narrow mechanic fixtures first, but support the all-features-on product default
  before a public solo preview. Advanced item tactics may improve later; legal play, privacy,
  correct effects and bounded execution must work throughout.

### Lifecycle policy

The human in seat One owns settings. The CPU in seat Two supplies ordinary revision-checked
readiness after admission, settings changes, canceled initial countdowns and rematch reset.
Human readiness starts the existing 180-tick countdown. After the rematch eligibility deadline,
the CPU consents while the human is connected; the human must still opt in. Consensus returns
the room to Waiting with readiness cleared.

During a running transport loss, clear human input immediately and continue physics and CPU
decisions through the existing 600-tick reservation grace. Preserve CPU memory and randomness.
Keep existing result holds and inter-round countdown cancellation/restart behavior; disconnect
does not pause physics or extend reservation deadlines. Before the first start, and for a new
disconnect after the final match result, the existing lifecycle releases the human immediately;
close the solo room. A reservation established before finality retains its original deadline.

On explicit departure or grace expiry, settle the ordinary result/forfeit precedence before
canceling CPU work and releasing the room. Do not submit a simultaneous CPU departure that
would turn a human forfeit into abandonment. Room retention follows the human's eligibility;
the CPU alone cannot keep a room alive.

New physical rounds and rematches reset round memory and pending inputs. Socket replacement
alone preserves memory and RNG. Command epochs are a separate fence: countdown cancellation
can invalidate pending decisions without creating a new physical world.

<a id="milestones"></a>

Remaining CPU work is maintained in the
[product backlog](../../BACKLOG.md#single-player), the sole source for delivery status.

## Acceptance

Fast deterministic tests cover escape, multiple burn windows, chains, movement/turning, Oil,
modifiers, item categories and lifecycle resets. Identical observation histories and bot seed
must produce identical intent despite different hidden authoritative facts. Keep ordinary
input validation, recorded-input replay determinism and bounded-work exhaustion tests.

Use seeded tournaments for balance and isolated reference workloads for performance. Do not
add hundreds of slow rounds or hardware-sensitive one-millisecond assertions to every routine
change. Public acceptance includes human-versus-CPU start/result/rematch/reconnect, all-on item
rules, privacy, localization and narrow-screen accessibility, with existing floors and no
hidden retries.

## Interrupted-escape continuation boundary

The [controlled Short Fuse investigation](../history/cpu-short-fuse-investigation-2026-11-01.md)
shows that repeated three-update movement pulses with intervening Stops can be too slow after
simultaneous bombs invalidate an escape. CPU44/45 therefore use a distinct opaque conditional
continuation certificate for an interrupted confirmed planting escape while the CPU still has its
own-origin exception. This does not enlarge existing pulse/route authority or infer stationarity
from bomb ownership. Ineligible or failed continuation attempts retain the existing checked
short-pulse recovery.

The certificate separates a complete nominal trajectory of at most eight legs, including planned Stops, and 96
movement updates from renewable movement grants. A fresh publication at `P` authorizes only
updates `P+1…P+3`. Validation runs before the owner services `P+4`;
missing, stale, duplicate or mismatched publications withdraw continuation. Admission binds an
exact stopped basis `B`: the owner observes the actual `B+3` and `B+6` publications, admits at
`B+6`, and submits the first certified input for `B+7`. Publication phase follows the Driver's
actual accumulated publication counter, not a global-tick modulus; an unexpected publication
revokes the contract.

Valid renewals carry predicted motion, residual distance, pending turn assistance, reverse state,
pit state and storm bounds. They preserve planned geometric-leg stops and consumed origin grace;
re-entry cannot restore origin grace. The owner commits a grant only after physical mailbox
submission succeeds. Rejection or invalidation submits ordinary Stop through that mailbox and
requires a physical observation fence before normal planning resumes. A lease or TTL cannot extend
the three authorized updates, and the first accepted continuation command prevents the superseded
origin record from being resurrected.

The proof certifies a Stop branch after every nominal update, including early revocation after the
first or second move. A regular grant's fallback Stop at `P+4` covers the existing margin through
`P+13` (exclusive horizon `P+14`). Complete nominal escape and every Stop branch need full
footprint, hazard and contact checks within the unchanged planning allowance. Each admission or
renewal has one fail-closed 8,192-work allowance and rechecks scene, actor/effects, bombs, contact,
Teleporter, pit and storm facts without new path search or a global forecast on the owner tick.

Fresh short-window Teleporter/contact obligations are checked at every grant; a long-lease refusal
cannot be bypassed. The current extension is limited to visible, unpaused standard timed bombs,
completed running updates, supported effects and no initial finite flames, Oil, exposed Teleporter
or Diarrhea. Remaining soft blocks may exist. Lairs-off and lairs-on scenes use their respective
honest contact modes. Unknown or changed hazards revoke the episode, and a late result cannot
resume a revoked episode.

CPU44 serves Easy/Standard and CPU45 serves Hard. Their non-origin decisions delegate to the
archived CPU42/43 behavior; CPU42/43 and private replay21 remain closed, while private replay22
binds the continuation identities. A [scheduling fixture](../history/cpu-recovery-timing-2026-11-01.md)
using synthetic grants establishes timing only; it is not a safety certificate. Remaining
improvements and broader scene support stay in the [backlog](../../BACKLOG.md#single-player).

## Intentional Teleporter collection boundary

CPU48 serves Easy/Standard and CPU49 serves Hard under private replay24. Their ordinary fallback
is exactly CPU44/45's result, memory and RNG; only the additional selector work is charged. An
active confirmed-origin record bypasses the new selector and retains CPU44/45 continuation
behavior. The existing planting, pulse and contact-topology guards still reject exposed
Teleporters. Deliberate collection instead uses a separate typed `TeleporterStep` certificate,
without weakening those guards, assigning curses globally positive pickup value or allowing
ordinary navigation to transit a Teleporter.

Each step authorizes one movement update, an unconditional Stop on the next physical update,
and a fresh confirmed stopped observation before further movement. Ordinary checked navigation
through the existing `SearchMap`/BFS reaches a stable neighboring cell center. A target-specific
**Approach** step may overlap only the named Teleporter with the footprint while every possible
final center remains outside its cell. Repeated Approach steps end at a pose where **Collect**
enters that cell in one update. Positions are never snapped, and an unchanged public pose does
not synthesize a stopped residual or clock.

For a stopped basis `B`, check every start in `[B+1, B+8)` and safety through exclusive `B+18`.
Keep the controller's actual physical Stop-confirmation tick `S <= B`; continued held Stop supplies
the stopped-motion premise. Any admitted direction, even one blocked by collision, clears that
provenance. Admission requires the same retained `S`, not a replacement or a clock inferred from
unchanged positions; Easy cadence may deliberately wait after confirmation.
For CPU48/49, an unchanged visible dead player remains an inert roster member. The sealed Stop
still takes effect at admission `A+2`; its confirmation may arrive in a later snapshot before
the original safety deadline. Only then does strategic progress commit. Archived CPU46/47 and
replay23 retain their original admission/confirmation behavior.
Cover partial or blocked movement, no collection, refused swap, consumed-without-swap and every
possible living partner, including humans and bounded moving-partner outcomes, without reading
private randomness. Swap has two endpoints and no connecting sweep; the same public markers and
opposite endpoint poses identify it. It resets both actors' residual and pending turn, preserves
headings and leaves the partner's held input active. The root's Stop never stops the partner.

The kernel requires exactly one exposed Teleporter, a complete visible living roster, visible
empty bombs and no flames, Oil, Bracelet, active origin, unsupported effects, immediately usable
Remote or Landmine, or reachable invalidating pickup. Both lair modes use honest checked contact.
Marker-aware sealed contact containment permits natural births and deaths without fabricating an
observation. Bound arbitrary rival movement, unknown residual and contested collection. Check
complete landing footprints and post-teleport mob, mine and storm consequences. Every pit site is
excluded from all prefixes and landing regions; the proof binds fresh pit layout and settings,
while irrelevant phase changes may occur. New bomb blocking remains possible, and a short proof
window excludes detonation only with a checked minimum-fuse argument. Unsupported landing regions
are rejected rather than treating engine destination validity as safety. The certificate promises
only the root CPU's safety under observed support, not a partner's future self-survival or safety
against future unseen enemy actions.

Retain the ordinary fallback's result, memory and RNG. Attempt new recovery only from the remaining
shared 65,536 work, capped at 32,768, after computing that fallback first. Use one forecast and
freeze at most two candidate slots after generation; charge generation, forecasts, every
start/partner branch and all failures. Prepared admission and Stop validation are each fail-closed
within an 8,192-work cap and perform no new search or forecast. Bind actual world/seat/actor,
stopped pose, settings, capabilities/effects/markers, pickup, topology, hazards and the full living
roster.
Memory, RNG and authority commit only after actual mailbox success. Missing observations, expiry,
submission rejection, world or epoch changes, or changed facts revoke the step and apply a physical
input queue fence before ordinary planning resumes.

The high-level goal retains its commitment and progress deadlines. Only observed BFS-cell or
subcell progress refreshes the progress deadline. On expiry it tries one alternate approach; an
invalid or stalled alternate ends that recovery episode instead of oscillating. Later fallback or
new world facts may make a future episode eligible, so this is not a permanent no-revisit promise.

The live roster owner executes the normal neighboring-center approach, repeated steps, a real swap
with matching markers and opposite endpoint poses, physical Stop, fresh completion and a later
independently certified useful plant that destroys the named block while the CPU survives its
flame. This path is shared by Standard, Easy and Hard with lairs on or off through the native 60 Hz
Driver/worker/queue integration. The legacy owner remains test-only and preserves its archived
profile and rejection behavior. Existing Teleporter refusals and archived policies remain closed.
Unsupported scenes and general passivity are intentionally not solved; broader farming/contact
stalls and further Hard tactics remain in the [backlog](../../BACKLOG.md#single-player).

## Plant-bound contact topology

The contact-envelope refinement uses public barriers to exclude unreachable space while
retaining the existing distance envelope wherever separation is not proved. Use conservative
eight-neighbor connectivity and a one-cell eight-neighbor dilation of every source component.
Compare every cell touched by the complete swept actor footprint, not only its center.
Freezing, occupancy, likely mob death and presumed fire avoidance cannot close reachable space.

Initially constrain the proof to an ordinary full plant certificate with visible empty initial
bombs, no finite flames or Oil, no exposed Teleporter and completed running updates. Use shared
standard-blast geometry for its hypothetical placement. Open every potentially removed block
from the beginning; retain only separators proved intact throughout the whole horizon. A blocked
birth location may be omitted only when its block is retained; potentially opened birth cells
contribute from their original earliest birth time.

An empty reachable source set is valid only when the original envelope is nonempty and every
source is a possible birth whose soft block remains retained. Existing mobs cannot be omitted.
This case needs no component-dilation or route sweep: there is no source that can reach it.
Renewal still checks the plant scene and every retained barrier, and rejects any observed mob.
This additional capability is opt-in; older profiles reject its certificate at admission.

An opaque `PlantContactCertificate` binds the full plant certificate to its original contact
envelope, source components and separator evidence. `ExecutionCertificate::ContactPlant` is
admitted by Core before the owner can use it instead of the ordinary contact lease. Committed
execution keeps one bomb-presence mapping; a confirmed bomb cannot silently become absent.
Expected detonation and predicted block removal are supported without restarting the proof.

Admission and every publication check the plant scene, actor compatibility, original source
envelope and retained separators before committing updated presence. Unexpected bombs,
displaced/replaced bombs, early disappearance, separator loss, unsupported observations or
exhausted work withdraw the refinement. The next simulation update must precede the original
exclusive horizon, even while stopped or waiting for a queued movement. Do not apply this
projection to an observed owned bomb merely because it belongs to the CPU.

Try the ordinary contact lease first; only `NoRoute` can request the refinement. Charge
construction, source scans and footprint queries inside the existing per-candidate child budget,
then try alternate movements when a candidate fails. Planning retains its 65,536-work ceiling;
each admission/publication has one shared, fail-closed 8,192-work renewal allowance, with no
fresh path search. Unsupported evidence keeps the old conservative result.

Policy42/43 retain the nonempty-source path and enable blocked-birth proofs without changing
archived planners; lairs-disabled behavior delegates to policy40/41. Safety remains conditional
on the plant's observed scene assumptions: withdrawal at the first mismatched publication does
not predict arbitrary unseen rival actions.
Physical tests, natural-spawn tests, real queued-owner execution and independent review are
required alongside geometric separation tests.
