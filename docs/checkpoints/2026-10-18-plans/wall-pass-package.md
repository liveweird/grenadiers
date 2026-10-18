# Wall Pass — v0.3.43 implementation package

Adopted coordinator summary of independent architect package. Inspected Remote
baselinee01d32ccba67e3390afcc906f0eaebedc0f56f90; depends on Teleporter42 and
preceding Landmine/Cloak/Stopwatch/InvisibleBombs packages. Plan only; no executed
tests/native claims. Keep gameplay-v10/rules5/generator2/journal2.

## Capability and private exit grace

Activate existing Equipment::WallPass/Inventory::can_pass_soft_walls as persistent,
replaceable equipment, independent weapon slot and no timer. Capability requires
item enabled AND equipped. Collection occurs after movement, so cannot alter the
collection update's movement. Death/reset clear; resume preserves. Passage only
through soft blocks; hard walls/bounds/solid bombs/fire/Contact remain effective.
No OilBarrel support is claimed before its separate release.

Private actor soft-block grace is separate from bomb_grace and keyed by CellId.
640-unit footprint versus1024cell overlaps at most4 soft cells. Centralize live
equipment transitions:

- WallPass -> another equipment/live loss: capture EXACT existing positive-area
  overlapping soft blocks. WallPass ->WallPass grants no accumulating grace.
- Acquiring WallPass clears obsolete soft grace. NonWall ->nonWall only preserves/
  prunes existing grace, never grants newly encountered cells. Death clears with
  no grant. Heart consumption after replacement must retain existing soft grace.
- Each cell permits movement until whole footprint exits; tangency ends grace.
  Prune after EACH actual movement segment, before a later segment can reenter;
  also after destruction/teleport/death/reset. No monotonic-distance requirement.
- Never regrant on admission/resume/replacement/stalled input. Living nonholder
  grace must be <=4 existing currently overlapping blocks. Reentry is blocked.
  No unequip command solely for testing; test legitimate Heart replacement and
  later Kick replacement. Do not weaken ordinary actor construction for fixtures.

## Collision, placement, pickup, Teleporter

Extract focused collision helper if movement would exceed400 lines. Preserve
budget/turn assistance/raw input/actual two axis-aligned traces; no diagonal AABB
or endpoint hazard substitutes. ActiveWallPass or valid own soft grace ignores
soft collision only. Existing bomb grace/Contact/swept fire/final damage unchanged.
Stopwatch/Reverse/Slow behavior unchanged; no immunity.

Bomb placement remains CENTER-CELL legality: must be floor, no soft block/bomb.
Cannot plant centered inside a block; free center may plant while footprint
overlaps neighboring block. Failed placement consumes no ID/capacity/weapon/
equipment; Diarrhea same. Owned Remote manual-trigger priority works inside block.
Passage never destroys/reveals/collects concealed or pending pickup contents.
No wire/art hint/count leaks. Existing destruction/fire-clear reveals normally.

Teleporter landing checks EACH arriving actor's capability and own EXISTING soft
grace, alongside hard/bomb/Contact/fire checks. Holder may land inside soft block;
nonholder needs own grace for same cell. No transferred/new grace. Precompute
both checks/retained sets; success prunes to destination overlap, failure changes
no position/marker/residual/turn/grace. Swap out of block may fail for nonholder
opponent. No swept teleport, arrival pickup or extra damage pass.

## Atomicity, protocol, presentation

Late timed-pickup overflow restores whole step including equipment/grace capture/
prune, movement, pause/bomb/Contact, pickups/events/audiences. Include private
grace in Actor/State clone/equality and replay. Test replacement multi-block,
Heart consumption, destruction, resume inside, teleport success/failure, death/
reset. No new input/RNG/journal format.

Existing v10 equipment/pickup DTO suffices. Canonical validation before hiding
checks private bounded grace/world/actor relation and legitimate soft positions.
Carry private facts where necessary but serialize no grace/countdown/exit marker.
Web must NOT reject every nonholder soft overlap, because valid grace is private.
Dead positions may remain inside per existing policy. Hidden malformed facts
still reject. Archivedv9/v8/earlier remain pinned and reject actual new equipment/
pickups/history/private-grace semantics. Preserve paired encoding/admission
atomicity and all asymmetric publication/resume/final routes.

Distinct pickup/equipment cue and EN/PL text: soft passage/hard+bomb blocking/no
fire protection/escape grace. No timer/weapon indicator. Keep authoritative
blocks fully rendered; actors legible in actor layer, no concealed transparency.
Equipment cue disappears on replacement even while private grace permits exit.
Cloak removes whole actor/cue/shadow/motion/accessibility as usual. Existing pickup
audio only; no passage/failed-placement/grace/resume synthesis. Preserve320px,
reduced motion, snapping, stale frames and teardown.

## Catalog and exhaustive checks

bit3=8; ALL3145712->3145720; available17->18; masks131072->262144.
Generation append weight15 AFTERTeleporter, pool365->380 interval[365,380), sole115.
Canonical wire order begins wallPass,heart,remoteBomb,piercingBomb,powerBomb,
landmine,shield,stopwatch,invisibilityCloak,goldenFlame,shortFuse,diarrhea,slowFoot,
shortFlame,longFuse,reverseControls,invisibleBombs,teleporter. Append-generation
order differs from wire order; preserve ALL prior maps/RNG/weights/layers.

Existing web it.each-permask and server fullroom/start/replay-permask are costly.
Measure walltime/registrationmemory/groups BEFORE changes. Web may batch exact
existing full-message assertions into disjoint deterministic1024-mask ranges;
reuse compiled AJV, fresh state per iteration, rawmask failure context, no giant
casearray. Prove union exactly[0,262144), no sampling. Keep all invalid cases,
floors, exhaustive algebra/conversion/generation boundaries and ALL old expensive
room/replay assertions. Default retains replay enumeration; if measured extension
is impractical, propose separately reviewed test design with evidence, never
silently replace old replay checks with parsing. Don't crossproduct long routes.

## Ownership and acceptance

1. Domain: enabled_items/pickups/simulation_pickups, equipment transition, Actor
   grace/collision/pruning, teleport landing, generation/focused tests/route.
2. Protocol/runtime after grace agreement: v10 trustedcapture/binding/validation,
   settings/archive guards, publication/admission/resume and replay.
3. Web after contract: semantics/settings/HUD/arena art/scene/ENPL/audio/visibility.
4. Coordinator: measured test batching, native/Compose/config/scenario/CI/version/
   docs. Serialize sharedfiles; independent domain/hazard +protocol/privacy reviews.

Required cases: acquisition/replace/slots/disabled parity; footprint/tangency/
percell-prune/no-reentry/two-segment/max4/noresumegrant; Heart+later replacement/
consumption/destruction/death/reset/ReverseSlowfreeze; soft versushardbomb/fire/
Contact; center-cell planting/nonconsumption; concealed/pendingprivacy; both
teleport legality/grace/atomicity; whole lateoverflowrollback/replayprivategrace;
hiddeninvalid/archive/pairencodefailure; renderer/audio/ENPL. Preserve16bombs,
195flames,96events,<64KiBframes,<=64byte records,<4MiB journal,400-line ceiling.

## Proposed native route — unexecuted

First ENTIRE pure-core seed31 soleWallPass ordinaryPress/Release/PlaceBomb route,
pinpickup3,1/intactsoft3,2/hard2,2/allescapes. ENdesktop/PL320 reducedmotion ready.
Reveal3,1 usingbomb2,1, escape1,1->1,2/waitfireclear/collect. WalkSouth into intact
soft3,2 andstop; bothviewskeepblock/equipment/nohiddenpickup. Pressbomb once:
no placement/newID/consumption/capacity/cue. AttemptWest intohard2,2 proveclamp;
ordinarykeysrestorealignment. Cheapcompositorcrops showequippedactoroverstill
renderedblock+ENPLHUD. Normalresumeinside preservesgear/position/nohistoryaudio,
thenexitNorth3,1. Replacementlossgrace remainscore/runtimeproof, not falsely
claimednative in sole-itemfixture. Actualkeys/UI/read-onlyobservations/reconnect,
releasefinally; no injection/seedsearch/retries/weakenedtimeouts.

Run all/coverage/security/k8s unchangedgates/bothprofiles/floors, measure costs.
Independentreviews then fresh cumulative sameimageCompose/K includingaudio,
finalCI/evidence. Correctedpreflight requirespinnedNode+YAML/exactstdout/failfast
andpre/postexternaldependencyhashes. Pairedrollback/reload/newrooms, oldmask
rejects; no DB migration.
