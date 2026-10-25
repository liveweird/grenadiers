# Gameplay and presentation backlog

All 22 requested item types shipped in published
[v0.3.47](https://github.com/liveweird/grenadiers/releases/tag/v0.3.47). The artist handoff and
runtime atlas shipped in v0.3.48/v0.3.49, and published
[v0.3.53](../releases.md#v0353--2026-10-25) includes the adopted full AI-generated redraw.
A later owner-supplied Napoleonic sheet is adopted on master in
[PR #162](https://github.com/liveweird/grenadiers/pull/162); use the artist handoff for the
canonical artwork and release records for published/deployed image status.
Those delivered features are context, not open backlog work. Dated source, image, matrix,
candidate, “pending,” and “unreleased” statements retain their original checkpoint meaning in the
[historical backlog](../history/backlog.md); use [releases](../releases.md) and
[verification](../verification.md) for current status.

The remaining requests are playtest-led turning refinement, wider shared room configuration,
further art/animation work, and cross-feature tuning. Do not infer that an unchecked review item
or a historical proposal is implemented merely because the item batch shipped. The
[implementation plan](implementation-plan.md) remains the milestone reference, and the
[rules](rules.md) remain a discussion/design reference rather than an implementation report.
Use [settings](settings.md) and the linked feature documents there for implemented behavior.

<a id="priority-and-delivery-sequence"></a>

## Current backlog index

| Area | Current baseline | Remaining request |
| --- | --- | --- |
| [TURN](#turn-forgiving-movement-and-corner-alignment) | Quarter-tile assistance shipped in v0.3.16 with native keyboard and collision/fire regressions. | Human playtesting and evidence-led tolerance adjustment; specify buffered turns separately if still needed. |
| [CONFIG](#config-shared-gameplay-configuration-panel) | Four numeric controls, all 22 item switches, presets, creator authority, readiness clearing, match locking, rematch editing, and reconnect authority are implemented shared room rules. | Add round timeout, loadout/cap, flame/pacing, arena-density/frequency/weight, additional pool, duration, and related cross-field controls below. |
| [ART](#art-modern-retro-pixel-art-presentation) | Artist tooling, runtime atlas, and the v0.3.53 redraw are delivered. | Extend animation, environmental detail, layered authoring sources, accessibility, and measured presentation without changing simulation geometry. |
| [Playtest and interactions](#verification-and-incremental-release-requirements) | The expanded item batch and its activation gates completed in v0.3.47. | Playtest combinations and retain deterministic, private, bounded behavior when tuning or extending them. |

## TURN: forgiving movement and corner alignment

The inclusive assistance window widened from 128 to 256 of 1,024 cell units in v0.3.16 while
historical profiles remained available. Regression and native keyboard acceptance cover the
natural 192-unit wall-stop offset. Corner tolerance is already exposed as a shared setting.
Human tuning remains open: adjust its range or default only against playtest evidence.

- Assist alignment toward the requested corridor without requiring pixel-perfect positioning.
- Preserve fixed-point movement, swept collision, and swept flame contact. A larger window must
  not allow wall clipping, tunneling, extra diagonal speed, or hazard avoidance.
- Test early/late turns, both axes and directions, every speed level, narrow openings, bombs,
  reversal, held inputs, and network delay. Evaluate tolerance before proposing a separately
  specified buffered-turn mechanic.

## CONFIG: shared gameplay configuration panel

Starting speed, turn tolerance (1/16–¼ tile), wins to finish, bomb fuse, all 22 item switches,
Expanded/Classic presets, and the shared authority lifecycle are implemented; see
[settings](settings.md). Keyboard and audio settings remain personal preferences. The table keeps
all original wishlist rows while distinguishing the shipped baseline from open extensions.

| Setting | Requested semantics | Status / remaining extension |
| --- | --- | --- |
| Player speed | Starting cells/second and its relationship to speed upgrades and caps. | Shipped: 3–5 cells/second in 0.5 steps, with pickups capped at 5. Further range/cap tuning is playtest-led. |
| Corner alignment tolerance | Maximum centerline correction, displayed as a tile fraction. | Shipped: 1/16–¼ tile in 1/16 steps. Further range/default tuning is playtest-led. |
| Number of rounds | Explicitly choose wins-to-finish versus fixed round count; the current baseline is first to three wins and draws award no point. | Shipped as configurable wins-to-finish (1–9); a distinct fixed-round-count mode remains an open design request. |
| Bomb fuse | Seconds from successful planting to ordinary detonation. | Shipped: 1–5 seconds in 0.5-second steps. Further range/default tuning is playtest-led. |
| Round time limit | Running duration before timeout resolution. | Open. |
| Starting bomb capacity and blast range | Initial loadout with separately validated upgrade caps. | Open. |
| Flame duration | How long exploded cells remain hazardous. | Open. |
| Countdown and inter-round hold | Pacing between readiness, results, and subsequent rounds. | Open. |
| Soft-block density and item frequency | Arena openness and upgrade availability while retaining safe-opening validation. | Open, including safe cross-field combinations. |
| Item pools and weights | Enable equipment, buffs, curses, and weapons individually; retain a classic preset. | All 22 enable switches and the Classic preset shipped; weights and additional pool controls remain open. |
| Effect durations | Timed buff/curse lifetime and documented refresh/stack policy. | Open as configurable controls; implemented effects retain their current fixed, tested policies. |

The authority policy is implemented: the connected original creator edits only while unready in
the waiting lobby; a genuine applied change clears both players' readiness and both see the
accepted values. Settings lock through automatic rounds. After mutual rematch consent, editing
resumes in the unready lobby. Reconnect preserves creator authority while the seat is reserved;
permanent departure or reservation expiry leaves the last accepted configuration locked, and a
replacement occupant does not become creator. Presets require Apply rules.

New controls must preserve that authority/lifecycle baseline, validate cross-field combinations,
avoid exposing active seeds or concealed contents, and extend contracts, replay metadata, and
reconnect baselines where affected. Those requirements do not claim that the open controls exist.

## ART: modern retro pixel-art presentation

The code-generated v0.3.18 slice, v0.3.48 handoff/export/preview, v0.3.49 runtime atlas adoption,
and v0.3.53 full redraw are delivered. The 95-entry manifest, frame rectangles, pivots, anchors,
binary alpha, and texture budget remain stable. The procedural template remains the parity
baseline and retains its known two-pixel vertical gap in flame frame 1; the adopted redraw is
gap-free. See the [artist handoff](../../assets/art/README.md).

Remaining presentation requests:

- Maintain a documented palette, pixel grid, sprite scale, lighting, and silhouette direction;
  review a small mockup and representative animations before another full revision.
- Extend directional movement, idle/death states, bomb fuses, explosions, destruction, pickup
  effects, status cues, arena borders, and environmental detail without obscuring gameplay.
- Keep crisp nearest-neighbor scaling, stable character foot anchors, seamless flame pieces,
  player distinctions beyond color, readable EN/PL controls, reduced-motion/effects behavior,
  and clear focus/connection states.
- Keep bounded particles, texture memory, and animation work; verify desktop/narrow screenshots,
  context restoration, provenance/licenses, and measured rendering performance.
- Preserve transparent stable-ID authoring sheets and generated overview/export validation.
  Retain layered sources such as Aseprite files as animation production grows; optional sliced
  PNGs must remain reproducible from the canonical PNG/JSON source.

No artwork is generated by this backlog document.

<a id="art-atlas-artist-handoff-and-sprite-sheet-pipeline"></a>

### Artist-sheet status

ART-ATLAS tooling and gameplay integration are complete; new animation production and future
artist revisions remain backlog work. Preserve the collision geometry, authoritative visibility,
hazard readability, renderer ownership, animation timing, IDs, padding, transparency, bounds,
and texture budgets already guarded by the tooling.

<a id="effects-inventory-and-status-foundations"></a>

## Delivered feature wishlist and future hooks

The v0.3.47 batch implements the requested slot model: one persistent equipment slot, multiple
different timed buffs/curses with bounded stacking, and one single-use special-weapon slot.
Basic capacity/range/speed upgrades remain separate. Picking replacement equipment or a weapon,
round/death cleanup, captured bomb properties, privacy, replay, reconnect, and deterministic
interactions are implemented behavior. Consult [settings](settings.md) and its linked feature
documents for current semantics; [expanded items](expanded-items.md) and the
[rules](rules.md) provide architecture/design context, not proof that a proposal shipped.

The catalog below preserves the user's requested behaviors. It is delivered, not a list of 22
open implementations. Conditional monster/enemy interactions remain future only if those separate
systems are introduced.

## Equipment: one persistent slot

| ID / item | Requested behavior |
| --- | --- |
| EQ-KICK / boot | Walking into a planted bomb slides it until a wall, another bomb, or a player stops it. |
| EQ-PUNCH / boxing glove | Punch an adjacent bomb three tiles forward over obstacles, wrapping at arena edges. |
| EQ-SHOVE / bracelet | Shove rival players when touching them. |
| EQ-WALL / wall pass | Phase through destructible soft blocks while hard walls, bombs, and hazards remain authoritative. |
| EQ-HEART / heart | Absorb one lethal explosion or enemy collision, then vanish; enemy-collision protection applies only if that separate damage system is added. |

## Timed buffs: multiple active effects allowed

| ID / item | Requested behavior |
| --- | --- |
| BUFF-SHIELD / shield | Temporary immunity to bomb blasts, monsters, and other configured damage; non-bomb coverage remains conditional on those systems. |
| BUFF-CLOCK / stopwatch | Freeze voluntary actor actions and pause timed bombs for the bounded holder window. |
| BUFF-CLOAK / invisibility cloak | Hide the actor from the opponent for 600 physical ticks without leaking private cues. |
| BUFF-GOLD / golden flame | New bombs use the selected ruleset's maximum blast radius while active. |
| BUFF-FUSE / short fuse | New bombs use half the ordinary detonation delay while active. |

## Curses and hostile pickups

Oil Barrel is a persistent arena hazard and Teleporter is an instant action; they remain in this
requested family without being forced into the timed-curse model.

| ID / item | Requested behavior |
| --- | --- |
| CURSE-DROP / diarrhea | Automatically drop bombs when capacity and placement permit, bounded by the player's limit. |
| CURSE-SLOW / slow-foot | Each stack halves effective speed again within the bounded stack/lifetime policy. |
| CURSE-RANGE / short flame | New bombs reach one adjacent tile in each direction while active. |
| CURSE-FUSE / long fuse | New bombs use double the ordinary detonation delay while active. |
| CURSE-REVERSE / reverse controls | Logical directions are inverted after personal key remapping. |
| CURSE-BLIND / invisible bombs | The afflicted recipient cannot see bombs, including their own, while public hazards remain truthful. |
| CURSE-OIL / oil barrel | Generated sites reveal as solid arena objects and burn until round end after ignition. |
| CURSE-SWAP / teleporter | Immediately attempt the sole two-player rival swap and consume even when unsafe. |

## Special weapons: one single-use slot

| ID / weapon | Requested behavior |
| --- | --- |
| WPN-REMOTE / remote bomb | The next successful manual placement creates an untimed bomb; the next retained bomb action detonates it. |
| WPN-LANDMINE / landmine | Successful manual placement creates a passable untimed contact bomb. |
| WPN-PIERCE / piercing bomb | The next successful manual bomb continues through destructible blocks. |
| WPN-POWER / power bomb | The next successful manual bomb damages floor cells in its Manhattan radius without line of sight. |

## Verification and incremental release requirements

For future tuning or extensions, update deterministic domain tests, interaction fixtures,
versioned rules, contracts/projections, generated types, replay, and reconnect where affected.
Verify native browser controls and both recipient views without disclosing hidden data. Retain the
classic preset and its 42-rule regression matrix; use separate acceptance scenarios for expanded
profiles instead of silently changing baseline expectations.

Playtest combined speed/tolerance/fuse extremes, replacement, simultaneous effects,
expiry/death boundaries, moving/airborne/frozen bombs, capacity, and chain reactions. Retain
bounded scheduling, performance budgets, EN/PL documentation, and applicable same-image
Compose/Kubernetes release gates. Bots, mobs, larger matches, and their feature interactions
remain separate tracks and are not prerequisites for the delivered item catalog.
