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

## Multiplayer and arenas

- [next] **MULTIPLAYER — three- and four-player matches.** Support human and/or CPU-controlled
  seats, including mixed lineups, with at least one human in every match. Give players three
  and four distinct uniform variants and contrasting palettes, preserving readable identity
  alongside the existing players. Extend lobby/seat configuration, spawning, round outcomes,
  readiness/rematch and disconnect handling. Adapt CPU observations/tactics, transport/replay
  contracts and the HUD to the selected player count; preserve two-player behavior.
  Follow the [multiplayer design](docs/game/multiplayer-design.md).
  - **MP2 — live activation:** connect the roster runtime and prepared lobby/HUD through
    the socket/owner adapter, per-seat CPU execution, replay format 4 and truthful diagnostics;
    verify complete human/mixed matches, reconnect and recipient privacy on 15×13 first.
- [todo] **BOARD-SIZE — variable board dimensions.** Allow the game board size to be configured;
  define supported sizes and bounds during design. Adapt terrain/item generation, fair spawn
  placement, CPU navigation, rendering and transport/replay limits. Verify supported board sizes
  with two-, three- and four-player lineups.

## Animation and arena environment

Implement after the current multiplayer and configurable-board work.

- [todo] **ANIMATION — richer movement and bomb pulsation.** Add player walking frames and
  bomb frames that communicate the approach to explosion. Extend the shared artist sheet and
  catalog while preserving anchors, readable timing and reduced-motion behavior.
- [todo] **STORM-CLOUD — roaming slowdown hazard.** A cloud moves slowly and randomly across
  the board, including blocked tiles. It never blocks movement and bombs cannot harm it.
  Player/cloud contact in either direction applies a three-second, 50% slowdown. Define contact
  refresh and interaction with existing slow effects before implementation; avoid per-tick
  compounding. Bound cloud count/speed and use replayable environment randomness.
- [todo] **LAIRS — indestructible mob spawners.** Add a wall-like, indestructible lair tile.
  Each lair spawns once every configurable N seconds, with a separate counter and the same
  configurable limit M. Mobs chase players reachable through unblocked paths, avoid bombs and
  never plant bombs. Before implementation settle whether M counts living or lifetime spawns,
  blocked spawn behavior, mob contact/damage/death, targeting and resource bounds.

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
- [new] **Further art.** Beyond the confirmed walking and bomb frames, consider idle/death
  animation, environment detail, layered authoring sources, accessibility and measured effects
  budgets. Follow the [artist handoff](assets/art/README.md).
- [new] **AUTO-PUNCH — consider automatic contact Punch.** The current idea is at most one
  activation per contact, reducing gameplay controls to movement and Bomb. The user has not approved this
  behavior; keep the existing manual Punch contract unless a product decision accepts it.
