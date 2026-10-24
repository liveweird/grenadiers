# Gameplay and presentation backlog

TURN, CONFIG and the first ART slice are implemented. Published v0.3.45 contains twenty
optional pickups: Heart, Shield, Golden Flame, Short Fuse, Short Flame, Long Fuse, Reverse
Controls, Diarrhea, Slow-foot, Piercing Bomb, Power Bomb, Remote Bomb, Landmine and Invisibility
Cloak, Stopwatch, Invisible Bombs, Teleporter, Wall Pass, Kick and Punch. Reviewed implementation
source, scoped acceptance, publication and localhost promotion are complete. The v0.3.46 source
candidate adds Bracelet as the twenty-first optional pickup on that published parent. Bracelet
native acceptance, final CI and publication remain pending; Oil Barrel v0.3.47 follows before
ART-ATLAS.

[v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43) publishes Wall Pass as the
eighteenth of 22 requested item types. Published source `c086f719ea470b9fe24963adef84b5e3bdf3e17a`,
tree `d74fad8a1c870f2834fca48f80ea4e38f0c512d9`, and annotated tag
`c8395fd7dcc5701842644967802e61c76211931c` pass first-attempt candidate, master and tag workflows;
the exact accepted image `sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`
is promoted locally. Native acceptance remains attributed to H2 source `11d0dc7`; reviewed
artifact equality connects corrected `5dbcd58` to the published source without claiming unchanged
Docker inputs, a new Rust compilation or new native execution. See the
[verification record](../verification.md) for the delivery chain.

[v0.3.44](https://github.com/liveweird/grenadiers/releases/tag/v0.3.44) publishes Kick as the
nineteenth requested item from source `f32fd86d8f0c7c4bebba8fa60d1c66c65d1a174a`, tree
`a0f2addb9204c40b4b30be2830fde4df368f019c`. Its accepted image was promoted locally before Punch v0.3.45.
Bracelet and Oil Barrel remain before ART-ATLAS.

At the 21 October 2026 16:47 UTC native acceptance checkpoint, Invisibility Cloak was an
**unreleased source candidate**. Corrected source `1c6724f` and
image `960976c9` pass a fresh 51 Compose plus 51 Kubernetes matrix: all 44 groups/reports,
106 runtime PNGs, 34 runtime records and 17 Kubernetes lifecycles pass with zero retries or
carried results. Independent raw review is clear; 2,298 payloads are sealed and the 2,299-file
durable copy is byte-verified. At that checkpoint, PR #84 was draft at the earlier `97a85d4`; both exact-head
seven-job candidate workflows, merge/post-merge CI, publication, exact-image promotion and tag
CI remained required. Earlier Cloak attempts remain source-specific history with zero carry.
Thirteen of the 22 requested item types were published, and Landmine v0.3.37 was published and
deployed locally. See [release notes](https://github.com/liveweird/grenadiers/releases) for current
release status. CI retains one image, two complete
browser shards (23 item/default plus 28 classic cases), and the existing 20-minute job limits.

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
v0.3.33 Slow-foot passes local checks and independent backend/frontend reviews, 22 latency cases
per target, 47 static scenarios and fresh 46 Compose plus 46 Kubernetes cases (**92 total**) with
zero retries. Candidate and post-merge CI pass, and
[v0.3.33](https://github.com/liveweird/grenadiers/releases/tag/v0.3.33) is published as a source
prerelease. Its final raw source is integrated into the v0.3.34 Piercing Bomb candidate, which
retains the feature and version. Repository, E2E lint, Knip, type checking and 48-scenario parity
pass at the integrated v0.3.34 source. Fresh same-image acceptance now passes all 47 Compose and
47 Kubernetes cases with zero retries; independent evidence review and its final seal are clear.
Candidate and post-merge CI pass, and [v0.3.34](https://github.com/liveweird/grenadiers/releases/tag/v0.3.34)
is published as a source prerelease. Its final raw source is integrated into the v0.3.35 Power
Bomb candidate, which retains the feature and version and passes local implementation review.
Fresh same-image acceptance passes all 48 Compose and 48 Kubernetes cases with zero retries;
independent evidence review, final sealing, both five-job candidate CI runs and the five-job
post-merge run are clear. PR 81 is merged and
[v0.3.35](https://github.com/liveweird/grenadiers/releases/tag/v0.3.35) is published as a source
prerelease and its accepted image is deployed locally. Its CI-corrected accepted raw source is
integrated into the v0.3.36 Remote Bomb candidate. Remote Bomb retains its reviewed one-charge
manual-trigger behavior and passes fresh 49/49 Compose plus 49/49 Kubernetes acceptance, zero
retries, independently audited and sealed. Both candidate CI runs and post-merge CI pass all
five jobs and all 49 browser cases; PR 82 is merged and [v0.3.36](https://github.com/liveweird/grenadiers/releases/tag/v0.3.36)
is published. Its accepted image is deployed locally. Its CI-corrected accepted raw source is
integrated into this v0.3.37 Landmine candidate. Landmine retains reviewed contact, grace and
immutable-planning behavior; fresh same-image 50/50 Compose plus 50/50 Kubernetes acceptance
passes, with zero retries, independent raw review clear and evidence sealed. Both candidate
workflows and post-merge CI pass five jobs and all 50 browser cases; PR 83 is merged, v0.3.37
is published and its accepted image is deployed locally. Verification preserves source-specific evidence
and earlier failed or interrupted attempts. The accepted Landmine raw source is integrated into the v0.3.38 Invisibility Cloak source
candidate, retaining its corrected reveal-position driver and private-recipient behavior. Corrected source `1c6724f` passed fresh 102/102 native acceptance with independent audit and durable seal. At the native acceptance checkpoint, candidate/post-merge CI, publication and promotion remained pending. The other eight
requested pickups remain outside these increments. Richer
transitions and artwork remain follow-up work. See [pixel-art presentation](pixel-art.md),
[expanded item implementation](expanded-items.md), and
[release notes](https://github.com/liveweird/grenadiers/releases).
At the 21 October 2026 18:44:23 UTC native acceptance checkpoint, v0.3.39 source `272b950` and
image `9a4dc108` passed a fresh 51 Compose plus 51 Kubernetes matrix with zero retries or carried
passes. Independent raw audit is clear; 2,320 payloads are sealed and the 2,321-file durable copy
is byte-verified. The source stages the paired gameplay-v9 false-only freeze-status foundation
without activating another pickup. It integrates accepted raw v0.3.38 source `04c070c5`, whose
exact tree `d1dc83fc` is published as v0.3.38 from merge `3c210c23`; the accepted image `960976c9`
is promoted locally. Fourteen of the 22 requested item types are published. At that checkpoint,
v0.3.39 had not been normalized, pushed, merged, published or promoted; candidate, post-merge and
tag CI remained pending. Earlier v0.3.39 source `30c051c`, failed v0.3.38 PR head `97a85d4`, and
source `4d9a751` remain historical with zero carry. Full acceptance evidence is recorded in
[verification](../verification.md); see the
[release notes](https://github.com/liveweird/grenadiers/releases) for current release status.

At the 21 October 2026 20:56:41 UTC v0.3.40 native checkpoint, corrected source `fa25467`,
tree `07ac34a8`, and image `ec8604af` passed all 32,768 masks in debug, release and coverage plus
fresh 52 Compose and 52 Kubernetes cases, all 46 groups, with zero retries or carried passes. The
failed `bd9d064` attempt remains historical: 51 Compose cases passed, the Stopwatch oracle failed
once, and Kubernetes did not start. The corrected oracle uses the independently visible deadline,
`deadline - placedAt - 1`; product behavior did not change. Independent raw audit and durable seal
are clear. v0.3.40 is now published and its accepted image is the local release baseline. Fifteen of
22 requested item types are published through v0.3.40. See
[release notes](https://github.com/liveweird/grenadiers/releases) for current release status.

At the 22 October 2026 00:55:23 UTC sealed checkpoint, v0.3.41 corrected source `cb10ec7`, tree
`9d38e0cb`, and image `d08b93af` add opt-in [Invisible Bombs](invisible-bombs.md) as the sixteenth
item and pass all 65,536 masks plus fresh 53 Compose and 53 Kubernetes cases, all 48 groups, with
zero retries or carried passes. The final browser scenario uses two independently collected
600-tick privacy windows and a polled sighted detonation witness; these oracle corrections changed
no game behavior. Independent audit is clear and 2,492 payloads are durably sealed. Hosted candidate, merge, tag
and promotion checks have not run. Six requested types remain: Teleporter,
Wall Pass, Kick, Punch, Bracelet and Oil Barrel. ART-ATLAS follows the completed feature batch.

TURN, CONFIG and the first ART slice are implemented. Published v0.3.45 contains twenty
optional pickups: Heart, Shield, Golden Flame, Short Fuse, Short Flame, Long Fuse, Reverse
Controls, Diarrhea, Slow-foot, Piercing Bomb, Power Bomb, Remote Bomb, Landmine and Invisibility
Cloak, Stopwatch, Invisible Bombs, Teleporter, Wall Pass, Kick and Punch. Source, publication and
exact-image promotion are complete. Bracelet and Oil Barrel remain.
Authoritative publication
status is recorded in [GitHub Releases](https://github.com/liveweird/grenadiers/releases), with source-bound
acceptance records in [verification](../verification.md). Richer transitions and artwork remain
follow-up work; see [pixel-art presentation](pixel-art.md) and
[expanded item implementation](expanded-items.md).
Updated 2026-10-24 after integrating the Bracelet source candidate with published Punch.
Release acceptance is recorded in
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
when scheduling; continue focused PRs, version bumps and the existing release gates. ART-ATLAS
follows completion of the whole expanded-item batch rather than interrupting individual item
activation releases.

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

Queued after the entire expanded-item gameplay batch. The existing code-generated PixiJS
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
| EQ-WALL / wall pass | **Published in v0.3.43.** Phase through destructible soft blocks. | Hard walls/borders, bombs and hazards stay authoritative; private per-cell exit grace resolves replacement while overlapping a block. Fresh source gates, 55+55 same-image native cases, first-attempt hosted CI, publication and exact-image promotion pass with independently reviewed evidence. |
| EQ-HEART / heart | Absorb one lethal explosion or enemy collision, then vanish. | Simultaneous/overlapping hits, remaining in flame, protection precedence with Shield and post-hit grace. Enemy collision applies when an enemy damage system exists. |

## Timed buffs: multiple active effects allowed

| ID / pickup | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| BUFF-SHIELD / shield | Temporary immunity to bomb blasts, monsters and other configured damage. | Damage-class coverage, duration, expiry while in danger and interaction with Heart. |
| BUFF-CLOCK / stopwatch | **Source/native verified at the v0.3.40 checkpoint.** V9 publishes authoritative voluntary freeze and Timed-bomb pause state. | Exact 180-tick collection, checked fuse-history provenance, two-holder deduplication, thaw/input handling and rollback pass fresh 52+52 native acceptance at source `fa25467`; independent audit and durable sealing are clear, while release gates remained pending at that checkpoint. |
| BUFF-CLOAK / invisibility cloak | **Prepared at the v0.3.38 native acceptance checkpoint.** Opponents receive only identity, seat and alive status for 600 physical ticks. | Event-time private cues, complete actor-root removal, current-position reveal, reconnect deadline preservation and paired checked publication are covered locally; corrected source `1c6724f` passed fresh 102/102 native cases with an independent audit and durable seal. Candidate/post-merge CI, publication and promotion remained pending at that checkpoint. |
| BUFF-GOLD / golden flame | New bombs use maximum blast radius while active. | Maximum derives from the selected ruleset; define precedence with Short-flame and expiry. |
| BUFF-FUSE / short fuse | Decrease new bombs' detonation delay by 50%. | Integer-tick rounding/minimum, repeated pickups and composition with Long fuse. |

## Curses and hostile pickups

Multiple timed curses may coexist. Oil barrel creates a persistent hazard; Teleporter is an
instant action. Keep both in this requested pickup family without forcing them into a timer model.

| ID / pickup | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| CURSE-DROP / diarrhea | Automatically drop bombs whenever capacity is available, up to the player's bomb limit. | Bounded attempt cadence, legal placement, capacity release, freeze, special-weapon consumption and remote-trigger behavior; no unbounded same-tick loop. |
| CURSE-SLOW / slow-foot | **Published in v0.3.33.** Each pickup halves current effective speed again. | Independent 600-tick lifetimes, exact expiry restoration, six-stack cap, exact fixed-point arithmetic and zero-budget ticks have implementation coverage; fresh 92/92 native acceptance plus candidate and post-merge CI pass. |
| CURSE-RANGE / short-flame | Reduce new bombs to one adjacent tile in each blast direction. | Center cell still explodes; precedence with Golden flame, upgrades and special footprints. |
| CURSE-FUSE / long fuse | Increase new bombs' detonation delay by 100% (double it). | Rounding, stacking/re-pick policy and composition with Short fuse, remote bombs and mines. |
| CURSE-REVERSE / reverse controls | Up moves down, left moves right, and vice versa. | Apply to logical movement after personal remapping; define held-key transition/expiry without stuck movement or stale replay. |
| CURSE-BLIND / invisible bombs | The afflicted player cannot see any bombs, including their own. | Recipient-specific bomb/event/effect projection and reconnect; explicitly define retained collision clues, fuse sounds and blast warnings. |
| CURSE-OIL / oil barrel | Before detonation it blocks passage like a soft wall; after detonation it burns indefinitely. | Define how acquisition creates/places the barrel, detonation/chain triggers, blast footprint and destruction; bound creation and specify cleanup at round end without expiry during play. |
| CURSE-SWAP / teleporter | **Published in v0.3.42.** Immediately attempt the sole two-player rival swap. | Atomic complete-footprint safety, unsafe consumption, actor-specific overlap grace, persistent position epoch, reconnect and privacy are covered without RNG. Fresh 54 Compose plus 54 Kubernetes cases pass; distinct first-attempt master/tag workflows pass and the exact accepted image is promoted locally. |

## Special weapons: one single-use slot

| ID / weapon | Requested behavior | Decisions / acceptance focus |
| --- | --- | --- |
| WPN-REMOTE / remote bomb | **Shipped in v0.3.36.** The next successful manual placement creates an untimed bomb; the next retained bomb action detonates it. | Independent stored weapon, one live Remote per owner, capacity/allocator independence, fire and chain ignition, reconnect and replay pass local checks and independent reviews; fresh 98/98 native cases pass and the evidence is independently audited and sealed. Both five-job candidate CI runs and post-merge CI pass 49 browser cases; PR 82 is merged, the source release is published and its accepted image is deployed locally. |
| WPN-LANDMINE / landmine | **Shipped in v0.3.37.** Successful manual placement creates a passable untimed Contact bomb. | Per-actor exit grace, swept entry, silent blast destruction, immutable planner priority, replay and checked v8 publication pass local checks; fresh 100/100 native acceptance is independently audited and sealed. Both candidate workflows and post-merge CI pass five jobs and 50 browser cases; PR 83 is merged, the release published and accepted image deployed locally. |
| WPN-PIERCE / piercing bomb | **Shipped in v0.3.34.** The next successful manual bomb's blast continues through destructible blocks. | One-charge replacement, retained failures, ordinary automatic bombs, captured blast identity, immutable same-tick chain planning and hard-wall stopping are covered locally with independent review clear; fresh 94/94 native acceptance is independently audited and sealed; primary candidate/post-merge CI passed and the source release is published. Its additional tag security failure is documented and the Vite fix is included in the published v0.3.35 release. |
| WPN-POWER / power bomb | **Shipped in v0.3.35.** The next successful manual bomb damages every floor cell within its Manhattan blast radius, ignoring line of sight. | One-charge replacement, retained failures, hard-wall holes, complete block and bomb coverage, immutable mixed-chain planning, captured stats and distinct presentation pass local checks and independent reviews; fresh 96/96 native acceptance, final seal, both candidate CI runs and the five-job post-merge run pass. PR 81 is merged, the source release is published and the accepted image is deployed locally. |

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

Complete the entire expanded-item activation batch before starting ART-ATLAS. Bracelet is the
current bounded source increment (v0.3.46); Oil remains after it. ART-ATLAS begins
only after those item increments and their release gates are complete.
