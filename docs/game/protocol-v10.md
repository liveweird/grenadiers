# Gameplay v10: authoritative teleport position epochs

The coordinated live contract is `api/gameplay-v10.schema.json`, Rust `core::protocol::v10`, and
generated `web/src/game/v10/schema.ts`. Deploy the paired server and browser assets together.
Gameplay v9 is archived and rejected before credential lookup, so an old client cannot claim or
revoke a credential for a v10 connection.

V10 retains Expanded-v5 rules, generated-v2 arenas, journal format 2, all v9 commands, room fields,
bombs, events, inventory, privacy and voluntary-action freeze semantics. Its independent catalog
adds Teleporter as the seventeenth item and accepts all 131,072 masks. Archived v9 remains pinned
to its sixteen-item catalog and rejects Teleporter in settings, pickups and event-only history even
when the live enabled mask omits it.

Every visible actor adds required nullable `lastTeleportTick`. A successful atomic Teleporter swap
sets the same marker on both actors. The marker must be after the physical round start and no later
than `physicalTick`, and paired visible actors must agree. It persists through pause, reconnect,
death and results until a new round state is constructed. A hidden actor remains exactly
`{type,id,seat,alive}`; its position, loadout, effects and marker remain private.

The server validates the complete canonical world, bounds, enabled catalog and history before
recipient filtering, including an audience of neither seat. It prepares both bounded recipient
frames before publication state or revision commits, and admissions remain event-free. Replay
checks both perspectives, state, reports and markers. A client accepts a newer marker only after
the ordinary round/revision/tick freshness checks, then snaps both authoritative positions without
distance heuristics, interpolation or synthetic audio.

Authenticate with `protocolVersion: 10`; successful authentication reports version 10 and an
incompatible client receives `supportedVersion: 10`. Client messages remain capped at 4 KiB and
server frames at 64 KiB. A state remains bounded to two actors, 16 bombs, 195 flames, 195 exposed
pickups and 96 events. The server write buffer remains 128 KiB. Event indexes remain recipient-local
and contiguous after filtering.
