# Gameplay v9: recipient-private actors, bombs and voluntary-action freeze

The coordinated server/browser contract is `api/gameplay-v9.schema.json`, Rust
`core::protocol::v9`, and generated `web/src/game/v9/schema.ts`. Version 8 is archived and rejected
before credential lookup, so an old client cannot claim or revoke a credential for a v9 connection.
Deploy the paired server and browser assets together.

Gameplay v9 retains the same Expanded-v5 rules identity, generated-v2 maps and journal format 2.
Version 0.3.41 adds Invisible Bombs to v9's independent sixteen-item catalog and supports all 65,536
masks. All 32,768 earlier masks and generator paths remain unchanged. Archived v8 settings and
projection rules stay pinned to the fourteen-item Cloak-era catalog and reject Stopwatch and
Invisible Bombs facts.

Visible actors carry the required boolean `voluntaryActionsFrozen`. A living actor is frozen exactly
when the living opponent has an active Stopwatch; both actors may be frozen. The status is derived
from both complete actors before recipient filtering.
Hidden actors remain exactly `{type,id,seat,alive}`, so recipient filtering does not reveal another
actor's position, inventory, effects, timers or freeze cause. Dead visible actors must also remain
unfrozen. Timed bombs may be paused only when Stopwatch is enabled and authoritative private timing
facts prove their original fuse and accumulated paused ticks. Remote and Contact bombs are untimed.

The existing capture and binding path still validates the complete world and report window before
recipient filtering. V9 converts the validated visible actor into its version-specific shape, then
prepares both recipient encodings before publication state can advance. Admission likewise prepares
its recipient-specific, event-free body before ownership or replacement is committed. The 4 KiB
client-frame, 64 KiB server-frame, 16-bomb and 96-event bounds are unchanged.

Event-side timing and audience facts are captured when an event occurs rather than inferred from
the final state. Archived v8 rejects both live and event-only Stopwatch or pause history, including
a resumed Timed bomb whose public paused flag is currently false.

An actor affected by Invisible Bombs receives an empty live-bomb list while retaining truthful
capacity counts and public flames. Bomb-event audiences intersect Cloak ownership privacy with the
seats blinded at emission and may be `Both`, one seat, or `Neither`. Other event audiences retain
the Cloak policy. Complete hidden bomb, timing and event facts are validated before filtering.
