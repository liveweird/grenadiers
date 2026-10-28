# Gameplay protocol v14

Gameplay v14 is a dormant transport contract for closed two-to-four-seat rooms. It is not routed
by the live server or web client yet. The canonical JSON Schema is
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
it is `null` for an all-human room and `easy` or `standard` when any CPU is configured. Supported
boards are exactly 11×9, 13×11, and 15×13. The checked Rust DTO constructs the domain
`RoomConfiguration`, so parser acceptance and domain invariants are identical.

The immutable checked configuration is repeated in every `RoomView`. Seat statuses, scores, and
actors retain every configured participant, including eliminated actors. Seats are local to v14
and comprise `one`, `two`, `three`, and `four`; archived protocol seat types remain unchanged.
Scores use the canonical array form `[{ "seat": "one", "wins": 0 }]` in seat order.

Rules versions 1–5 remain two-seat rules. Version 6 permits two to four actors. Map versions 1–3
remain available. Published arena dimensions and terrain length must match the configured board;
the current runtime capture supports 15×13 only until smaller-board activation is implemented.

## Privacy and limits

The publication pipeline validates the complete physical state and report history before applying
recipient filtering. Cloaked actors and invisible bombs remain recipient-private. Private
per-actor movement history and grace state travel with that actor rather than in parallel arrays.
All recipient frames are encoded before a runtime publication may be committed.

Client messages are limited to 4,096 bytes. V14 server messages are limited to 98,304 bytes,
without changing archived limits. A state contains at most 32 bombs, and a snapshot contains at
most 192 events with indexes 0–191. At most three consecutive physical reports are published.

## Regeneration and checks

From `web/`, run `npm run gen:gameplay` and `npm run gen:validators` after changing the schema.
Use `npm run check:gameplay` and `npm run check:validators` to verify checked-in artifacts. Rust
contract tests compare the v14 parser with the schema conformance fixture.
