# Bracelet / Shove — v0.3.46 implementation package

Adopted coordinator summary of independent architect; depends Punch45 and all
earlier packages. Plan only; native route/timing must be proved in implementation.
Keep gameplay-v10/rules5/generator2/journal2. Oil remains disabled.

## New moving contact and exact pair geometry

Persistent replaceable Bracelet, no manual command. New voluntary FORWARD contact
shoves other living actor <=1tile, clamped, success30physicaltick cooldown plus
separation before rearm. Exactly2actors; no speculative multi-seat/mob graphs.
E with Bracelet is admitted no-op only when settings enablePunch; otherwise
Equipment stays unsupported perPunch45.

Holder alive/enabled/equipped/unfrozen/cooldownelapsed, positive ACTUAL forward
travel entering rival through leading face. Adjacency/stationary overlap/packet/
zero budget/correction-only/rival entering stationary holder do not initiate.
Reverse uses effective travel; gear collected later cannot retroactivate.

Use actual2voluntarysegments and ORIGINAL movement budget. Segmentlength d takes
d/budget fraction, then stationary remainder if blocked/completed; zero budget
is stationary interval. Don't normalize shortened travel to whole update. Union
of both actors' segment/stationary boundaries yields <=5 intervals. Solve relative
square-footprint positive overlap with exact bounded integer/rational comparisons,
no float epsilon/diagonalAABB/unordered swept rectangles that mistake paths at
different times for contact. Separations strictly<640 on both axes (tangent not
overlap). Prove arithmetic bounds; forward-face entry requires relative closing,
not parallel/rival-only side contact or lane-correction-only motion.

Derive overlap episodes from start positions+paths: initial/continued overlap
creates no entry. Cooldown expiry/acquisition/replacement/resume cannot rearm
while overlapped. Positive overlap must end before new entry. Zero-distance
blocked attempt still consumes episode until separation. Teleport/forced arrival
creates no voluntary candidate. Prefer geometry instead of independent persistent
latch; any latch needs equivalence proof plus State/replay inclusion.

<=1candidate/actor, competing requests sameupdate resolve RoundAssignmentpriority.
ONE winner globally for currentpair; blockedwinner0displacement has no loser
fallback/recursive claim.

## Forced segment, collision and phases

After BOTH voluntary paths and Kickclaims, beforeoldfire, displace target from
POSTvoluntary position cardinally <=1024. Clamp victim's footprint by bounds/
hard/soft unless VICTIMWallPass/ownsoftgrace; TimedRemote unless VICTIMowngrace;
Contact passable/swepthazard; pickups nonblocking. Protection/Cloak/Invisible/
freeze don't remove collision. Pusher's capability/grace never authorizes victim.
Onlyotheractorpusher blocks if initially nonoverlapping; if alreadyoverlapped,
allow leaving in this ONEsegment without persistentplayergrace/reentry rights.

Prunevictimexistingsoft+bombgrace afteractualsegment, grantnone. Positivedistance
setsinitiatorlastSuccessTick, clearsvictimresidual+turnintent, keepsheading/rawheld
recency/identity/inventory/effects/ownership/capacity. Zero changesnone of those,
but episode consumed. Frozenvictim canmove/frozenholder can'tinitiate; later
pusherdeath doesn'tundo accepted shove.

KeepvoluntaryMovementTrace hard2segments. Add SEPARATE optionalforcedcardinal
segment/seat with <=1populated globally, included in private report/replay equality.
No wire fields. Fullphases: validation/pausepreflight; expiry/reveal; Kickmotion;
inputs/bothvoluntarypaths+Contact/Kick/Bracelet candidates; Kickclaims thenShove;
forcedmineentries; ONEoldfirepass over each voluntary+forced path; manualwaves/
Diarrhea fromfinalpostshoveposition; ONEimmutableexplosionplan unionminecontacts;
finalfire; ONEpickupcollection/deferredTeleport/result. No recursiveShove/Kick/
extraactions/turns/budget/extra damageclass. Admitted victim manualactsfromnew
position/heading ifalive/unfrozen, once.

## Hazards, pickup, cooldown, rollback

Oldfirechecks every actualsegment and stationary start asappropriate, not just
endpoint/boundingrect. SharedShield-beforeHeart damage gateway with <=1Shield
prevention/actor/update and existing Heartgrace. Carrycorrectminegrace across
voluntary THENforcedsegments, pruneonfullExit beforelaterentry. Preexistingmines
only; acceptedcontact persists afteroldfiredeath andwinsblastdestruction inone
laterplan. Newflames damage finalfootprint, neverretroactivelysweepoldpath.

Collect only eligible FINALcenterpickup afterdamage/explosions; crossingpickup
doesn'tcollect, newBraceletcan'trecurse/undo damage. EndpointTeleporter mayperform
normaldeferredsafeswap with no repeatpickup/action/movement; retain owngrace and
lastTeleportTick semantics.

SuccessT blocksT..T+29, allowsT+30 withnewentry. Private last_successful_shove_tick
checkedelapsed, physicalroundclock validation, persistsreplacement/resume/death/
results ashistory, resetsnewround. SeparatePunchcooldown, switchingdoesn'treset.
Stopwatchdoesn'tpausecooldown but blocksvoluntaryinitiation.

Validateprivatehistoryenabled/clock andforcedtracecount/cardinality/length/endpoints
matchingcommittedphasevictimpos. Preservephasefacts iflaterTeleporter changesfinal
endpoint. No publicpusher/cause marker. Latepickupoverflowrestoresvoluntarymoves/
requests/forceddisplacement/residualturn/bothcooldowns/grace/Contact/KickPunch/
pause/damage/pickups/teleport/events/allocator. Preserveerrorpriority; fullState+
reportsreplaythroughblocked/success/freeze/resume/replacement/death/results/reset.

## Protocol, rendering and privacy

Existingv10position/inventory/effectrecords suffice. Shove is actualswepttravel,
so keepnormalactorinterpolation; lastTeleportTick isONLYteleport. No extraevent/
extrapolatedpath/attackertrail/sourcecue/delayeddamage/specialcooldownreadiness.
Visiblevictimcanmovefromhiddeninitiator withoutleaking initiatoridentity/position;
hiddenvictim staysEXACThiddenunion. Worldchangesremaininferenceclues.

HUD/helpstatictruthfulrules: newmovingcontact, <=1tile,0.5s+separation. Private
cooldownnotinferredfrommovement. No shovesound/event; existingpickup/damage/elim
audiences unchanged, no snapshot-diffaudio. Hiddenroot/cue/shadow/motionremoved;
revealcurrentquiet. ENPL320/reducedmotion/teardown retainPunchprefsmigration.

Completeprivatefacts/eventsvalidate BEFOREfilter. Archivedv9/v8/earlierreject
Braceletsettings/gear/pickup/shovehistory evenifhidden/no longerheld. Bothframes
preparebeforecommit; secondfailurekeepsrevision/reports/no partialadmission.

## Catalog, cost and ownership

Braceletbit2=4; ALL3145723->3145727;20->21items;1048576->2097152 masks.
Append15 AFTERPunch,410->425 interval[410,425),sole115. Canonicalstarts
kick,punch,bracelet,wallPass,heart,...; Oilbit20disabled/no barrelclaim.
PreservealloldRNG/maps/layers/weights. Measureactualgatecost beforebatching;
1024rangeswouldbe2048Vitesttests retainingEXACTfullmessageassertions/freshstate/
rawmaskdiag/fullunionproof. PreserveALLoldinvalid/archive/floor and expensive
room/replay assertions. Newmask expensive-replaydesignneedsseparatecost-based
review; no sampling/silentreplacement/timeoutraise.

Ownership domain pairgeometry/timing/actorhistory/forcedtrace/collision/phase/
catalog/gen/tests first; protocol/runtime trustedfacts/archives/replay/atomicity;
webgearHUD/art/privacy/audio; coordinatorcost/native/CI/version/docs. ReuseKick
forwardsegmentidentity and ORIGINALbudget. Sharedfilesserialized,400line limit.
Independentgeometry/hazard andprotocol/runtime/browserreviews.

Acceptance: allaxes/tangent+1unit/different-timings/correction/2segments/blocked
stationaryremainder/parallelclosing/opposition/zero/stationaryholder; initial+
continuedoverlap/cooldown/replacement/failed0/separate-reenter/bothpriorities/no
fallback; full/partialclamp/victimcapability+grace/pusheroverlapexit/frozenhidden/
notunneling/recursiveactions; sweptfire/protection/voluntary+forcedmines/grace/
contactprecedence/newmineexclude/postshoveactions/oneplan; endpointpickup/damage
beforegear/deferredsafeteleport; exactcooldown/nearhorizon/rollback/replay/lifecycle/
oldmaskparity/hiddeninvalid; currentpaint/hiddeninitiatorandvictim/nocue/ENPL.
Preserve16bombs195flames96events<64KiBpairedframes<=64byterecords<4MiBjournal.

## Proposed two-player native route, unexecuted

Mainrisk is legitimate2playerpreparation within existingitemscenariobudget.
FIRST proveENTIREseed31soleBracelet ordinaryinputroute fromREALspawns including
ALLplacements/frontiers/escapes/firewaits/baseupgrades/staging/survival/budget.
P1reveal/collect3,1via2,1bomb+1,2escape, clearsTOPperimeterEast. InparallelP2
clearsRIGHTperimeterNorthfrom13,11, using12,11initialcornerescape. Pin EACHreal
frontierplacement+legalescape, accountnewrange/speed. BringP2 to11,1 via cleared
topright, P1to10,1, bothalive/fireclear/separated. P2releasesmovement; P1walksEast
untilcontactshovesP2exactly1tileto12,1, releasesfinally. Bothstreams samevictim
displacement/no victimvoluntarycommand/no equipmentaction/no inventedshoveevent/
sound. Cheapcompositorcrops authoritativepaint+ENPLgearfeedback; exactgeometry/
cooldown/clamp coretests. ENdesktop/PL320readybeforeplay.

Coreproofmustdemonstratemargin. Ifperimeterroutecostly, reviseON SAMEseed before
browser, no seedsearch/teleport/setupinjection/retry/timeoutraise. Actualkeys/UI/
readonlyobs/releasefinally. ComplexforcedminesfireStopwatch staycore/runtime.
Unchangedall/coverage/security/k8s gates +bothprofiles/shuffledweb/measurecost,
independentreviews/fresh cumulativesameimageC+K/audio/finalCI; pinnedchecker+
pre/postexternaldependencyhashes. FocusedPR46 explainscontact/clamp/separation/
cooldown; pairedrollback/reload/newrooms/oldmaskreject; no DB migration.
