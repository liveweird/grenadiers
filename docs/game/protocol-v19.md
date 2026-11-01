# Gameplay v19: room-wide CPU difficulty

Gameplay19 is the live paired contract on `/api/v1/game/ws`. Its sources are
[`api/gameplay-v19.schema.json`](../../api/gameplay-v19.schema.json), the generated
`web/src/game/v19/schema.ts`, and the standalone v19 server-frame validator. Authentication requires protocol19; older clients must reload the paired application.

## Identity-only extension

V19 changes the gameplay identity from 18 to 19 and widens the closed `Difficulty` enum to
`easy | standard | hard`. It retains rules10, all board and frame bounds, closed fields,
environment settings, lifecycle rules, recipient privacy, and scalar movement physics from v18.
There is no fallback from Hard to Standard.

`RoomConfiguration.cpuDifficulty` is null exactly when `controllers` contains no CPU. Otherwise
it is required, and every published CPU `SeatStatus.controller.difficulty` must equal the room
value. Mixed CPU difficulties and private planner data are rejected.

## Runtime and compatibility

The lobby offers three localized choices and defaults to Standard. Every CPU in a room uses
the selected tier. Easy/Standard select CPU31; Hard selects CPU32. Rules10, generator3/4,
navigation3 and all movement/damage settings remain shared.

Runtime publication identity is explicit and immutable. Every newly created live room uses19,
including human-only rooms. Archived constructors and replay10–12 retain18; no Hard room is
projected through an Easy/Standard shadow. Retained v14–18 wire types and replay readers reject
Hard rather than downgrading it. Private replay13 retains Hard28 and replay14 retains Hard29/Easy/Standard27. Replay15 binds gameplay19 and the current Hard32 or Easy/Standard31 pair.
Production recording/export remains disabled.

The browser ships only the v19 generated runtime validator and retains the same semantic,
privacy, lifecycle, world, lair and spike-pit checks. Shared archived types are compile-time
references; they do not load an additional generated validator into the production bundle.
