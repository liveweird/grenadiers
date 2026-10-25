# Product backlog

Outstanding work only. Implemented behavior belongs in the [feature guides](docs/README.md),
[release records](docs/releases.md) and [changelog](CHANGELOG.md); completed items leave this file.
Historical requests and checkpoints remain in [backlog history](docs/history/backlog.md).

Each top-level item starts with one status: `[next]` current delivery, `[todo]` confirmed work,
`[new]` a proposal awaiting a product decision, `[parked]` deliberately deferred, or `[blocked]`
waiting on a stated dependency. Explicit user requests and accepted plans are confirmed work;
new assistant suggestions start as `[new]`. Existing authorization carries forward without
repeated approval for routine implementation choices. Entries are not delivery-date promises.

## Current delivery — single-player

- [next] **SOLO-CPU — deliver the approved staged hybrid.** A private human-versus-CPU duel uses
  the normal rules/input path, restricted observations, interval hazard forecasts, bounded goal
  planning and recorded bot inputs. Follow the [accepted design](docs/game/single-player-cpu.md)
  following the shipped QoL work; expose a playable slice early and support the all-features-on default before
  the public preview.
- [parked] **More seats and campaign mobs.** Revisit after the two-seat CPU mode is stable;
  neither is part of the initial solo duel.

## Engineering follow-ups

- [todo] **ENG-REVIEW — finish the remaining agreed review work.** Use the open dispositions in
  [quality review](docs/quality-review.md): typed configuration/transport cleanup, genuinely shared
  fixtures, measured renderer/registry behavior and the remaining protocol foundations. Keep
  privacy, exhaustive masks, coverage floors and local-first verification intact.
- [new] **Capability rotation and production replay export.** Establish recovery/consumer needs
  before changing authentication or adding durable storage. Current limitations are documented
  in [security](docs/security.md) and [architecture](docs/architecture.md).

## Playtesting and extensions

- [todo] **TURN — tune with human evidence.** Tolerance is already configurable. Evaluate early/
  late turns, speed levels, collision/fire contact and network delay; preserve fixed-point safety.
- [todo] **Expanded interactions.** Exercise item combinations, replacement/stacking, reconnect,
  generation and privacy. Keep the [implemented catalog](docs/game/expanded-items.md) authoritative.
- [new] **More room controls.** Consider fixed-round versus wins-to-finish semantics, round time,
  initial capacity/range and caps, flame duration, countdown/result pacing, soft-block density,
  item frequency/pools/weights and configurable effect durations. Existing numeric controls,
  item switches and creator/readiness/rematch policies are already implemented; see
  [settings](docs/game/settings.md). New cross-field combinations need explicit validation.
- [new] **Further art and animation.** Build on the adopted Napoleonic sheet and artist pipeline:
  directional/idle/death animation, environment detail, layered authoring sources, accessibility
  and measured effects budgets. Follow the [artist handoff](assets/art/README.md).
