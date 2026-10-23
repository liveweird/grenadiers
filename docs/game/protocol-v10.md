# Gameplay v10: Teleporter epochs and Wall Pass

The coordinated live contract is `api/gameplay-v10.schema.json`, Rust `core::protocol::v10`, and
generated `web/src/game/v10/schema.ts`. Deploy the paired server and browser assets together.
Gameplay v9 is archived and rejected before credential lookup, so an old client cannot claim or
revoke a credential for a v10 connection.

V10 retains Expanded-v5 rules, generated-v2 arenas, journal format 2, all v9 commands, room fields,
bombs, events, inventory, privacy and voluntary-action freeze semantics. Its independent catalog
adds Teleporter and Wall Pass as the seventeenth and eighteenth items and accepts all 262,144
masks. Archived v9 remains pinned to its sixteen-item catalog and rejects either newer item in
settings, equipment, pickups and event-only history even when the live enabled mask omits it.

Every visible actor adds required nullable `lastTeleportTick`. A successful atomic Teleporter swap
sets the same marker on both actors. The marker must be after the physical round start and no later
than `physicalTick`, and paired visible actors must agree. It persists through pause, reconnect,
death and results until a new round state is constructed. A hidden actor remains exactly
`{type,id,seat,alive}`; its position, loadout, effects and marker remain private.

The server validates the complete canonical world, bounds, enabled catalog and history before
recipient filtering, including an audience of neither seat. Wall Pass exit grace is canonical,
private state: each live or dead actor may retain at most four in-bounds soft-block cells that its
footprint still positively overlaps, and no grace count, timer or marker appears on the wire. It
prepares both bounded recipient
frames before publication state or revision commits, and admissions remain event-free. Replay
checks both perspectives, state, reports and markers. A client accepts a newer marker only after
the ordinary round/revision/tick freshness checks, then snaps both authoritative positions without
distance heuristics, interpolation or synthetic audio.

Authenticate with `protocolVersion: 10`; successful authentication reports version 10 and an
incompatible client receives `supportedVersion: 10`. Client messages remain capped at 4 KiB and
server frames at 64 KiB. A state remains bounded to two actors, 16 bombs, 195 flames, 195 exposed
pickups and 96 events. The server write buffer remains 128 KiB. Event indexes remain recipient-local
and contiguous after filtering.
