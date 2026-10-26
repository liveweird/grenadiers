# Grenadiers game specification

This guide describes the classic Bomberman-style game loop and the Grenadiers ruleset. It covers
player abilities, arena contents, bombs, damage, power-ups, enemies, bots, modes, timing and
acceptance scenarios. Numerical tuning remains subject to playtesting.

The implemented human-duel and [solo game](single-player.md) use the [headless world and movement model](movement.md),
[combat](combat.md), versioned [gameplay contracts](protocol.md), authenticated [rooms](rooms.md),
authoritative scheduling and the [browser duel](browser.md). Generated Expanded-v5 matches are
first-to-three by default, with upgrades, automatic round resets, consensual rematch and
process-local reserved-seat reconnect. The [product backlog](../../BACKLOG.md) is the sole source
for planned work and delivery status.

Start with [the rules specification](rules.md) and the
[release checklist and future-track boundaries](implementation-plan.md). Historical examples and
optional extensions are distinguished from the initial rules. Exact historical fidelity requires
choosing a particular release and platform; “Bomberman” and “Dyna Blaster” do not identify one
universal ruleset.

## Suggested reading

- [Modes and objectives](rules.md#modes-and-objectives) and [initial defaults](rules.md#initial-grenadiers-rules-and-defaults).
- [Tile behavior](rules.md#tile-and-object-behavior), [bomb placement](rules.md#bomb-placement-and-ownership), and [explosions](rules.md#explosions-and-chain-reactions).
- [Upgrades and effects](rules.md#upgrades-and-temporary-effects), [weapon extensions](rules.md#weapon-and-bomb-extension-catalog), and [mobs](rules.md#mobs-and-campaign-creatures).
- [Bot competitors](rules.md#bot-competitors), [round lifecycle](rules.md#round-and-match-life-cycle), and [disconnects](rules.md#disconnects-and-browser-behavior).
- [Event order](rules.md#authoritative-timing-and-event-order) and [acceptance scenarios](rules.md#acceptance-scenarios).

<a id="implementation-sequence"></a>

## Implementation and release references

The [live implementation guide](implementation-plan.md) owns the reusable release checklist and
future-track design boundaries. The original M0–M8 sequence, work packages and baseline
acceptance traceability are preserved in the
[implementation-plan archive](../history/implementation-plan.md). Use the
[product backlog](../../BACKLOG.md) for current priorities and progress.

[Baseline rule coverage](rule-coverage.md) maps all 42 acceptance rules to concrete automated assertions.

[Single-player versus CPU design](single-player-cpu.md) records the accepted staged hybrid and
its static acceptance boundary; the backlog owns its prerequisite and staged work.

The [product backlog](../../BACKLOG.md) contains open work; [legacy backlog links](backlog.md)
remain valid. Feature guides document implemented movement, shared settings, presentation and
the expanded item catalog. Use [settings](settings.md) and [expanded items](expanded-items.md)
for current defaults and behavior, rather than treating design proposals as delivery evidence.

[Runtime reference measurements](performance.md) documents bounded workloads, selected budgets and evidence limits.

[Browser hardening](browser-hardening.md) records real visibility recovery and its tested scope.

[Keyboard controls](controls.md) describes defaults, safe remapping and non-secret preference storage.

[Room settings](settings.md) covers shared starting speed, turn tolerance, match target and fuse.

[Gameplay v11 compatibility](protocol-v11.md) defines the paired live wire contract, including the
authoritative teleport position epoch. [Gameplay v10](../history/protocols/protocol-v10.md) and
[v9](../history/protocols/protocol-v9.md) are archived; the
[protocol history index](../history/protocols/README.md) links every standalone historical guide.

[Pixel-art presentation](pixel-art.md) describes original artwork, animation and resource bounds.

[Expanded item implementation](expanded-items.md) records the inventory, effects, weapons,
visibility and historical incremental activation decisions; new rooms now enable the complete
available catalog and creators may configure any subset.

[Ordered WebSocket delay](websocket-latency.md) defines the measured profile, causal observations and limits.

[Procedural sound](audio.md) explains gesture activation, mute, visibility recovery and event baselines.

Protection mechanics: [Heart](heart.md) and [Shield](shield.md).

- [Golden Flame](golden-flame.md): ten-second maximum-range pickup and captured bomb behavior.
- [Reverse Controls](reverse-controls.md): ten-second movement inversion with exact boundaries.
- [Diarrhea](diarrhea.md): ten-second automatic ordinary-bomb placement curse.
- [Slow foot](slow-foot.md): up to six independent ten-second movement-slowing stacks.
- [Piercing Bomb](piercing-bomb.md): one stored charge for a manual bomb that crosses soft blocks.
- [Power Bomb](power-bomb.md): one stored charge for a Manhattan blast that ignores obstruction.
- [Remote Bomb](remote-bomb.md): one untimed bomb triggered by the owner's next bomb action.
- [Landmine](landmine.md): one passable contact mine with per-actor exit grace.
- [Invisibility Cloak](invisibility-cloak.md): ten seconds of recipient-private actor details.
- [Stopwatch](stopwatch.md): three seconds of opponent action freezing and Timed-bomb pausing.
- [Invisible Bombs](invisible-bombs.md): ten seconds without live bomb records or bomb cues.
- [Teleporter](teleporter.md): an immediate, atomic safe-position swap or consumed unsafe attempt.
- [Kick](kick.md): permanent bomb-kicking equipment with private motion history.
- [Punch](punch.md): permanent three-cell wrapped bomb movement with private cooldown history.
- [Bracelet](bracelet.md): passive contact shove equipment with private cooldown history.
- [Oil Barrel](oil-barrel.md): bounded arena objects that reveal solid and burn until round end.
- [Wall Pass](wall-pass.md): permanent soft-block passage with private replacement exit grace.
