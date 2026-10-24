# Gameplay v11: materialized Oil Barrel state

The coordinated live contract is `api/gameplay-v11.schema.json`, Rust `core::protocol::v11`, and
generated `web/src/game/v11/schema.ts`. Server and browser must deploy together. Gameplay v10 and
earlier contracts remain archived and reject Oil settings and every private Oil phase or grace.

V11 retains Expanded-v5 rules, generated-v2 arenas, journal format 2, and the v10 actor, bomb,
event, command, Teleporter epoch, Kick, Punch and Bracelet shapes. It extends the canonical catalog
to 22 items by inserting `oilBarrel` at bit 20 immediately before Teleporter. All `4,194,304`
settings masks are valid.

Every Snapshot and state-bearing Admitted message requires a row-major, unique `oilBarrels` array
of at most eight `{cell,state}` entries. `state` is `unlit` or `burning`. Finite flames retain their
existing `{cell,expiresAt}` records; a burning Oil cell never appears there with a fabricated
deadline. Concealed and pending locations, typed grace, generation data and seeds are captured and
validated before recipient filtering but never serialized. Oil is forbidden in public pickups and
`pickupCollected` events.

Canonical validation checks the lifetime cap, phase exclusivity, floor/block/layer relationships,
bomb and finite-fire conflicts, actor capability or exact typed grace, and the shared 195-cell fire
bound. Both recipient frames must validate and encode before publication state, revision or pending
reports commit. Reload an older browser when the server reports `supportedVersion: 11`.
