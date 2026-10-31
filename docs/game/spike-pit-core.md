# Spike-pit core primitives

The core library provides checked timing, contact, placement and opt-in rules10 simulation for
the [spike-pit design](spike-pits.md). Existing profiles remain unchanged; the live game does
not select rules10 until its paired contract, AI and presentation are activated.
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

## Opt-in simulation

`RulesConfig::environment_v10` and its explicit mob-navigation variant accept the existing
settings, cloud/lair configuration and a `SpikePitConfig`. `State::spike_pits()` returns `None`
when disabled, or `Some(views)` when enabled, including an empty vector if no layout fits.
Each view contains only a cell, phase and remaining ticks. Result phases freeze these facts
even if headless callers advance the room clock; a new state begins with retracted pits.

Warnings rise before movement on their deadline, so departure during that update cannot undo
a standing hit. Contact includes ordinary and mob movement traces, each Bracelet segment and
Teleporter destinations without a connecting sweep. Raised spikes use existing player damage
protection. Mobs killed by a pit cannot harm players later on their trace, while genuine mine
contacts remain effective. Each dead mob schedules its lair's replacement delay once.

Warning or raised pits cannot receive a newly spawned mob. A newborn on a dormant pit activates
its warning at birth. Stopwatch does not pause trap timers. Rules10 stages the complete tick,
so an overflow or later fallible operation rolls back clocks, actors, mobs, allocator and RNG.

## Public observation clocks

`SpikePitClock::from_observation(phase, remaining_ticks, observed_at, settings)` reconstructs
future timing from public facts. It validates phase-specific remaining-time bounds and checked
deadlines, without inferring an activator or historical position. An idle observation fences
same-tick rearming; a raised observation supplies a future-clipped harmful interval starting
at `observed_at`. Forecast execution begins on the following physical tick.

## Verification

Focused tests cover phase/interval boundaries, repeated contacts, atomic overflow, monotonic
updates and forecasts, piecewise contact geometry, all board sizes, deterministic exclusions,
spawn protection and preference for four-cell layouts over nearer two-cell alternatives.

```sh
cargo test -p grenadiers-core --locked spike_pit
cargo test -p grenadiers-core --release --locked spike_pit
```
