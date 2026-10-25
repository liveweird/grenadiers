# Retro pixel-art courtyard

The 21 October 2026 16:47 UTC native acceptance checkpoint's Cloak captures come from corrected
source `1c6724f` and image `960976c9`:
[English desktop](../screenshots/invisibility-cloak-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisibility-cloak-polish-320.png). Root reviewed the
pre-collection pickup and readable HUD/settings; normal Polish vertical scrolling and the expected
unfocused-arena prompt remain. Native assertions prove privacy/expiry for that named source;
these stills do not establish behavior. Kubernetes captures are retained without visual review.
Fresh native acceptance was independently audited and sealed; release CI and publication remained
pending at that checkpoint. See [release notes](https://github.com/liveweird/grenadiers/releases)
for current release status.

These accepted v0.3.38 captures are integrated into the published v0.3.39 gameplay-v9
foundation without changing artwork or runtime code. At the 21 October 2026 20:56:41 UTC v0.3.40
native checkpoint, corrected source `fa25467` passed fresh 52 Compose plus 52 Kubernetes cases.
Root reviewed the Compose [Stopwatch English desktop](../screenshots/stopwatch-desktop.png) and
[Polish 320px reduced-motion](../screenshots/stopwatch-mobile-pl.png) captures: both show the
pre-collection clock pickups and readable localized HUD/settings. They do not prove freeze or
fuse behavior; native assertions do. Independent raw audit is clear and 2,434 payloads are durably sealed; release gates remained pending
at that checkpoint, and published/local v0.3.39 remained the release baseline.

The latest Landmine desktop and Polish 320px captures were refreshed from corrected source `420d8df` and tested image1736. The pressure plate/consumed charge/no-flame state and normal HUD layout were visually reviewed; these stills precede contact and do not replace behavioral assertions. Kubernetes captures were retained without visual review.

Implemented in v0.3.18. [Actual application captures](../screenshots/pixel-art.md) and the
[verification record](../verification.md) cover both deployment paths.

The PixiJS arena uses original code-native pixel artwork. Weathered stone tiles, mossy
masonry, layered brickwork and small brass highlights establish a courtyard palette; bright
warm flames and gold/blue uniforms keep hazards and competitors distinguishable.

Terrain variation comes only from public cell coordinates, never private generation seeds.
Players have front, back and side poses, two walking frames, and different helmet marks.
Bombs have stepped shell highlights and fuse sparks; pickups use distinct beveled symbols.
Connected flame cells retain continuous rays. Artwork can extend beyond a collider without
changing the server's fixed footprint or damage rules.

The logical surface stays 720×624 with the existing 15:13 aspect ratio. CSS pixelated scaling
fits the available space; Pixi uses no antialiasing and caps resolution at device-pixel ratio
two. Resizing no longer rebuilds scene geometry. Interpolation still stops at authoritative
targets and never predicts movement or damage.

Terrain is retained until its public layout changes. Each actor owns eight fixed pose drawings
and one shadow; bombs own three drawings, flames two frames, and blocks/pickups one drawing
each. Visibility toggles animate existing objects instead of allocating frames. Unchanged
authoritative targets stop walking; terminal state and reduced motion stop decorative frames.
Removed entities and round resets destroy owned child graphics contexts. The arena keeps one
renderer during mounted room/reconnect transitions, suspending its ticker and clearing hidden
artwork while waiting. A new baseline is painted before the canvas becomes visible again.
Actual component unmount still releases the renderer and WebGL resources.

Since v0.3.49 gameplay draws these from the [artist sheet](../../assets/art/README.md); the
current Napoleonic sheet gives player one an orange-and-white uniform with a plumed shako and
player two a blue uniform with a cockaded bicorne. The procedural sources below still produce the template and parity reference.

Sources: `web/src/game/arena/terrainArt.ts`, `actorArt.ts`, `entityArt.ts`, `palette.ts` and
`pixel.ts`. All patterns were authored for this repository; no franchise sprites, external
texture packs, downloaded fonts or new rendering dependency are included.

Geometry/resource-bound, retention, removal, interpolation, reduced-motion and StrictMode
tests accompany the artwork. Independent code and visual review is clear. The same image
passes 24 scenarios each through Compose and Kubernetes, plus a separate native capture case.
This first visual slice covers terrain, actors, bombs, flames and the existing three pickups;
new item icons and richer death/destruction/collection transitions remain follow-up artwork.

Golden Flame, introduced in v0.3.27, adds a bright pixel flame pickup and a separate crest
above the actor, coexisting with Shield and Heart-grace indicators. Its timer comes from the
authoritative baseline; reduced motion keeps the crest static. See its [accepted desktop and
Polish mobile captures](golden-flame.md) and the source-specific verification record.

Actor pose visibility is assigned idempotently. Snapshot updates retain the current animation
frame until rendering selects its final pose, avoiding intermediate visibility invalidations.

Reverse Controls adds its own directional pixel pickup and localized rule/HUD presentation.
See the [accepted desktop and Polish mobile captures](reverse-controls.md); exact inversion
and expiry remain authoritative simulation behavior.

Diarrhea adds a distinct bomb-drop pickup and actor cue; reduced motion keeps the cue static.
Its [desktop and Polish mobile captures](diarrhea.md) show the exposed pickup and localized
eighth rule switch before collection. Active-effect presentation also has renderer unit coverage.

The refreshed Diarrhea captures come from the corrected-readiness acceptance recorded in
[verification](../verification.md). Both show the uncollected pickup; the active curse and
automatic placement are established by renderer/domain and paired-browser assertions.

Slow Foot adds a distinct boot pickup and localized ninth rule switch. Its [English desktop
and Polish 320px captures](slow-foot.md) show the exposed pickup before collection, with no
active effect timer. Grouped effect presentation has renderer coverage; speed, independent
stack deadlines and exact restoration remain authoritative simulation behavior.

Piercing Bomb adds its own armed-bomb cue and localized tenth rule switch. Its [English desktop
and Polish 320px captures](piercing-bomb.md) show the armed weapon before detonation. The reviewed
captures have no overlap. The compact English four-column HUD wraps “Piercing bomb” inside the
word, a nonblocking cosmetic preview limitation; no perfect compact-layout claim is made. Blast
continuation, one-charge consumption and the following Standard bomb remain authoritative
simulation and paired-browser behavior.

Power Bomb adds its own armed-bomb cue and localized eleventh rule switch. Its [English desktop
and Polish 320px captures](power-bomb.md) show the armed weapon before detonation, without active
bombs or flames. “Power bomb” wraps normally between words in English; the Polish narrow view
uses vertical scrolling and shows the unfocused-arena prompt. The reviewed captures have no
unintended overlap. Manhattan blast geometry, one-charge consumption and the following Standard
bomb remain authoritative simulation and paired-browser behavior.

Remote Bomb adds an antenna-topped planted-bomb cue, localized armed state and twelfth rule
switch. Its [English desktop and Polish 320px captures](remote-bomb.md) show the live Remote at
(3,1), its stored charge consumed, owner availability at zero, both actors alive and no flames.
The English Gear label fits normally; the Polish narrow view uses vertical scrolling and shows
the unfocused-arena prompt. The reviewed Compose captures have no observed unintended overlap;
Kubernetes captures were not visually reviewed. Native assertions establish the 183-tick wait,
exact-ID activation without another placement and the following fresh Standard bomb. The native
evidence audit is clear and its final manifest is sealed.


Landmine adds a visible pressure plate and localized thirteenth rule switch. Fresh reviewed
Compose [English desktop](../screenshots/landmine-desktop.png) and
[Polish 320px](../screenshots/landmine-polish-320.png) captures show the mine at (3,1), its owner
at (2,1), charge consumed and no flames. HUD labels fit; Polish uses normal vertical scrolling
and the intentional unfocused-arena prompt. No unintended overlap was observed. Kubernetes
captures were retained but not visually reviewed. Paired native assertions establish exit grace,
persistence past 183 ticks and exact contact detonation; the following fresh Timed Standard bomb
is a separate core regression, not an additional native assertion.


Invisibility Cloak adds a distinct hooded pickup glyph. Recipient projection removes an opponent's
complete actor root, including shadow, motion record and attached cues, while hidden; expiry paints
the actor at the current authoritative position without a stale interpolation trail.


Fresh reviewed Compose [English desktop](../screenshots/invisibility-cloak-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisibility-cloak-polish-320.png) captures show the
Cloak pickup before collection and readable localized HUD/settings. Root actually viewed only
these two Compose stills; Polish uses normal vertical scrolling and the unfocused-arena prompt is
expected. These stills do not prove actor privacy: the paired native assertions establish owner
visibility, exact peer-hidden actor shape, compositor removal, reconnect and current-position
reveal at the physical deadline. Kubernetes captures are retained but were not visually reviewed.

Stopwatch adds a clock-face pickup and holder marker. A frozen visible actor receives static corner
brackets, while a paused Timed bomb replaces its burning spark with a pause mark. Reduced motion
keeps those cues visible without pulsing; Remote and Contact bombs retain their untimed art.
