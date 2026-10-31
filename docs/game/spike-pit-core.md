# Spike-pit core primitives

The core library provides checked timing, contact and placement primitives for the
[spike-pit design](spike-pits.md). These APIs do not enable a trap in existing game profiles.
Work status remains in the [backlog](../../BACKLOG.md).

## Timing and settings

`SpikePitSettings` validates exact physical-tick durations: warning 6–120 ticks and raised
60–600 ticks. Defaults are 36 and 180 ticks, equivalent to 0.6 and 3 seconds at 60Hz.
`SpikePitConfig` defaults to `Disabled`; callers explicitly select `Enabled(settings)`.

`SpikePitClock::update(tick, contacted, settings)` arms an idle pit on contact. Both rise and
retraction deadlines are checked before mutation. The harmful interval is half-open:
`[activation + warning, activation + warning + raised)`. Contact cannot extend an active cycle.

Retraction makes the pit idle. Repeated contact checks during that same physical tick cannot
rearm it; contact in a later tick can begin a fresh warning. The clock records the last
successful update. Older updates and past forecast queries return typed errors, and a failed
overflow check cannot advance this horizon or otherwise mutate the clock.

`phase_at` and `remaining_ticks` support checked forecasts at or after that horizon.
`raised_interval` supplies the known harmful window without altering the authoritative clock.

## Contact and placement

Position, movement-segment and piecewise-trace contact helpers reuse the existing strict
positive-area footprint geometry. Edge-only tangency is safe. A piecewise turn is tested by
its actual segments rather than the bounding sweep between its first and last positions.

`spike_pit_layout_v1(arena, excluded)` searches bounded interior reflection orbits, considering
near-center seeds first with row-major tie-breaking. It globally prefers a clear four-cell
orbit, then a two-cell center-axis orbit. Results remain symmetric across both axes and sorted
row-major; at most four cells are returned. No eligible orbit means an empty result.

Every chosen cell must already be unblocked floor, outside the supplied overlay exclusions
and the protected escape cells of all four potential corner spawns. The helper neither carves
blocks nor samples randomness, so it cannot change base generation or RNG progress.

## Verification

Focused tests cover phase/interval boundaries, repeated contacts, atomic overflow, monotonic
updates and forecasts, piecewise contact geometry, all board sizes, deterministic exclusions,
spawn protection and preference for four-cell layouts over nearer two-cell alternatives.

```sh
cargo test -p grenadiers-core --locked spike_pit
cargo test -p grenadiers-core --release --locked spike_pit
```
