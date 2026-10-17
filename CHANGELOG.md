# Changelog

## 0.3.8 — 2026-10-17

- Stage the v4 browser with private in-memory seat capability, bounded fresh-session retries,
  generation fences, full admission baselines, and preserved bomb-action high-water.
- Present localized connection/grace/held-result status and disable controls before recovery.
- Retain authority after rejected leave and return to a usable lobby after terminal recovery.

The panel remains dormant; active gameplay switches with the coordinated server release.

## 0.3.7 — 2026-10-17

- Stage private, independently minted resume capabilities with bounded collision retries and
  redacted diagnostics, plus exact connection/generation binding fences.
- Verify entropy/collision failure atomicity and stable seat capability matching.

Helpers remain test-compiled until coordinated owner/socket activation.

## 0.3.6 — 2026-10-17

- Check v4 room projection against actual lifecycle connectivity, grace, scores, and history.
- Validate physical state epochs for held/canceled rounds and final outcomes, plus private
  admission identity, full-state nullability, and bomb-action high-water context.

These APIs are dormant; active gameplay remains v3 until coordinated reconnect activation.

## 0.3.5 — 2026-10-17

- Stage the closed gameplay-v4 reconnect contract with distinct redacted resume capabilities,
  personal admitted baselines/action high-water, shared room updates, and connection status.
- Add strict Rust/AJV conformance, three-version generated-type drift checks, physical v3
  parity, and maximal complete-frame bounds for snapshots and private admissions.

The contract is dormant. Active gameplay remains v3 until checked projection and coordinated
server/browser reconnect activation are verified.

## 0.3.4 — 2026-10-17

- Integrate reserved-seat grace into the pure match lifecycle with atomic two-seat loss and
  expiry settlement, strict resume boundaries, and eligible-participant forfeits.
- Hold completed rounds through disconnects, cancel inter-round countdowns without consuming
  actual rounds, and start fresh countdowns after return.
- Preserve round scoring before expiry, third-win finality, terminal reservations, and rematch
  reset while retaining the legacy v3 transport-loss interruption path.

The new APIs remain inactive in the network adapter until the v4 server/browser switch.

## 0.3.3 — 2026-10-17

- Capture accepted production match operations in private bounded in-memory journals.
- Preserve immediate input fences, TTL refreshes, actual round seeds, and atomic shutdown/
  departure ordering; rejected commands never enter the journal.
- Mark construction failures incomplete and capacity exhaustion truncated without changing
  gameplay. Replay refuses to certify either as a complete record.

Journals are process-local evidence, released with their owner. Replay/inspection are test-only;
there is no public export, durable storage, credential recording, or reconnect activation.

## 0.3.2 — 2026-10-17

- Add a private, bounded semantic replay journal and runner using the real match runtime.
- Validate format/rules/generator versions, command chronology and bounds, exact round seeds
  and assignments, missing/extra starts, and explicit truncation.
- Compare full private world, lifecycle, input, reports, and revision checkpoints through real
  generated draws, wins, rematch, cancellation, interruption, and immediate input fences.

This runner is test-only. Live accepted-operation capture follows separately; no replay
endpoint, durable storage, credential recording, or reconnect activation is introduced.

## 0.3.1 — 2026-10-17

- Add a pure bounded two-seat reconnect-grace model with checked 600-tick deadlines.
- Reject expired/stale resumes atomically, retain original duplicate-loss deadlines, and expose
  batched expiry facts without deciding match outcomes.

This is a reconnect foundation. Active gameplay v3 still interrupts on transport loss;
credential, lifecycle, server, and browser activation follow separately.

## 0.3.0 — 2026-10-17

- Activate authoritative two-player first-to-three matches over gameplay protocol 3.
- Carry scores and completed-round history through automatic result holds, fresh worlds,
  alternating spawns, and inter-round countdowns; draws award no score.
- Add consensual rematches with withdrawable votes, fresh scores/readiness, and retained
  connection sequencing; distinguish round results from final forfeits and interruptions.
- Activate localized accessible match scores, phase clocks, final outcomes, and rematch controls.
- Drain upgraded game and diagnostic sockets explicitly on shutdown, delivering the final
  checked match view before close within a bounded deadline.
- Add an isolated generated-map browser fixture for a real draw, three wins, and rematch.

## 0.2.13 — 2026-10-17

- Add the dormant bounded match owner with generation-fenced admissions, two-seat departure
  batches, fixed-tick scheduling, lobby/rematch deadlines, and final-match retirement.
- Signal shutdown independently of the command queue and retain one checked final publication
  with actual vacancy, pending reports, and preserved match finality.
- Reject canceled admissions before mutation and settle overloaded prematch leaves promptly.

Production registry/socket wiring and coordinated browser activation follow in v0.3.0.

## 0.2.12 — 2026-10-17

- Stage the protocol3 browser client, match scores/timers, completed-round and final-match
  presentation, and authoritative two-player rematch consent.
- Accept newer-epoch snapshots even when intermediate room views are coalesced; reset
  controls/action IDs while preserving transport sequencing.
- Share the existing Pixi arena and preserve authoritative vacancy over frozen actors.

The new client/panel remains dormant until the bounded owner and coordinated activation land.

## 0.2.11 — 2026-10-17

- Add a dormant authoritative adapter for real multi-round simulation and checked v3 publication.
- Build fresh worlds and input state per actual assignment, flush bounded report windows,
  and freeze physics during result/rematch holds while retaining the authoritative room clock.
- Fence stale input before action IDs, preserve private generation records, and publish
  truthful interruptions when round construction fails.

The adapter is test-only until the bounded owner and coordinated browser/server switch land.

## 0.2.10 — 2026-10-17

- Add checked v3 room projection from actual match lifecycle facts.
- Validate canonical seats, readiness phases, score/history budgets, epoch chronology,
  result holds, rematch eligibility, and independent forfeit/interruption outcomes.
- Reject impossible unanimous-consent snapshots and occupied forfeit losers.

This adapter is dormant; production gameplay remains v2 until runtime/browser activation.

## 0.2.9 — 2026-10-17

- Stage a self-contained gameplay-v3 contract for truthful match scores, actual round count,
  completed-round records, automatic countdowns, match results, and rematch votes.
- Add namespaced Rust DTOs and generated TypeScript, strict conformance/physical-parity
  checks, and dual-version generation/drift checks.
- Guard the new metadata/frame budget with a 44,742-byte maximal-body fixture.

This contract is dormant. Live gameplay remains protocol v2 and diagnostics v1; checked
projection, real adapters, and coordinated activation follow separately.

## 0.2.8 — 2026-10-17

- Add a pure match lifecycle with initial and automatic inter-round countdowns, result holds,
  actual-round assignment, first-to-three termination, and consensual rematch reset.
- Separate prematch interruption from post-start match outcomes, batch same-tick departures,
  retain the global clock, and make transition failures transactional.
- Invalidate departed players' rematch consent and reject consensus with a vacant opponent.

The lifecycle is staged alongside the live single-round runtime. Gameplay v2 remains active;
truthful v3 contracts/adapters and coordinated activation follow separately.

## 0.2.7 — 2026-10-17

- Add bounded headless first-to-three match-series facts: scores, actual round assignment,
  immutable latest completed round, and a separate final match outcome.
- Preserve result holds, batched forfeit/abandonment/interruption precedence, and finality.
- Add idempotent, withdrawable two-player rematch votes after the 180-tick hold, with retained
  command-epoch/time high-water checks and atomic overflow failures.

Live generated gameplay remains the single-round v2 preview. Automatic orchestration, v3
contracts, browser match UI, and full first-to-three acceptance follow as focused increments.

## 0.2.6 — 2026-10-17

- Activate private generated-v1 arenas with classic-v2 pickups and upgrades in live rooms.
- Draw fresh round entropy only at an actual start; retain private generation records and
  keep authentication/discovery randomness independent.
- Add a strict read-once operator seed-file mode that fails startup on invalid configuration.
- Prove generated-map smoke behavior and deterministic keyboard collection in real browsers,
  including simultaneous old-range/new-range bombs and both authoritative upgrade HUDs.

The two-player preview still supports a single round. Complete match scoring/rematch is M5.

## 0.2.5 — 2026-10-17

- Draw exposed capacity, range, and speed items as distinct retained Pixi graphics, with
  owned-resource cleanup on replacement, removal, and room reset.
- Show authoritative available bombs, capacity, range, and speed levels for both players;
  distinguish alive/eliminated status from lobby readiness in English and Polish.
- Exercise actual HUD updates and bounded layouts down to 320 pixels in the rendered duel.

Generated live rounds and real pickup collection acceptance follow in the activation release.

## 0.2.4 — 2026-10-17

- Coordinate gameplay protocol v2 across schema, Rust, server, and browser authentication.
- Transmit exposed typed pickups and collection events with checked rules/map identifiers;
  preserve bounded frames and keep concealed/pending contents and seeds private.
- Bind construction and publication labels to an immutable room profile; retain the fixed
  classic-v1 live preview until generated-room activation.

Gameplay v1 clients must reload with the matching v2 server. Diagnostics remain protocol v1.

## 0.2.3 — 2026-10-17

- Collect exposed pickups after damage using final actor positions and deterministic ties.
- Apply permanent capacity/range upgrades up to eight and five speed levels; preserve
  placement-time bomb range and movement residuals. Consume items even at the cap.
- Add explicit round-number spawn and tie-priority assignment, independent of canceled
  countdown attempts. Keep classic-v1 and the fixed live preview unchanged.

Gameplay wire support, browser items, and generated-room activation follow separately.

## 0.2.2 — 2026-10-17

- Replace inert pickup cells with checked typed concealed, pending, and exposed layers.
- Reveal destroyed-block contents after flames clear; burn exposed items without shielding
  downstream blasts, and preserve revealing-blast immunity for pending contents.
- Add generated-world construction with separate private generation records and exposed-only
  public domain views. Freeze every pickup transition after a result.

Collection, upgrades, wire/client support, and generated network rounds follow separately.
The fixed-map playable preview remains unchanged.

## 0.2.1 — 2026-10-17

- Add a versioned deterministic arena API with canonical mirrored soft blocks and typed
  concealed upgrades, clear spawn openings, and validated first-bomb escape routes.
- Pin ChaCha20 initialization and bounded unbiased draw semantics; use a deterministic
  empty fallback on validation or random-draw exhaustion.
- Keep generation records and concealed locations outside public snapshots and redact
  sensitive debug output. Promote the already-locked rand_chacha 0.9.0 dependency.

This M4 groundwork does not activate generated network rooms or pickup behavior. The playable
fixed-map preview remains unchanged while those dependent features are implemented.

## 0.2.0 — 2026-10-17

- Make the fixed-map two-player duel playable through Create/Join, readiness, countdown,
  keyboard movement, bombs, destruction, elimination, and a shared round result.
- Render the authoritative arena with pinned PixiJS 8.22.0 alongside the React/Mantine HUD,
  English/Polish controls, reduced motion, scoped keyboard focus, and retained diagnostics.
- Validate incoming gameplay frames, fence stale async connection work, and keep credentials
  in tab memory. Release graphics contexts, tickers, observers, and listeners on teardown.
- Verify actual WebGL pixels and desktop/narrow layouts in a complete two-browser duel.

This completes M3’s first playable preview. It requires a keyboard and supports one fixed-map
round; generated upgrades, complete match scoring, touch controls, and reconnect follow later.

## 0.1.14 — 2026-10-17

- Run real two-player rounds with readiness, exact countdown, authoritative 60 Hz updates,
  and checked public snapshots/events published at 20 Hz.
- Bind logical movement and bomb actions to the current seat/attempt; expire stale held
  input, reject duplicate bomb IDs, and bound each player’s admitted work per update.
- Preserve completed results, interrupt live rounds on departure or overload, and retire
  terminal rooms after a fixed retention window.

The protocol now supports a full fixed-map round through native sockets. The playable
browser interface and real graphics acceptance follow in the duel preview.

## 0.1.13 — 2026-10-17

- Add bounded room creation and code-based joining with two authenticated seat owners.
- Publish personalized admission and shared membership views, reject full/stale bindings,
  and acknowledge explicit departures before allowing another room on the same connection.
- Bound discovery, actor queues, identifiers, and waiting lifetime; release registry entries
  even when an owner task is canceled before its first poll.

Rooms remain in Waiting in this increment. Readiness, authoritative simulation scheduling,
public snapshots, and the browser game interface follow separately.

## 0.1.12 — 2026-10-17

- Add authenticated gameplay WebSockets with compatibility checks before credential lookup,
  atomic single-owner claims, and credential revocation when the owning connection ends.
- Share the existing connection limit with diagnostics and bound upgrades, input, frames,
  buffers, sends, authentication, inactivity, and absolute unseated lifetime.
- Reject replayed command sequences and expose typed unseated command errors without
  fabricating rooms or gameplay state.

Room ownership/admission and authoritative gameplay scheduling follow separately. This
preview does not support reconnecting with a previously claimed credential.

## 0.1.11 — 2026-10-17

- Add a pure headless two-seat room lifecycle for occupancy, readiness, the exact 180-tick
  countdown, running updates, results, and preview interruptions.
- Fence canceled countdowns with increasing round attempts, clear readiness consistently,
  and preserve terminal outcomes after either explicit departure or transport loss.
- Precheck clock/deadline/attempt arithmetic so errors leave lifecycle state unchanged.

This domain API does not create networked rooms. Authenticated gameplay sockets and room
ownership/routing follow separately.

## 0.1.10 — 2026-10-17

- Add `POST /api/v1/game/session` for anonymous 256-bit credentials, with exact Origin checks,
  closed no-store responses, a bounded in-memory registry, and five-minute unclaimed expiry.
- Limit issuance bursts/rate, handle entropy failures and collisions safely, and omit query
  strings from HTTP tracing.
- Document the real HTTP operation and exercise it through deterministic server tests and
  blackbox browser requests.

Tokens do not yet authenticate a gameplay socket or reserve a room/seat. Gameplay WebSocket
handlers follow in the next increment; existing diagnostic transport remains available.

## 0.1.9 — 2026-10-17

- Add explicit fallible conversion from public simulation state to bounded gameplay wire DTOs.
- Reject invalid clocks, dimensions, identities, ownership counts, ordering, positions,
  upgrades, deadlines, and outcomes without exposing private simulation bookkeeping.
- Convert up to three consecutive update reports into stable indexed events, preserving
  same-tick placement/detonation and accepting legitimate frozen result snapshots.

Authenticated session issuance and gameplay transport follow separately. This adapter
introduces no network route, room, or browser gameplay.

## 0.1.8 — 2026-10-17

- Define bounded public snapshots for the fixed arena, two actors, owned bombs, and flames
  in Rust, JSON Schema, and generated TypeScript.
- Include indexed placement, detonation, elimination, and round-end events without private
  simulation bookkeeping or concealed future contents.
- Validate full nested messages, collection/scalar limits, and actual encoded frame sizes.

This completes the snapshot/event wire shape. Checked domain conversion, authenticated
transport, rooms, and browser play follow separately.

## 0.1.7 — 2026-10-17

- Define strict gameplay authentication, room control, movement/bomb intent, lifecycle,
  result, and typed error messages in one JSON Schema with Rust and generated TS types.
- Add canonical identifiers, bounded integer parsing, required nullable movement, token
  Debug redaction, payload-free errors, and message byte limits.
- Check gameplay type drift in the shared web gate and compare Rust/AJV conformance.

This is the control/lifecycle contract foundation. Snapshot/event wire types and gameplay
endpoints follow separately; the existing diagnostic protocol remains unchanged.

## 0.1.6 — 2026-10-17

- Support bounded strings, Unicode regular-expression patterns, bounded arrays, and
  uniqueness constraints in the explicit realtime type generator.
- Reject malformed, reversed, unsafe, or misplaced constraints while preserving existing
  fail-closed schema handling and byte-identical diagnostic types.
- Add nested contract compilation and validation regressions for gameplay credentials
  and bounded snapshots. Runtime AJV validation continues to enforce these constraints.

This tooling prerequisite starts M3; gameplay endpoints and browser play follow separately.

## 0.1.5 — 2026-10-17

- Add explicit public-state DTOs for the arena, players, bombs, flames, timer, and result,
  excluding private input, movement bookkeeping, collision grace, and allocator state.
- Emit deterministic placement, detonation, elimination, and round-end events in actual
  phase order, preserving placement and explosion within the same tick.
- Complete the headless M2 combat boundary for the future gameplay transport.

Networked rooms and browser gameplay follow in M3. Domain DTOs are independent of
wire schemas and do not serialize the internal world directly.

## 0.1.4 — 2026-10-17

- Add swept contact damage from existing fire and final-footprint damage from new explosions.
- Prevent eliminated players from moving or planting; keep their existing bombs active
  until the round resolves.
- Decide sole-survivor wins, mutual-elimination draws, and the exact 10,800-update timeout
  after all same-tick damage, with elimination taking precedence over timeout.
- Freeze gameplay after a result while the absolute simulation clock continues.

This remains headless: public gameplay projections and events are the last M2 slice;
networked rooms and the playable browser arena follow in M3.

## 0.1.3 — 2026-10-17

- Execute bomb fuses and same-tick chains against one immutable occupancy snapshot.
- Stop cardinal blast rays at walls, soft blocks, and other bombs; remove hit soft blocks
  after planning the complete chain.
- Add 30-tick flames with exclusive expiry, overlapping lifetime extension, and bomb ignition.
- Release bomb capacity once, available for placement on the following tick.

This headless increment adds explosions and flames. Player damage, round outcomes,
public gameplay snapshots, and browser play follow separately.

## 0.1.2 — 2026-10-17

- Add owned bomb placement after movement, one live bomb per player, captured range,
  and 180-tick detonation deadlines. Fuse execution follows in the next increment.
- Resolve eligible simultaneous placements deterministically; failed requests consume
  no capacity or entity ID and never carry into another tick.
- Preserve collision grace for every overlapping player until they leave the bomb.
- Replace the unused ownerless obstacle API with `InputAction::PlaceBomb`.

This remains a headless engineering prerelease. Explosions, damage, and browser play
are not implemented yet.

## 0.1.1 — 2026-10-17

- Verify the Rust/React foundation, locked builds, coverage, dependency/image scans,
  Docker Compose, and local Kubernetes operation.
- Add the deterministic headless arena/movement model, turn assistance, logical input
  priority, swept collision, bomb exit exemptions, and repeatable state comparisons.
- Specify the classic two-player rules and phased implementation plan.
- Select PixiJS for future arena rendering alongside the React/Mantine interface.

This engineering prerelease has no playable browser arena or combat yet. The browser
continues to expose transport diagnostics. See `docs/verification.md` for observed results.
