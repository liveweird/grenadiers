# Grenadiers game specification

This design describes the classic Bomberman-style game loop and a proposed, implementable
Grenadiers ruleset. It covers player abilities, arena contents, bombs, damage, power-ups,
enemies, bots, modes, timing, and acceptance scenarios.

Status: **working baseline**, with numerical tuning still subject to playtesting. The first
playable target remains two human players. M1 implements the
[headless world and movement model](movement.md); M2 adds [bombs, flames, damage, and round
outcomes](combat.md), including explicit public projections/events. M3 completes
[gameplay contracts](protocol.md), anonymous sessions, authenticated [rooms](rooms.md),
authoritative scheduling, and the [playable browser duel](browser.md). The current preview
plays generated configurable-v4 matches (first to three by default) with upgrades, automatic round resets,
consensual rematch, and process-local reserved-seat reconnect. Bots, larger matches,
matchmaking, cross-process recovery, and durable stats follow.

Start with [the rules specification](rules.md) and the
[implementation and release plan](implementation-plan.md). Historical examples and optional extensions
are distinguished from the proposed initial rules. Exact historical fidelity requires choosing
a particular release and platform; “Bomberman” and “Dyna Blaster” do not identify one universal ruleset.

## Suggested reading

- [Modes and objectives](rules.md#modes-and-objectives) and [initial defaults](rules.md#initial-grenadiers-rules-and-defaults).
- [Tile behavior](rules.md#tile-and-object-behavior), [bomb placement](rules.md#bomb-placement-and-ownership), and [explosions](rules.md#explosions-and-chain-reactions).
- [Upgrades and effects](rules.md#upgrades-and-temporary-effects), [weapon extensions](rules.md#weapon-and-bomb-extension-catalog), and [mobs](rules.md#mobs-and-campaign-creatures).
- [Bot competitors](rules.md#bot-competitors), [round lifecycle](rules.md#round-and-match-life-cycle), and [disconnects](rules.md#disconnects-and-browser-behavior).
- [Event order](rules.md#authoritative-timing-and-event-order) and [acceptance scenarios](rules.md#acceptance-scenarios).

## Implementation sequence

The release plan defines the maintained milestone dependencies, work packages, RULE coverage,
and release gates. The near-term sequence is verified foundation, playable two-browser duel,
classic match progression, then reconnect resilience and playtest hardening. Mark a milestone complete only when its exit evidence exists in the maintained plan and
verification record.

[Baseline rule coverage](rule-coverage.md) maps all 42 acceptance rules to concrete automated assertions.

[Gameplay and presentation backlog](backlog.md) records playtest requests for forgiving turns,
shared room settings, detailed pixel art, equipment, timed effects and single-use weapons.
Forgiving turns, shared settings and the first pixel-art slice are available. New item activation
and remaining presentation work are planned extensions to the implemented Classic baseline.

[Runtime reference measurements](performance.md) documents bounded workloads, selected budgets and evidence limits.

[Browser hardening](browser-hardening.md) records real visibility recovery and its tested scope.

[Keyboard controls](controls.md) describes defaults, safe remapping and non-secret preference storage.

[Room settings](settings.md) covers shared starting speed, turn tolerance, match target and fuse.

[Pixel-art presentation](pixel-art.md) describes original artwork, animation and resource bounds.

[Expanded item implementation](expanded-items.md) records the optional inventory, effects,
weapons, visibility and incremental activation decisions; these items are not enabled yet.

[Ordered WebSocket delay](websocket-latency.md) defines the measured profile, causal observations and limits.

[Procedural sound](audio.md) explains gesture activation, mute, visibility recovery and event baselines.
