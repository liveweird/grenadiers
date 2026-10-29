# Gameplay v16 lair contract

`api/gameplay-v16.schema.json`, `core::protocol::v16` and generated
`web/src/game/v16/schema.ts` define the checked rules8 profile. The production WebSocket
endpoint is `/api/v1/game/ws` and requires gameplay16; versions 1 through 15 are rejected before
credential claim. New-room defaults enable cloud and lairs; either environment feature may be disabled.

## Configuration

Configuration and complete settings both require:

```json
{"environment":{"stormCloud":true,"lairs":{"enabled":true,"spawnSeconds":10,"maxLivingMobs":3}}}
```

Their environment values must agree, including in room-only and state-free admission frames.
All fields remain required when disabled; numeric choices survive disabling and reenabling.
Spawn intervals accept 5–60 whole seconds and limits accept 1–8 living mobs per lair. Base
rules, clouds and lairs share one settings revision. Controller, board, creator editing grant,
readiness, lifecycle, physical-round and recipient-privacy invariants remain explicit.

Rules8 supports two to four configured seats with at least one human. Generation remains
version3 on 15×13 and version4 on 13×11/11×9; the wire map ID remains `generated_v2`.

## Lairs and mobs

State retains the cloud and actor fields from gameplay15 and adds `lairs` and `mobs` arrays.
Disabled lairs require both arrays empty. Enabled lairs require the two canonical hard-wall
cells from [layout1](lairs.md), ordered by IDs0 and1. Each reports `cell`,
`spawnRemainingTicks` and `livingMobs`. Remaining ticks are bounded by the configured interval;
the two attempt deadlines stay synchronized. Counts must match the listed mobs and per-lair cap.

At most16 mobs report their globally unique entity `id`, `lairId`, fixed-point `position`,
cardinal `heading` and `frozen` boolean. Complete footprints must fit the board. IDs cannot
collide with actors or bombs. No pursuit target, path, random seed, concealed pickup or private
Stopwatch deadline is published. A visible living Stopwatch holder establishes freezing;
hidden holders remain private rather than supplying inferred expiry times.

## Publication and consumers

Capture accepts actual rules8 state. Binding checks exact rules/settings, generation,
canonical overlay, lifecycle and physical epoch before preparing recipient frames. The CPU
adapter consumes that same checked publication and constructs `Observation::new_rules_v8`.
Historical constructors and transport validators remain closed to rules8; shared neutral
geometry and lifecycle helpers do not relabel it as an older profile.

Rust and browser conformance use a shared corpus produced from actual roster lifecycle
publications. Run `cargo test --locked -p grenadiers-core protocol::v16 --lib` and
`npm --prefix web test -- src/game/v16` for focused checks. Regenerate gameplay types and
standalone validators through the existing `gen:gameplay` and `gen:validators` commands.
