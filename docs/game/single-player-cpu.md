# Single-player: human versus CPU

This is the accepted design boundary for the staged hybrid chosen on 2026-10-25 after comparing
an independent proposal with [Claude's original proposal](../history/single-player-cpu-proposal.md).
It defines invariants and acceptance, not delivery status. All sequencing and open work,
including the prerequisite review and CPU0–CPU4 packages, live in the
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

The former milestone section is maintained as scoped CPU0–CPU4 work in the
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
simultaneous bombs invalidate an escape. A recovery extension must introduce a distinct conditional
continuation contract; do not enlarge existing pulse/route authority or infer stationarity from
bomb ownership.

Separate a complete nominal trajectory from renewable movement grants. A fresh publication at
`P` can authorize only updates `P+1…P+3`. Validation must run before the owner services `P+4`;
missing, stale, duplicate or mismatched publications withdraw continuation. Admission binds an
exact stopped basis and publication at `basis+6`, with the first movement at `basis+7`.
Publication phase follows accumulated reports, so an early publication must revoke the initial
contract rather than silently assuming a global tick modulo.

Carry predicted motion, residual distance, pending turn assistance, reverse state and storm
bounds across valid renewals. Preserve planned geometric-leg stops, consumed origin grace and
the existing eight-leg limit. Re-entry cannot restore origin grace. Commit each grant only
after mailbox submission succeeds; TTL cannot extend its authorized updates.

Certify Stop branches after each authorized update, including early revocation after the first
or second move. A regular grant's fallback Stop at `P+4` must cover the existing margin through
`P+13` (exclusive horizon `P+14`). Complete nominal escape and every Stop branch need full
footprint/hazard/contact checks within the unchanged work allowance. Renewal must have an
explicit measured bound, without new path search or a global forecast on the owner tick.

Retain default item support through fresh short-window Teleporter/contact obligations at every
grant; a long-lease refusal cannot be bypassed. Initially constrain the extension to visible,
unpaused standard timed bombs, completed running updates, supported effects and no initial finite
flames, Oil or exposed Teleporter. Remaining soft blocks may exist. Unknown or changed hazards
revoke the episode, and a revoked episode cannot resume from a late result.

Before activation, require a separate reviewed Core proof, normal queued-owner integration, both
Short Fuse CPUs planting and surviving, and negatives for intermediate unsafe Stops, stale
publications, changed bombs, displacement, origin re-entry and exhausted work. A scheduling
fixture using synthetic grants establishes timing only; it is not a safety certificate. Keep
new policy/replay identities opt-in and every archived contract unchanged. Work sequencing remains
in the [backlog](../../BACKLOG.md#single-player).
