# Product backlog

Outstanding work only. Implemented behavior belongs in the [feature guides](docs/README.md),
[release records](docs/releases.md) and [changelog](CHANGELOG.md); completed items leave this file.
Historical requests and checkpoints remain in [backlog history](docs/history/backlog.md).

Each top-level item starts with one status: `[next]` current delivery, `[todo]` confirmed work,
`[new]` a proposal awaiting a product decision, `[parked]` deliberately deferred, or `[blocked]`
waiting on a stated dependency. Explicit user requests and accepted plans are confirmed work;
new assistant suggestions start as `[new]`. Existing authorization carries forward without
repeated approval for routine implementation choices. Entries are not delivery-date promises.

## Arena presentation

- [next] **DEATH-ANIM — player and mob knockout sprites.** Append four artist-editable death
  poses for each uniform and the mob; play short, recipient-safe animations with reduced-motion
  support, bounded lifetime and no replay across admission or round resets.
- [todo] **ARENA3D — human visual feedback.** Evaluate camera/depth, sprite detail, readability,
  texture revisions and filter strength after the first playable version.
- [new] **Full 3D character models.** Consider after the billboard trial; retain the actor-view
  boundary while commissioning any required models, rigs and animation.

## Single-player

- [todo] **SOLO-CPU — validate and tune the staged hybrid.** Compare the
  [private Solo opponent](docs/game/single-player.md) on Easy and Standard with all items enabled.
  Follow the [accepted design](docs/game/single-player-cpu.md).
  - **CPU4 — human difficulty and playability validation:** collect feedback on pressure,
    stalls, fairness, unexpected item/death behavior and the difference between levels. Turn
    reproducible weaknesses into bounded tuning or further combat/equipment tactics. Preserve
    real queued-driver progress regressions, both reference matrices, bounded planning work
    and exact accepted-input replay; measure trajectory and cost alongside survival/outcomes.
- [parked] **Campaign stages and PvE progression.** Revisit after arena multiplayer and
  lair mobs; a campaign remains separate from the confirmed arena environment features.

## Engineering follow-ups

- [new] **Capability rotation and production replay export.** Establish recovery/consumer needs
  before changing authentication or adding durable storage. Current limitations are documented
  in [security](docs/security.md) and [architecture](docs/architecture.md).

## Playtesting and extensions

- [todo] **BASELINE — broaden browser/network validation.** Complete human playtesting and decide
  supported browser/network conditions. The 1.0 release records the current limits and
  upgrade/rollback guidance; remaining archived M7/M8 validation criteria stay open. CPU difficulty
  and campaign content remain separate tracks.
- [todo] **TURN — tune with human evidence.** Tolerance is already configurable. Evaluate early/
  late turns, speed levels, collision/fire contact and network delay; preserve fixed-point safety.
- [todo] **Expanded interactions.** Exercise item combinations, replacement/stacking, reconnect,
  generation and privacy. Keep the [implemented catalog](docs/game/expanded-items.md) authoritative.
- [new] **More room controls.** Consider fixed-round versus wins-to-finish semantics, round time,
  initial capacity/range and caps, flame duration, countdown/result pacing, soft-block density,
  item frequency/pools/weights and configurable effect durations. Existing numeric controls,
  item switches and creator/readiness/rematch policies are already implemented; see
  [settings](docs/game/settings.md). New cross-field combinations need explicit validation.
- [new] **Further art.** Beyond the confirmed walking and bomb frames, consider idle/death
  animation, environment detail, layered authoring sources, accessibility and measured effects
  budgets. Follow the [artist handoff](assets/art/README.md).
- [new] **AUTO-PUNCH — consider automatic contact Punch.** The current idea is at most one
  activation per contact, reducing gameplay controls to movement and Bomb. The user has not approved this
  behavior; keep the existing manual Punch contract unless a product decision accepts it.
