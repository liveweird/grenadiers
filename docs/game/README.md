# Grenadiers game specification

This design describes the classic Bomberman-style game loop and a proposed, implementable
Grenadiers ruleset. It covers player abilities, arena contents, bombs, damage, power-ups,
enemies, bots, modes, timing, and acceptance scenarios.

Status: **working baseline**, with numerical tuning still subject to playtesting. The first
playable target remains two human players. M1 implements the
[headless world and movement model](movement.md); M2 adds [bombs, flames, damage, and round
outcomes](combat.md), including explicit public projections/events. M2 is complete;
M3 implements [gameplay contracts](protocol.md), anonymous HTTP sessions, and the pure
[room lifecycle](rooms.md); authenticated sockets and playable browser integration follow.

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
