# Gameplay v9: Stopwatch and voluntary-action freeze

The coordinated server/browser contract is `api/gameplay-v9.schema.json`, Rust
`core::protocol::v9`, and generated `web/src/game/v9/schema.ts`. Version 8 is archived and rejected
before credential lookup, so an old client cannot claim or revoke a credential for a v9 connection.
Deploy the paired server and browser assets together. After deployment, reload clients so every
connection uses the server's current protocol version; do not mix v8 browser assets with a v9
server.

Gameplay v9 retains the same Expanded-v5 rules identity, generated-v2 maps and journal format 2.
Version 0.3.40 adds Stopwatch to its independent fifteen-item catalog and supports all 32,768 masks.
All 16,384 earlier masks and generator paths remain unchanged. Archived v8 settings and projection
rules stay pinned to the fourteen-item Cloak-era catalog and reject Stopwatch facts.

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

At the 21 October 2026 20:56:41 UTC v0.3.40 native checkpoint, corrected source `fa25467`
activated Stopwatch and passed all 32,768 settings subsets in debug, release and coverage plus
fresh 52 Compose and 52 Kubernetes cases, with zero retries or carried passes. The paired
server/browser deployment and reload used current gameplay v9 assets; archived v8 rejection and
the exact hidden-actor shape remained covered. Independent raw audit is clear and 2,434 payloads are durably sealed; hosted release gates
remained pending at that checkpoint. Published/local v0.3.39 remained the release baseline.

At the historical 21 October 2026 18:44:23 UTC native acceptance checkpoint, frozen source `272b950` and
image `9a4dc108` passed all 51 Compose and 51 Kubernetes cases with zero retries or carried
passes. The matrix covered the required visible false field, the exact four-field hidden shape,
current-version connection handling, and the unchanged fourteen-item catalog. Independent raw
audit is clear and the evidence is durably sealed. At that checkpoint, v0.3.39 had not been
normalized, pushed, merged, published or promoted; candidate, post-merge and tag CI remained
pending. Those v0.3.39 results do not accept v0.3.40. See [repository verification](../verification.md) for the full evidence and
[release notes](https://github.com/liveweird/grenadiers/releases) for current release status.
