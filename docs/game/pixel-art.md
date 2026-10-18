# Retro pixel-art courtyard

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

Sources: `web/src/game/arena/terrainArt.ts`, `actorArt.ts`, `entityArt.ts`, `palette.ts` and
`pixel.ts`. All patterns were authored for this repository; no franchise sprites, external
texture packs, downloaded fonts or new rendering dependency are included.

Geometry/resource-bound, retention, removal, interpolation, reduced-motion and StrictMode
tests accompany the artwork. Independent code and visual review is clear. The same image
passes 24 scenarios each through Compose and Kubernetes, plus a separate native capture case.
This first visual slice covers terrain, actors, bombs, flames and the existing three pickups;
new item icons and richer death/destruction/collection transitions remain follow-up artwork.

Actor pose visibility is assigned idempotently. Snapshot updates retain the current animation
frame until rendering selects its final pose, avoiding intermediate visibility invalidations.
