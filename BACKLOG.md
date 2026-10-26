# Product backlog

Outstanding work only. Implemented behavior belongs in the [feature guides](docs/README.md),
[release records](docs/releases.md) and [changelog](CHANGELOG.md); completed items leave this file.
Historical requests and checkpoints remain in [backlog history](docs/history/backlog.md).

Each top-level item starts with one status: `[next]` current delivery, `[todo]` confirmed work,
`[new]` a proposal awaiting a product decision, `[parked]` deliberately deferred, or `[blocked]`
waiting on a stated dependency. Explicit user requests and accepted plans are confirmed work;
new assistant suggestions start as `[new]`. Existing authorization carries forward without
repeated approval for routine implementation choices. Entries are not delivery-date promises.

## Single-player

- [next] **SOLO-CPU — deliver the approved staged hybrid.** A private human-versus-CPU duel uses
  the normal rules/input path, restricted observations, interval hazard forecasts, bounded goal
  planning and recorded bot inputs. Follow the [accepted design](docs/game/single-player-cpu.md)
  and expose a playable slice early and support the all-features-on default before the
  public preview.
  - **CPU0 — contracts and seams:** participant/mode metadata, neutral observation and intent,
    work budget, replay/lifecycle policy and deterministic fixture harness.
  - **CPU1 — internal playable slice:** solo room, legal bot input, real result and
    rematch/reconnect; a stand-still controller is only a harness step.
  - **CPU2 — survival:** interval forecasts, timed routing, safe planting and bounded memory,
    checked against fully known rule fixtures.
  - **CPU3 — public preview:** scored goals, all-features-on compatibility, localized accessible
    presentation and real-browser acceptance of the complete solo flow.
  - **CPU4 — stronger play:** difficulty tuning, tactical look-ahead and measured performance
    from deterministic tournaments and human feedback.
- [parked] **More seats and campaign mobs.** Revisit after the two-seat CPU mode is stable;
  neither is part of the initial solo duel.

## Engineering follow-ups

- [new] **Capability rotation and production replay export.** Establish recovery/consumer needs
  before changing authentication or adding durable storage. Current limitations are documented
  in [security](docs/security.md) and [architecture](docs/architecture.md).

## Playtesting and extensions

- [todo] **BASELINE — close the two-player release scope.** Complete human playtesting, decide
  supported browser/network conditions and publish the stable baseline with explicit limits and
  upgrade/rollback guidance. Archived M7/M8 criteria define acceptance, not completion; CPU and
  campaign content are separate tracks.
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
- [new] **AUTO-PUNCH — consider automatic contact Punch.** The current idea is at most one
  activation per contact, reducing gameplay controls to movement and Bomb. The user has not approved this
  behavior; keep the existing manual Punch contract unless a product decision accepts it.
