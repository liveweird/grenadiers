# Retro pixel-art courtyard

Production presentation now uses the [3D miniature arena](arena-3d.md), with the same canonical
billboard artwork and additional shared surface textures. The Pixi implementation remains an
isolated reference for the art workflow.

[Actual application captures](../screenshots/pixel-art.md) illustrate the current presentation.
Dated capture and release evidence is preserved in
[game acceptance history](../history/game-acceptance.md#pixel-art-and-screenshot-provenance).

The PixiJS arena renders the canonical [artist sheet](../../assets/art/README.md). The current
Napoleonic atlas uses weathered stone, mossy masonry, layered brickwork and brass highlights;
bright flames and orange-and-white/blue uniforms keep hazards and competitors distinguishable.

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

Actor pose visibility is assigned idempotently. Snapshot updates retain the current animation
frame until rendering selects its final pose, avoiding intermediate visibility invalidations.

Player one wears an orange-and-white uniform with a plumed shako; player two wears blue with a
cockaded bicorne. The procedural sources in `web/src/game/arena/terrainArt.ts`, `actorArt.ts`,
`entityArt.ts`, `palette.ts` and `pixel.ts` produce the template and parity reference, not a
runtime fallback. All patterns were authored for this repository; no franchise sprites, external
texture packs, downloaded fonts or new rendering dependency are included.

Geometry/resource-bound, retention, removal, interpolation, reduced-motion and StrictMode tests
accompany the artwork.

Golden Flame has a bright flame pickup and separate crest above the actor, coexisting with Shield
and Heart-grace indicators. Reverse Controls has a directional pickup. Diarrhea has a bomb-drop
pickup and actor cue. Slow Foot has a boot pickup and grouped-effect presentation. Reduced motion
keeps their persistent cues static.

Piercing Bomb and Power Bomb have distinct stored-weapon and armed-bomb cues. Remote Bomb uses an
antenna-topped planted bomb plus a localized armed state. Landmine uses a visible pressure plate.
These visuals do not replace authoritative blast, trigger, contact, capacity or consumption rules.

Invisibility Cloak uses a hooded pickup glyph. Recipient projection removes the hidden opponent's
complete actor root, including shadow, motion record and attached cues; expiry paints the actor at
the current authoritative position without a stale interpolation trail.

Stopwatch uses a clock-face pickup and holder marker. A frozen visible actor receives static corner
brackets, while a paused Timed bomb replaces its burning spark with a pause mark. Reduced motion
keeps those cues visible without pulsing; Remote and Contact bombs retain their untimed art.
