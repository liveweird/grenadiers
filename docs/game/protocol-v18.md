# Gameplay v18: paired spike pits

Gameplay18 is an archived contract. Its sources are
[`api/gameplay-v18.schema.json`](../../api/gameplay-v18.schema.json), Rust `protocol::v18` and
generated `web/src/game/v18/schema.ts`. Live authentication rejects protocol18; use the paired [gameplay20 client](protocol-v20.md). Session and realtime
API versions are independent.

## Closed live profile

The live room uses rules10. CPU and mob-navigation identities are private replay provenance,
not gameplay18 wire fields. Generator3 covers 15×13; generator4
covers 13×11 and 11×9. Gameplay17/rules9/CPU25 remains an archived contract and is not widened.

Room configuration and settings extend the v17 environment with required
`spikePits { enabled, warningTicks, raisedTicks }`. Warning accepts 6–120 ticks and raised accepts
60–600 ticks; defaults are enabled, 36 and 180. Configuration, settings and ready updates retain
the existing atomic revision and creator policy.

## Authoritative pit state

Each admitted running state and snapshot has a required `spikePits` array. Disabled rooms publish
an empty array. Enabled rooms publish zero, two or four immutable, unique, row-major cells whose
reflection set is symmetric across both axes. Each entry is closed:

```json
{"cell":{"column":5,"row":3},"phase":"warning","remainingTicks":24}
```

`phase` is `idle`, `warning` or `raised`. Idle has zero remaining ticks; warning and raised carry
the authoritative positive countdown within the configured bound. Cells must be valid clear
floor outside spawn reservations and excluded overlays. The wire exposes no activator identity,
private route, seed or forecast.

The projection validates room tuning against the immutable construction profile, complete
phase clocks, layout symmetry, arena occupancy and recipient privacy before publication. CPU
observations are derived from that same checked recipient projection.

## Compatibility and bounds

V18 retains v17 admission, room lifecycle, solid-player collision, respawn-lair state, commands,
events and privacy rules. The closed outgoing frame limit remains 196,608 UTF-8 bytes and incoming
commands remain limited to 4,096 bytes. Upgrade or restart ends process-local matches; there is
no cross-version resume.

Replay format10 privately binds rules10, CPU26, mob navigation2, spike-pit layout1 and exact
spike timing. Format11 binds navigation3 with CPU26; format12 retains navigation3 and binds
live CPU27. These private versions do not widen retained readers or enables production replay recording. See
[spike pits](spike-pits.md), [lairs](lairs.md), [protocol consumers](protocol-consumers.md) and
the [historical v17 contract](protocol-v17.md).
