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

## Animation and arena environment

- [next] **LAIRS — indestructible mob spawners.** Add a wall-like, indestructible lair tile.
  Each lair spawns once every configurable N seconds, with a separate counter and the same
  configurable limit M. Mobs chase players reachable through unblocked paths, avoid bombs and
  never plant bombs. The [lair foundation](docs/game/lairs.md) supplies simulation, checked wire16, CPU19,
  server/replay7 integration and sprites. Complete their integrated acceptance and release, then
  pair the live controls/rendering, run native browser checks and deploy activation.
  - Initial defaults: N = 10 seconds (configurable 5–60), M = 3 living mobs per lair (1–8).
    Two mirrored existing pillars bound the arena to 16 mobs. Full/blocked attempts wait one
    interval; deaths release capacity, and rounds reset counters.
  - Mobs move at 1.5 cells/second toward the nearest reachable living, uncloaked player.
    Contact uses Shield/Heart/grace; flames and Oil kill mobs. No drops or scoring.
    Stopwatch freezes existing and newly spawned mobs while lair timers continue.
  - Keep mobs separate from player seats and reuse geometry, damage and blast rules. Introduce
    rules8/wire16/CPU19/replay7 with bounded planning, private targets, recovery and replay checks.
    Append lair/mob art without changing existing sprite rectangles or reserved entries.

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
