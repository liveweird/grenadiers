# Gameplay and presentation backlog

TURN v0.3.16, CONFIG v0.3.17 and the first ART slice v0.3.18 are shipped. Heart
[v0.3.25](https://github.com/liveweird/grenadiers/releases/tag/v0.3.25), Shield
[v0.3.26](https://github.com/liveweird/grenadiers/releases/tag/v0.3.26), Golden Flame
[v0.3.27](https://github.com/liveweird/grenadiers/releases/tag/v0.3.27) and Short Fuse
[v0.3.28](https://github.com/liveweird/grenadiers/releases/tag/v0.3.28), and Short Flame
[v0.3.29](https://github.com/liveweird/grenadiers/releases/tag/v0.3.29) are published.
The v0.3.30 Long Fuse increment passes local checks and fresh 64/64 Compose/Kubernetes browser
cases against the corrected stable-cursor source, with wrapper exit zero and no retries or
carried results, and is published. The v0.3.31 Reverse Controls increment also passes fresh
66/66 native acceptance and all required CI; both are published source prereleases. The v0.3.32
Diarrhea increment passes local checks and fresh corrected-readiness acceptance: 45 Compose plus
45 Kubernetes cases, with all eight optional item scenarios and zero retries. Independent final
native review, candidate CI and post-merge CI pass; [v0.3.32](https://github.com/liveweird/grenadiers/releases/tag/v0.3.32)
is published. The original shaped-latency CI counterexample and source-specific 68/68 native
result remain historical evidence. The
v0.3.33 Slow-foot candidate passes local checks and independent backend/frontend reviews; its
current scope passes 22 latency cases per target, 47 static scenarios and fresh 46 Compose plus
46 Kubernetes cases (**92 total**) with zero retries. Independent native evidence review is clear;
final-head CI and publication remain release gates. Its final raw source is integrated into the
v0.3.34 Piercing Bomb candidate, which retains the feature and version but still requires its
post-integration checks, fresh native acceptance and final-head CI. Verification retains
source-specific evidence and earlier failed or interrupted attempts. The other 12 requested
pickups remain outside this increment. Richer transitions
and artwork remain follow-up work. See [pixel-art presentation](pixel-art.md),
[expanded item implementation](expanded-items.md), and
[release notes](https://github.com/liveweird/grenadiers/releases).
Updated 2026-10-20 with the artist-handoff follow-up to the playtest feedback. Release acceptance is recorded in
[verification](../verification.md).
The [implementation plan](implementation-plan.md) remains the milestone/release reference;
the [rules](rules.md) describe the current baseline and historical extension catalog.
This document records the requested Grenadiers behavior and decisions required before coding.
Proposed implementation policies below remain subject to design review and playtesting.

## Priority and delivery sequence

| Order | Increment | Completion evidence |
| --- | --- | --- |
| 1 | TURN: forgiving turning | Wider, tunable assistance; actual keyboard playtest and collision/fire regressions. |
| 2 | CONFIG: room gameplay settings | Validated shared settings, immutable during a match, visible to both players. |
| 3 | ART: detailed modern retro pixel art | Complete readable arena asset set, animation, accessible presentation, measured rendering. |
| 4 | EFFECTS: equipment and timed-effect foundations | Separate slots, bounded timers/stacks, deterministic interactions, private projections. |
| 5 | Individual equipment, buffs, curses and weapons | One independently reviewed behavior per bounded feature increment. |
| 6 | ART-ATLAS: artist handoff and sprite-sheet pipeline | Reviewed art sample, complete labelled asset inventory, reproducible PNG/JSON exports and validated PixiJS integration. |

TURN belongs in M7 playability work. CONFIG and ART can progress separately once their
interfaces are agreed. The expanded item system is an optional ruleset expansion, not a new
requirement for the classic two-player v0.4.0/v1.0.0 baseline. Choose expansion release numbers
when scheduling; continue focused PRs, version bumps and the existing release gates.

## TURN: forgiving movement and corner alignment

Reported issue: players get stuck when requesting a perpendicular turn outside the inclusive
128/1,024-cell-unit window (one eighth of a tile). v0.3.16 widens this to 256 units
(one quarter) in classic-v3 while retaining historical profiles. Regression and native keyboard
acceptance cover the natural wall-stop offset of 192 units. Human tuning remains open;
expose tolerance in CONFIG and adjust further against playtest evidence.

- Assist alignment toward the requested corridor without requiring pixel-perfect positioning.
- Preserve the fixed-point movement budget, swept collision and swept flame contact. A larger
  window must not allow wall clipping, tunneling, extra diagonal speed or hazard avoidance.
- Test early/late turns, both axes/directions, every speed level, narrow openings, bombs,
  reversal, held inputs and network delay. Evaluate whether tolerance alone resolves sticking
  before adding a separately specified buffered-turn mechanic.

## CONFIG: shared gameplay configuration panel

Initial implementation: starting speed, turn tolerance (1/16–¼ tile), wins to finish and
bomb fuse. See [settings](settings.md). The remaining controls below are follow-up work.

Add a room/lobby panel for the following playability settings, with units, defaults, validated
ranges, contextual explanations and a restore-defaults action. Existing keyboard/audio settings
remain personal preferences; gameplay settings are authoritative shared room rules.

| Setting | Purpose / semantics |
| --- | --- |
| Player speed | Starting cells/second and the relationship to speed upgrades/caps. |
| Corner alignment tolerance | Maximum permitted centerline correction, displayed as a tile fraction. |
| Number of rounds | Expose match length; explicitly choose wins-to-finish versus a fixed round count. Current baseline is first to three wins, with draws awarding no point. |
| Bomb fuse | Seconds from successful planting to ordinary detonation. |
| Round time limit | Running duration before timeout resolution. |
| Starting bomb capacity and blast range | Initial loadout, with separately validated upgrade caps. |
| Flame duration | How long exploded cells remain hazardous. |
| Countdown and inter-round hold | Pacing between readiness, results and subsequent rounds. |
| Soft-block density and item frequency | Arena openness and upgrade availability; retain safe-opening validation. |
| Item pools and weights | Enable equipment, buffs, curses and weapons individually; provide a classic preset. |
| Effect durations | Timed buff/curse lifetime and documented refresh/stack policy. |

Proposed policy: creator edits only in the unready lobby; changes clear readiness and both
players see the same revision before starting. Lock settings for the match, including automatic
rounds; specify rematch edit ownership and creator departure behavior. Validate cross-field
combinations, not just individual bounds. Never expose active map seeds or concealed contents.
Propagate rules identity/settings through contracts, replay metadata and reconnect baselines.

## ART: modern retro pixel-art presentation

The first code-generated pixel-art slice shipped in v0.3.18. Extend it with detailed, coherent
artwork using the existing PixiJS renderer, keeping collision and simulation independent of art.
The artist handoff and sprite-sheet pipeline are queued separately under ART-ATLAS below.

- Establish a palette, pixel grid, sprite scale, lighting and silhouette guide before production.
  Use crisp nearest-neighbor sampling and deliberate scaling at desktop and narrow viewports.
- Create original floor/wall/soft-block variants, dressed arena borders, animated player sprites,
  directional movement, idle/death states, bomb fuses, explosions, destruction and pickup effects.
- Give every new item/effect a recognizable icon, animation and HUD indicator. Add environmental
  detail without obscuring walkable cells, bomb positions, flame reach or player identity.
- Retain distinctions beyond color, readable EN/PL controls, reduced-motion/effects options and
  clear focus/connection states. Bound particles, texture memory and animation work.
- Deliver an art-direction preview first, then small coherent asset/animation slices with
  screenshots, provenance/licenses and measured performance. No artwork is generated by this backlog change.

## ART-ATLAS: artist handoff and sprite-sheet pipeline

Queued after the current expanded-item gameplay batch. The existing code-generated PixiJS
art remains the baseline; this follow-up does not interrupt those feature releases.

- Export the current drawings into a labelled visual inventory, including terrain variants,
  directional player frames, bombs, connected flames, pickups and status overlays. Reserve
  named slots for planned items and animations, distinguishing placeholders from finished art.
- Give the artist screenshots and a brief covering palette, lighting, silhouettes and the
  existing 48×48 logical tile grid. Review a small arena mockup and representative animated
  assets before redrawing the full catalogue.
- Provide transparent authoring sheets with stable asset IDs and separate guide/label layers.
  Maintain one generated overview of everything; allow separate editable terrain, character
  and item/effect sheets as animations grow. Retain layered sources, such as Aseprite files.
- Automate reproducible PNG-atlas and JSON-manifest exports containing named frame rectangles,
  anchors and animation timing. PixiJS should use atlas regions directly; individual sliced
  PNG files are optional exports. Document one command for exporting and validating assets.
- Validate required frames, dimensions, transparency, bounds, IDs, padding and texture budgets.
  Keep nearest-neighbor sampling, consistent character foot anchors and seamless flame pieces.
- Integrate artwork in bounded releases with independent visual review, desktop and narrow
  screenshots, reduced-motion checks and measured rendering performance. Preserve collision
  geometry, authoritative visibility, hazard readability, player distinctions and EN/PL UI.

This is a backlog specification; the authoring templates and export pipeline are not implemented.

## EFFECTS: inventory and status foundations

The [expanded item implementation plan](expanded-items.md) records the staged architecture,
bounded state, interactions and activation acceptance. The v0.3.19 foundation does not expose
item pickups or controls. Classic remains the default; later increments will extend contracts
and transport before activation.

Keep the existing capacity/range/speed upgrades. Add these independent categories:

- **Equipment:** one persistent slot. Picking another equipment item replaces the previous one.
  “Permanent” means no timer while held; round-reset/persistence policy must be explicit.
  Heart is consumed when it protects the player.
- **Buffs:** multiple different timed positive effects may coexist.
- **Curses:** multiple different timed negative effects may coexist; Slow-foot explicitly stacks.
- **Special weapons:** one single-use weapon slot, separate from equipment. Picking another
  weapon replaces the unused one; consume only on successful placement/use.

Before activation, define tick-based duration, re-pick refresh/stack behavior, caps, expiry order,
death/reset/leave cleanup and HUD feedback. Proposed default is round-local inventory/effects;
record any exception in the enabled ruleset. Store base upgrades separately from overlays so
expiry restores the correct values. Capture fuse/range/weapon properties on successful planting
unless a mechanic explicitly operates on existing bombs (Stopwatch).

Create an interaction matrix covering protection versus damage, opposing modifiers, frozen and
moving bombs, control inversion versus input expiry, concealed entities and damage/collection
order. Bounds must prevent unbounded stacks, entities, timers or per-tick work. Any new action
needs remappable controls, input sequencing, replay and connection-generation fencing.

## Equipment: one persistent slot

| ID / pickup | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| EQ-KICK / boot | Walking into a planted bomb slides it across the floor until a wall, another bomb or a player stops it. | Sliding speed, soft-block collision, contact priorities, fuse/chain behavior and bomb ownership; no tunneling. |
| EQ-PUNCH / boxing glove | Punch an adjacent bomb three tiles forward over obstacles, wrapping at arena edges. | Action/direction, airborne collision and fuse behavior, landing on blocked cells, repeated punches; wrap logical arena cells rather than screen pixels, defining treatment of border walls. |
| EQ-SHOVE / bracelet | Shove rival players when touching them. | Contact detection despite normally non-blocking players, displacement, cooldown, simultaneous shoves and collision/fire along the pushed path. |
| EQ-WALL / wall pass | Phase through destructible soft blocks. | Hard walls/borders stay solid; interaction with bombs and hazards; safe resolution if replaced/removed while inside a block. |
| EQ-HEART / heart | Absorb one lethal explosion or enemy collision, then vanish. | Simultaneous/overlapping hits, remaining in flame, protection precedence with Shield and post-hit grace. Enemy collision applies when an enemy damage system exists. |

## Timed buffs: multiple active effects allowed

| ID / pickup | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| BUFF-SHIELD / shield | Temporary immunity to bomb blasts, monsters and other configured damage. | Damage-class coverage, duration, expiry while in danger and interaction with Heart. |
| BUFF-CLOCK / stopwatch | Temporarily freeze bomb fuses, monsters and enemy players. | Affected owners/entities, existing versus newly spawned bombs, multiple clocks, thaw/input handling; freeze selected gameplay entities without stopping server ticks, round deadlines or reconnect grace. |
| BUFF-CLOAK / invisibility cloak | Opponents cannot see the owner. | Owner feedback, visibility duration, shadows/events/HUD leakage and reconnect. Use recipient-specific projections rather than hiding sprites while sending secret positions. |
| BUFF-GOLD / golden flame | New bombs use maximum blast radius while active. | Maximum derives from the selected ruleset; define precedence with Short-flame and expiry. |
| BUFF-FUSE / short fuse | Decrease new bombs' detonation delay by 50%. | Integer-tick rounding/minimum, repeated pickups and composition with Long fuse. |

## Curses and hostile pickups

Multiple timed curses may coexist. Oil barrel creates a persistent hazard; Teleporter is an
instant action. Keep both in this requested pickup family without forcing them into a timer model.

| ID / pickup | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| CURSE-DROP / diarrhea | Automatically drop bombs whenever capacity is available, up to the player's bomb limit. | Bounded attempt cadence, legal placement, capacity release, freeze, special-weapon consumption and remote-trigger behavior; no unbounded same-tick loop. |
| CURSE-SLOW / slow-foot | **Prepared in v0.3.33.** Each pickup halves current effective speed again. | Independent 600-tick lifetimes, exact expiry restoration, six-stack cap, exact fixed-point arithmetic and zero-budget ticks have implementation coverage; v0.3.33 passes fresh 92/92 native cases, with final release gates tracked above. |
| CURSE-RANGE / short-flame | Reduce new bombs to one adjacent tile in each blast direction. | Center cell still explodes; precedence with Golden flame, upgrades and special footprints. |
| CURSE-FUSE / long fuse | Increase new bombs' detonation delay by 100% (double it). | Rounding, stacking/re-pick policy and composition with Short fuse, remote bombs and mines. |
| CURSE-REVERSE / reverse controls | Up moves down, left moves right, and vice versa. | Apply to logical movement after personal remapping; define held-key transition/expiry without stuck movement or stale replay. |
| CURSE-BLIND / invisible bombs | The afflicted player cannot see any bombs, including their own. | Recipient-specific bomb/event/effect projection and reconnect; explicitly define retained collision clues, fuse sounds and blast warnings. |
| CURSE-OIL / oil barrel | Before detonation it blocks passage like a soft wall; after detonation it burns indefinitely. | Define how acquisition creates/places the barrel, detonation/chain triggers, blast footprint and destruction; bound creation and specify cleanup at round end without expiry during play. |
| CURSE-SWAP / teleporter | Immediately swap with a random enemy player. | Deterministic recorded randomness; eligible living targets, safe/occupied/soft-block destinations, hazard contact, overlap grace and no-target behavior. Two-player mode has only one eligible rival. |

## Special weapons: one single-use slot

| ID / weapon | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| WPN-REMOTE / remote bomb | Next bomb is planted normally; the next bomb-button press detonates it instead of using a fuse. | Armed-remote state independent of consumed inventory; button priority, owner death/disconnect, chain ignition, freezing, capacity and inability to plant; no implicit safety fuse added without a decision. |
| WPN-PIERCE / piercing bomb | **Prepared in v0.3.34.** The next successful manual bomb's blast continues through destructible blocks. | One-charge replacement, retained failures, ordinary automatic bombs, captured blast identity, immutable same-tick chain planning and hard-wall stopping are covered locally with independent review clear; native acceptance and final CI remain pending. |
| WPN-POWER / power bomb | Next bomb damages a Manhattan-distance area of the player's blast radius, ignoring line of sight. | Exact cell mask, propagation across hard walls versus destruction of those walls, borders, block/item damage and chains; visual footprint must match affected cells. |
| WPN-MINE / landmine | Next placement explodes only when someone steps on it, never merely because time passes. | Arming/owner-exit grace, swept step detection, eligible actors, visibility, capacity and remote ownership; explicitly decide whether other blasts can trigger/destroy it without contradicting contact-only detonation. |

## Verification and incremental release requirements

For each behavior, add deterministic domain tests and interaction fixtures before activation;
update versioned rules, contracts/projections, generated types, replay and reconnect as affected.
Verify native browser controls and both players' views, including non-disclosure of hidden data.
Retain the classic preset and its 42-rule regression matrix. Use separate acceptance scenarios
for expanded profiles rather than silently changing baseline expectations.

Playtest combined speed/tolerance/fuse extremes, equipment replacement, multiple simultaneous
effects, expiry/death boundaries, moving/airborne/frozen bombs, capacity and chain reactions.
Retain bounded scheduling, performance budgets, EN/PL documentation and full release gates,
including same-image Compose/Kubernetes acceptance. Bots, mobs and larger matches remain
separate tracks; their future interactions must be specified without making them prerequisites.
