# Gameplay v5: forgiving turns

Archived reference; the live endpoint uses [v11](protocol-v11.md).

Product candidate v0.3.16 activated `api/gameplay-v5.schema.json`, Rust
`core::protocol::v5` and generated `web/src/game/v5/schema.ts` together.

V5 retains v4 messages, personal admission, ten-second reserved-seat reconnect, bounds and
semantic validation. Its rules-version enum adds classic-v3: the generated classic-v2 rules
with an inclusive 256/1,024-tile turn-assistance window instead of 128/1,024. Correction still
consumes ordinary movement distance and uses swept collision and flame contact. It does not
buffer turns outside that window or change speed, bomb timing, scoring or arena generation.

New production rooms identify rules version 3 and generated-v1 maps. The movement engine reads
the active immutable rules configuration; historical classic-v1/v2 keep their original window.
Replay headers identify v2 or v3 explicitly and select the matching simulation at playback.

The closed v4 contract cannot describe rules version 3. Its schema and generated TypeScript
remain archived and unchanged; the active implementation moves to v5. Unsupported gameplay
versions, including 4, receive `incompatibleProtocol` with `supportedVersion: 5` before any
credential claim or seat admission. Reload the page after upgrading. Diagnostic protocol v1,
HTTP paths, sessions and the independent product version are unchanged.

The [v4 field reference](protocol-v4.md) remains useful for unchanged fields; v5's schema is
authoritative for active messages. Shared gameplay settings and additional abilities remain
[backlog work](backlog.md), requiring a subsequent explicit contract/rules revision.
