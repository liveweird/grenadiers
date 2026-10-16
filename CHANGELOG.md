# Changelog

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
