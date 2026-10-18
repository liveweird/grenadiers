# Kick — v0.3.44 implementation package

Adopted coordinator summary of independent architect package; inspected Remote
baselinee01d32ccba67e3390afcc906f0eaebedc0f56f90. Depends WallPass43/priorpackages.
Plan only. Keep gameplay-v10/rules5/generator2/journal2. Grid motion is deliberate;
continuous subcell physics would require larger independent contract/design.

## Contact and motion

Kick is persistent replaceable equipment, no actionkey/timer/consumption. Walking
into stationary solid Timed/Remote bomb at T arms it; first1cell cardinalhopT+1,
thenT+11,T+21...; blockedhop stops in lastlegalcell. Contactmines excluded.
Punch/Bracelet/Oil unavailable. Render authoritative cellsteps, no prediction.

Request comes from actual voluntary FORWARD movement collision, not adjacency/
packet/finalpos/rayguess. Requires alive-at-movement, eligibleunderStopwatch,
enabledKickequipped, positivebudget attemptedblockingface and unique nearest
stationary-at-update-start bomb with no equalnonbombblocker. EffectiveReverse
direction applies; lane-centering correction doesn'tkicksideways, forwardsegment
maykick. ZerobudgetSlow/frozen/stationary/expiredinput do notkick. Exitgrace
meansnocollision so leavingownbomb doesn'tkick. Ambiguousmultiplebombfaces reject.
<=1request/actor. Collectbothrequests before resolution: samebomb chooses
RoundAssignmentpriority, no loserfallback; distinctbombs botharm. Moving-at-start
bombs cannotredirect/rephase evenifstoppedthisupdate. Applyacceptedrequestbefore
oldfiredamage; laterkickerdeath/replacementdoesnotcancelmotion. ContactTstillclamps
actor/bombstaysput.

Private perbomb optionaldirection+countdown0..9 and persistent hasBeenKicked flag.
Newbomb none/false; armingcountdown0; nextrunningupdatephasehops; successsets9,
subsequentupdatesdecrementonceuntilnextduehop. <=16records; noabsolute tickadd,
speedmodifier/accumulatedbudget/extraoverflow. Stopped canbekickedlater, no
speculativecooldown. Remove withbomb/reset. PreserveID/owner/range/blast/placedAt/
capturedfuse/Stopwatchpauseprovenance. Onlycell/private motionhistory changes.

## Exact phase ordering

Existingvalidation/errorpriority +expiry/pausepreflight; revealpendingpickups;
advance PREEXISTINGbombmotion fromimmutablephaseworld/livingactorfootprints;
inputs/voluntarymovement/Contact+kicks; resolvekickrequests(nohopnow);
sweptoldfiredamage; manual/Diarrhea; ONEimmutableexplosionplan overpostmotion+
postplacementworld; finaldamage/pickups/deferredTeleporter/result. Newplacement
cannotretroactivelykick. Resultfreezesphysicalworldincludingmotion.

Beforehop AFTERStopwatchdeadlineupdates, bombalreadyinactivefire OR TimeddueT
doesNOTmove: leaveat sourceforlateroneexplosionplan. Pauseddeadline mayadvance
soformerlyduecanmove. Hop INTOactivefire commitsnewcell andsameplanignitesthere;
adjacenthop sweepssource+destination so cannotjumpfire. No extraexplosionpass/
fireextension. ManualRemote triggerslaterthanmotion: mayhopthenexplodeatnewcell
sameupdate; explicitobservableorderingtest. StopwatchpausesTimedfuses, notmotion;
frozenplayers remainblockers/cannotinitiatekick. Motiondoesnotfreecapacity though
vacatedcellmayallowlaterlegalplacement ifcapacityalreadyavailable.

## Atomic swept cell collision

All due/nonburning/nonexpiredmovingbombs proposeadjacenttarget fromSAMEimmutable
startworld. Blockbounds/hardwall/softblock regardlessofWallPass; ANYotherbomb's
startcell(includingContact/moving-awaybomb); positive-area sweptFULL-CELL square
intersectionwithANYlivingstartactorfootprint. Swept hull isunionof2adjacentcells;
tangencyalone okay. Aliveactoroverlappingsource blocksuntilwholefootprintclears,
includingowner/kicker/graced/frozen/cloaked/blinded; deadactors don'tblock.

Resolveallproposals beforecellschange: sameemptytargetclaims ALLstop; exchange/
followingstartcellblocked; novacancychain/push/IDorderdependency. Commitdisjoint
acceptedhops atomically. Priorities sourcefire/expiryhold; static/bombs/actors;
same-targetconflicts; remaindercommit. Obstructiondestroyedlater oractorkilledlater
stillblocked; no retrospective retry.

BombremainsFULLYsolid CURRENTcellbetweenthese10tickhops. Actormovementusesposthop
world: mayentervacatedsource, cannotenterdestination. Anactorenteringfuturehop
sweepduringwaiting becomesblockeronnexthop. Nointangibletravel/tunneling. Clear
actorbomb_graceforALLmovedIDs, no landinggrant; stationarygraceprunesnormally.
Pickupsdon'tblock/collect. Contactminesblockmotionbutdon'ttrigger/shove/chain/
destroyoncollision. No newcollisiondamageclass.

## Explosion, rollback, interaction

Planner seesfinalcells aftermotion/placement, immutablethroughrays/chains/Power.
Detonationusescurrentcell/unchangedcaptures; historicalplacementkeepsoriginalcell
andneedn'tmatchsurvivingbombcell. PreserveStandard/Piercing/Powerobstructions,
Remoteuniqueness/priority, Contactacceptedpriority/silentdestruction, exactBHD+
pausedticks, no extracapacityrelease/place/detonationevent.

Wholelatepickupexpiryoverflowrollback restoresmotion/provenance/countdowns/cells/
gracepruning/newkicks/actorinputs+movement/pause/explosions/pickups/allocator/
damage/events/audiences/teleport. Validatemalformedmotionhistorybeforemutation
withoutreorderingoldinput/tick/trigger/blast/ownererrors; Contactcannotcarrymotion.

Teleporter usescurrentpostmotionbombcells/owngrace, grantsnone; arrivingactorcan
blockfuturehop. WallPass->Kick usesexactreplacementsoftgrace; actorcanexitblock,
bombstillcannotpasssoftblock. ShieldHeartgrace/CloakInvisible don'tremovephysical
blockers. Disconnect/deathaddnofuse/stop; physicalresultsfreezeeverything.

## Trusted provenance, archives, rendering

ExistingpublicBombView.cell suffices, nonewisSliding/direction/countdown/kicker
wirefields/events. Carryprivatemotion+hasBeenKickedthroughtrustedliveANDhistorical
capture (includingevent-onlydetonation), likeStopwatchprivateprovenance. Validate
countdown/direction/trigger/Kickenabledhistory/owner/count/uniquecell/fuse/pause
beforefiltering, evenhiddenfromboth. Archivedv9/v8/earlier rejectnewequipment/
pickup/activeORstoppedkickhistory, includingevent-onlyfacts, neverdowngradebecause
stoppedwirelooksordinary. Preservepairedatomicpublication/admission.

No movementevent/audio. Cloak/Invisiblebomb placement/detonationaudiences stay
atemission; publicbombcellmayinferhiddenkickerbutneverattachidentity/origincue.
CurrentArenaSceneupdateBombs skipsretainedIDs: UPDATEretainedrootpositiononcell
change, sameID/drawing/art, no duplicateplacement/source-shadow/trail. Immediate
gridsteps, no diagonalinterpolation/prediction; actorinterpolationunchanged.
PausedTimedfusecuepausedwhilecellmoves; Remoteuntimed, Contactnevermoves.
Invisible removesroots/history/revealsCURRENTcellsquietly. DistinctKickart/HUD+
ENPLhelp explainwalkingkick/cadence/blockers/unchangedownerfuse/minesexcluded.
No binding. Preserve320px/reducedmotion/teleportsnap/stale/lifecycle/disposal;
no audiofromcellchange/stop/resume.

## Catalog, test cost, ownership

bit0=1; ALL3145720->3145721; available18->19; masks262144->524288.
Appendweight15 AFTERWallPass, pool380->395 interval[380,395), sole115. Canonical
order kick,wallPass,heart,remoteBomb,piercingBomb,powerBomb,landmine,shield,
stopwatch,invisibilityCloak,goldenFlame,shortFuse,diarrhea,slowFoot,shortFlame,
longFuse,reverseControls,invisibleBombs,teleporter. Punch1/Bracelet2/Oil20inactive.
PreserveALLoldmaskmap/RNG/layers; generationappend != wireorder.

ApplyWallPassmeasuredtestpolicy at524288: exhaustivemaskalgebra/conversion/pool/
exactfullmessageassertions inboundeddeterministicranges(e.g1024), provecomplete
disjointcoverage/rawmaskfailures/freshstate/AJVreuse, no giantregistrationarray.
Preservealloldinvalidtests/floors/fixtures/EXPENSIVEoldroom-start-replays. Measure
duration/memorybeforebatching/sharding. Ifnewreplayenumerationimpractical, propose
separatelyrevieweddesign/evidence; never silentlysample/deleteoldchecks/raisetimeouts.
CIshardsneedcompleteexplicitrangeaggregate; localunpartitionedcommandcoversall.
Don't crossproductlongKickrouteswithallmasks.

1. Domain: bombprivateobs/motion/collisionresult, focusedkick+motionmodules,
   step/validation/equipment/generation/tests/route.
2. Protocol/runtime: v10trustedcapture/binding/archive/eventonlyhistory,
   publication/admission/resume/replay.
3. Web: semantics/settings/HUD/retainedbombupdates/art/audio/ENPL.
4. Coordinator: measuredtestorganization/fixture/config/Compose/scenario/CI/
   version/docs. Serializesharedfiles; independentdomainandprotocol/browserreviews.

Required: allforwarddirections/adjacency/zero/lane/ReverseSlowfreeze/owngrace/
nearestambiguity/Contactexclude; actorinputIDorder/prioritysamebomb/distinct/
movingnoredirect/stoppedthisupdate; exactT+1/11/21/sweep/tangent+1unit/source
overlap/walls/blocks/allbombs/alive/dead/hidden/frozen; conflicts/exchange/follower/
permutations/currentcellsolidity/futureactorblock/notunneling; due/sourcefire/
targetfire/pause/Remotehoptrigger; immutablecaptures/capacity/chains/PowerPierce
Contact; noretroplacement/earlycapacity/collection/collisiondamage; WallPass/
Teleport; fullrollback/replaystop/resume/death/result/reset; hiddeninvalidlive+
eventhistory/archive/pairencodefailure; sameroot/currentpaint/quietaudio/ENPL/
teardown; exhaustivemasks. Retain16bombs195flames96events<64KiBframes<=64byte
records<4MiBjournal400line modules.

## Proposed native route — unexecuted

First ENTIREpurecoreseed31soleKick/legal300fuse ordinaryinputproof, pickup3,1,
soft3,2+4,1/corridor/survival/timing pinned. ENdesktop/PL320readybeforeplay.
Reveal3,1 bybomb2,1, escape1,1->1,2/wait/collectKick. PREPAREescape corridor with
ordinarybomb3,1: exit2,1->1,1->1,2 andwait; assertsoft3,2 AND4,1destroyed.
PlantDEMO bomb3,1, fullyexitEast4,1, holdWestintoblockingface/releaseaftermotion
finally. BothstreamsSAMEID ordered3,1->2,1->1,1stopbeforeborder; coreprovesexact
cadence(publicationsquantize). Noextraaction/BombPlacedforhops. Escapecleared3,2
through3,1 BEFOREscreenshots. Cheapcompositorcropsfinalbomb+oldunobscuredcell
absence/ENPLHUD. NaturaldetonationatplacedAt+300at1,1 unchangedowner/range/trigger/
blast/bothalive/no extraaudio. Preparatoryblastessential: initialtopcorridoralone
hasnosafeescapefromstopped1,1bomb. Complexcollision/Remote/Stopwatch remainfocused
core/runtimeproof, notmultipleinjectednativefixtures. Realkeys/UI/read-onlyobs,
releasefinally/no injection/seedsearch/retry/weakenedbudgets.

Rununchangedall/coverage/security/k8s/bothprofiles/contracts/shuffledweb/E2E/
audits/floors/lineceilings; independentreviews then freshcumulativesameimageC+K
includingaudio/finalCI/evidence. Correctedpreflight+pinnedchecker+pre/postdeps.
Rollbackpairedpriorimage/reload/newrooms; oldmask/journalrejects; no DB migration.
