# Archived gameplay protocol guides

Only gameplay v11 is accepted by the live server. These guides preserve the historical meaning
of retained wire schemas and regression fixtures; they do not promise that an older client can
join the current server. See the [current protocol](../../game/protocol.md),
[live v11 guide](../../game/protocol-v11.md), and
[retention decision](../../decisions/0021-gameplay-retention.md).

| Version | Historical guide | Support status |
| --- | --- | --- |
| v3 | [Gameplay-v3 match contract](protocol-v3.md) | Archived |
| v4 | [Resumable match contract](protocol-v4.md) | Archived |
| v5 | [Forgiving turns](protocol-v5.md) | Archived |
| v6 | [Shared room rules](protocol-v6.md) | Archived |
| v7 | [Recipient state and manual actions](protocol-v7.md) | Archived |
| v8 | [Expanded rooms and optional pickups](protocol-v8.md) | Archived |
| v9 | [Recipient-private state and action freeze](protocol-v9.md) | Archived |
| v10 | [Teleporter epochs and later equipment](protocol-v10.md) | Archived |

Gameplay v2 has no standalone guide in this repository. Its retained schema is
`api/gameplay.schema.json`; [ADR 0013](../../decisions/0013-gameplay-v2-and-room-profiles.md)
records its contract decision.
