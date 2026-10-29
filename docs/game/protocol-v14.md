# Gameplay protocol v14

Gameplay v14 is the archived transport contract for closed two-to-four-seat rooms. It is rejected
before credential claim; the live server and web client use gameplay v16. The canonical JSON Schema is
[`api/gameplay-v14.schema.json`](../../api/gameplay-v14.schema.json).

## Room configuration

`createRoom` replaces the legacy `mode` field with a required `configuration` object:

```json
{
  "type": "createRoom",
  "sequence": 1,
  "configuration": {
    "controllers": ["human", "cpu", "human", "cpu"],
    "cpuDifficulty": "standard",
    "board": { "width": 15, "height": 13 }
  }
}
```

There are two to four controllers and the creator is always human. `cpuDifficulty` is required:
it is `null` for an all-human room and `easy` or `standard` when any CPU is configured. Contract
boards are exactly 11×9, 13×11, and 15×13, with 15×13 as the default custom selection. The
checked Rust DTO constructs the domain `RoomConfiguration`, so parser acceptance and domain
invariants are identical.

The immutable checked configuration is repeated in every `RoomView`. Seat statuses, scores, and
actors retain every configured participant, including eliminated actors. Seats are local to v14
and comprise `one`, `two`, `three`, and `four`; archived protocol seat types remain unchanged.
Scores use the canonical array form `[{ "seat": "one", "wins": 0 }]` in seat order.

Rules versions 1–5 remain two-seat rules. Live two-seat 15×13 rooms select rules 5, generator 2,
and CPU policy 16. Version 6 permits two to four actors: 15×13 three/four-seat rooms use generator
3, while every 11×9/13×11 room uses generator 4; both use CPU policy 17. See
[board geometry](board-sizes.md). Published arena dimensions must match the configured board and
terrain contains exactly `width × height` entries.

## Privacy and limits

The publication pipeline validates complete physical facts and historical events before applying
recipient filtering. Every cell validates its column and row separately before row-major
flattening, preventing an out-of-range column from aliasing the next row. Cloaked actors and
invisible bombs remain recipient-private. Private
per-actor movement history and grace state travel with that actor rather than in parallel arrays.
All recipient frames are encoded before a runtime publication may be committed.

Client messages are limited to 4,096 bytes. V14 server messages are limited to 98,304 bytes,
without changing archived limits. Wire schemas retain the 195-cell global maximum while each
state carries only its selected board's terrain. A state contains at most 32 bombs, and a snapshot
contains at most 192 events with indexes 0–191. At most three consecutive physical reports are published.

Private replay formats 4 and 5 retain simulation randomness 1, derivation 1 and lifecycle profile
3; format 5 additionally binds generator 4 and small-board geometry. Production replay recording
and replay export remain disabled.

## Regeneration and checks

From `web/`, run `npm run gen:gameplay` and `npm run gen:validators` after changing the schema.
Use `npm run check:gameplay` and `npm run check:validators` to verify checked-in artifacts. Rust
contract tests compare the v14 parser with the schema conformance fixture.
