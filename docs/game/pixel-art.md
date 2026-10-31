# Modern retro pixel artwork

The [3D miniature arena](arena-3d.md) combines textured geometry with detailed pixel-art
billboards. The canonical [artist sheet](../../assets/art/README.md) owns both sprite drawings
and the named surface maps. Lighting and restrained colour grading/static dithering give the
courtyard depth while keeping players, pickups and blast cells readable.

## Source artwork and animation

Four uniforms distinguish the seats: orange/white with a plumed shako, blue with a cockaded
bicorne, forest green/cream with silver, and plum purple/cream with gold. Each player has four
poses per cardinal direction, with the walking cycle `0 → 2 → 1 → 3`. All poses share a foot
anchor. Timed bombs have three shell poses plus separate fuse-spark and pause drawings;
remote bombs and landmines retain their untimed art.

Nearest sampling and pixelated CSS preserve hard edges. Logical 48×48 artwork is mapped onto
world tiles; the orthographic camera fits each supported board to the responsive viewport.
The canvas uses device-pixel ratio capped at two. Reduced motion selects static decorative
frames and immediate positions without changing authoritative hazards or effects.

Terrain variation uses public cell coordinates. Artwork may extend beyond a collider without
changing the server's fixed footprint or damage rules. Snapshot interpolation stops at received
targets and never predicts movement, fuse expiry or damage.

## State cues and recipient visibility

Golden Flame has a flame pickup and crest, coexisting with Shield and Heart-grace indicators.
Reverse Controls uses directional arrows; Diarrhea a bomb-drop symbol; Slow Foot a boot and
stack markers. Permanent gear and stored weapons retain distinct pickup and attached cues.
Remote Bomb has an antenna-topped shell and localized armed state; Landmine uses a pressure plate.
These visuals do not replace blast, trigger, contact, capacity or consumption rules.

Cloak removes a hidden opponent's complete actor root, shadow, motion history and attached cues.
Reappearance paints the received position without a trail across concealed space. Invisible
Bombs removes bomb drawings and their attached cues for the afflicted recipient.

Stopwatch uses a clock-face pickup and holder marker. Frozen actors receive static corner
brackets; paused Timed bombs replace the burning spark with a pause mark. Persistent cues stay
visible under reduced motion. Suspension hides the canvas and clears presentation history;
a new baseline is painted before revealing it. Final unmount releases owned WebGL resources.

## Artist and reference workflow

Use the [artist handoff](../../assets/art/README.md) for guides, palette/alpha/anchor rules and
validated extraction. Procedural terrain, actor, entity and material factories produce the
reference template; the isolated Pixi workbench checks art parity and does not provide a
production fallback. [Earlier 2D captures](../screenshots/pixel-art.md) remain useful for artwork
comparison; current 3D screenshots are attached to the [release](../releases.md).
