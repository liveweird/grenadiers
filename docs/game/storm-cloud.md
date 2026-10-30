# Storm-cloud simulation

Rules7 introduced seeded, nonblocking weather. Live rules9/gameplay17 includes the same cloud
alongside lairs. Older protocols remain
archived and cannot publish it.

New rooms enable the cloud by default. Forming-lobby All on/Restore defaults enable it;
All off/Classic disable it. The cloud has its own setting and is not an item pickup. The arena
renders the authoritative cloud above terrain, and the local player's effect bar shows the
remaining slowdown time. Reduced motion snaps between published positions.
See the [English desktop](../screenshots/storm-cloud-desktop.png) and
[Polish reduced-motion mobile](../screenshots/storm-cloud-mobile-pl-reduced.png) presentation.

## Construction and movement

Choose `StormCloudConfig::Disabled` or `Single` explicitly through
`RulesConfig::environment_v7(settings, clouds)`. The configuration has no implicit default.
An enabled cloud requires `State::new_at_tick_with_assignment_and_seed`; a missing seed returns
`StateError::MissingSimulationRandomness`. All three board presets and two to four actors are
supported.

The single cloud starts at the board's center cell. It moves eight coordinate units per
physical tick, taking 128 ticks to cross one cell. Its center stays between the interior
cell centers. At each completed leg it chooses a cardinal direction, reflecting an outward
choice at the boundary. Walls, blocks, bombs, flames, Oil and actors do not obstruct or harm it.
Stopwatch does not freeze cloud motion.

## Contact and slowdown

Cloud and actor footprints use strict positive-area overlap; merely touching edges is safe.
Contact includes the cloud's sweep, each actual ordinary/forced movement segment and a
Teleporter destination. Corner turns retain their separate segments; teleportation has no
connecting sweep.

Contact at tick `T` refreshes one deadline to `T + 180`, affecting subsequent movement. The
deadline expires when the current tick reaches it. Repeated contact refreshes duration without
adding stacks. Storm slow contributes one independent half-speed factor alongside up to six
Slow Foot stacks, even when the Slow Foot item is disabled. Movement residuals remain intact.
Shield, Heart and Cloak do not prevent the slowdown. Death and terminal results clear deadlines;
terminal steps retain the final cloud pose without advancing motion or randomness.

## Determinism and boundaries

Cloud randomness uses ChaCha20 stream 1 from the existing private round seed. Teleporter stream
0 is unchanged. One word selects each leg; contact and collisions consume no extra words.
Seed/state debug output is redacted, and cloning/rollback includes RNG progress.

Expiry overflow is rejected before mutation or RNG advancement. A later pickup failure restores
the cloud together with the rest of the simulation. Existing `PublicState`, `ActorView`, CPU
observation and replay contracts keep their layouts. Headless callers use `State::storm_cloud`,
`Actor::storm_slow_until` and `Actor::storm_slow_active`; shared constants and integer geometry
provide the same physics to consumers.

Run `cargo test --locked -p grenadiers-core storm` and the same command with `--release` for
focused coverage. Full-step contact, independent multi-leg RNG goldens, overflow rollback and
legacy compatibility also participate in the normal Rust gates.
