# Changelog

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
