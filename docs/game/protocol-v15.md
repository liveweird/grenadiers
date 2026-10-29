# Gameplay v15 compatibility contract

`api/gameplay-v15.schema.json`, `core::protocol::v15` and generated
`web/src/game/v15/schema.ts` define the archived environment profile. The live WebSocket endpoint
uses gameplay16; protocol 15 and older versions are rejected before credential claim.

## Configuration and state

V15 uses rules7 for every supported board and controller count, including cloud-disabled
matches. Configuration and complete lobby settings each require
`environment: { stormCloud: boolean }`; their values must agree. Two to four controllers,
at least one human and the existing three board presets remain supported. All new rooms use
generator 3 on 15×13 and generator 4 on 13×11/11×9.

`room.settingsEditor` is an explicit editing grant. A null grant makes the controls read-only;
clients must not infer editability from lobby phase or readiness. The creator can edit only
before the first start. That lock survives rematches; changing rules afterward requires a new
room. Base rules and environment changes share one atomic revision and readiness reset.

Before the first start, explicit leave and reconnect-grace expiry release human slots as
unoccupied/Vacant. A new admission can fill the same configured seat; retired capabilities stay
invalid. Creator authority is never transferred. After first start, departures remain permanent
through rematches and their slots cannot be refilled.

Snapshots add nullable `state.stormCloud`. An enabled cloud reports fixed-point `position`,
cardinal `direction` and `legRemainingTicks` from 0 through 128. Position and leg progress must
agree with the board bounds and eight-unit movement rate. Disabled environments require null.
The representation exposes the current straight leg, never the seed or future random turns.

Visible actors add `stormSlowRemainingTicks` from 0 through 180. Hidden actors retain only
their existing ID, seat and alive fields. Owners always receive their own visible actor.
Disabled environments and dead actors have zero storm time. No storm-contact event reveals
cloaked opponents. Item effects and the dedicated environment timer remain separate.

## Checked publication and consumers

Capture validates actual rules7 state. Binding checks configuration, settings, lifecycle,
generation, physical epoch and recipient facts before preparing admission or snapshot frames.
It reuses the physical projection boundary without relabelling rules7 as an older profile.
Rules5/6 publication through archived protocols remains closed to rules7.

The CPU adapter consumes this same checked recipient publication. Policy18 handles either
Disabled or Single, models the published leg and timer, and bounds uncertainty after future
turns. CPU inputs remain ordinary validated actions; private simulation state is unavailable.
See [CPU boundaries](multiplayer-cpu.md) and [replay profiles](multiplayer-replay.md).

Run `cargo test --locked -p grenadiers-core protocol::v15` and
`npm --prefix web test -- --run src/game/v15` for focused conformance tests. The shared corpus
checks actual server projections in Rust and in the browser. Generated files must be rebuilt
through the existing gameplay contract scripts, never edited by hand.
