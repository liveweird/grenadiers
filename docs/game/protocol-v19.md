# Gameplay v19: dormant Hard CPU contract

Gameplay19 is a dormant scaffold and is not accepted on `/api/v1/game/ws`. Its sources are
[`api/gameplay-v19.schema.json`](../../api/gameplay-v19.schema.json), the generated
`web/src/game/v19/schema.ts`, and the standalone v19 server-frame validator. Production remains
on gameplay18 until the Rust protocol adapter and Hard decision policy land together.

## Identity-only extension

V19 changes the gameplay identity from 18 to 19 and widens the closed `Difficulty` enum to
`easy | standard | hard`. It retains rules10, all board and frame bounds, closed fields,
environment settings, lifecycle rules, recipient privacy, and scalar movement physics from v18.
There is no fallback from Hard to Standard.

`RoomConfiguration.cpuDifficulty` is null exactly when `controllers` contains no CPU. Otherwise
it is required, and every published CPU `SeatStatus.controller.difficulty` must equal the room
value. Mixed CPU difficulties and private planner data are rejected.

## Activation boundary

The dormant web selector has three localized choices and defaults to Standard, but no live UI
imports it. Activation requires a Rust v19 adapter, Hard policy integration, Rust-produced
client/server conformance corpora, authentication negotiation for 19, and replacement of the
v18 generated validator import. The production bundle must replace v18 with v19 rather than
shipping both validators. Current fixtures exercise the closed web boundary only; Rust producer
proof remains pending.
