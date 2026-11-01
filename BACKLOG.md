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

- [todo] **ARENA3D — human visual feedback.** Evaluate camera/depth, sprite detail, readability,
  texture revisions and filter strength after the first playable version.
- [new] **Full 3D character models.** Consider after the billboard trial; retain the actor-view
  boundary while commissioning any required models, rigs and animation.

## Single-player

- [next] **CPU-RELIABILITY — repair interrupted escapes and validate liveness.** Address
  recovery after simultaneous rival bombs invalidate committed routes; the
  [controlled investigation](docs/history/cpu-short-fuse-investigation-2026-11-01.md) defines the
  observation-gated continuation boundary and required owner witnesses. The
  [timing experiment](docs/history/cpu-recovery-timing-2026-11-01.md) establishes
  scheduling facts; certified continuation and actual queued-owner Short Fuse survival now cover
  the controlled interrupted-escape case. Validate sustained farming
  from the original captured conditions: alternate-route proof and refused-origin guidance have
  separate witnesses, without a continuous restored-match trajectory. Retain complete escape/contact
  proofs and the existing work allowance. The [fresh late-match investigation](docs/history/cpu-late-passivity-2026-11-01.md)
  identified exposed Teleporter as a global planting support limit. The
  [typed one-update handoff](docs/game/single-player-cpu.md#intentional-teleporter-collection-boundary)
  now covers Approach/Collect, all possible swap partners, mandatory physical Stop and a
  subsequent productive-action witness. A fresh CPU47 round records two real Teleporter swaps
  followed by renewed bombing, then a third persistent Teleporter. Characterize sample 88/tick 5461
  and its preceding queue/intent transition through a continuous owner witness: seat Four is
  observed at an unchanged sampled pose with unavailable diagnostics, while seat Two keeps
  cycling farming approaches.
  Preserve real Stop provenance, worker/refusal/admission evidence and full useful-action
  execution; static reachability alone is insufficient. Reproduce remaining long-match
  stalls, passive pickup behavior and waiting beside lethal bombs under current rules/policy.
  Preserve exact public observations and authoritative before/after witnesses. Use action-refusal
  and queue diagnostics plus productive-action witnesses; elapsed time or a free bomb slot alone
  does not establish a missed safe action. Preserve safety certificates, bounded scheduling,
  replay/privacy and sustained per-seat progress regressions.

- [todo] **HARD-CPU — deepen fair tactics.** Improve productive farming,
  pickup contention and opponent-exit guidance toward useful bomb combinations. Use permitted
  observations, bounded planning and unchanged physics. Require authoritative before/after
  witnesses and independent review; preserve archived policies and complete seeded episodes.

- [todo] **SOLO-CPU — validate and tune the staged hybrid.** Compare the
  [private Solo opponent](docs/game/single-player.md) on Easy, Standard and Hard with all items enabled.
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
- [new] **Further art.** Beyond the walking, bomb and death frames, consider idle
  animation, environment detail, layered authoring sources, accessibility and measured effects
  budgets. Follow the [artist handoff](assets/art/README.md).
- [new] **AUTO-PUNCH — consider automatic contact Punch.** The current idea is at most one
  activation per contact, reducing gameplay controls to movement and Bomb. The user has not approved this
  behavior; keep the existing manual Punch contract unless a product decision accepts it.
