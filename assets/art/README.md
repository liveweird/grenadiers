# Grenadiers arena art handoff

This directory contains the artist-editable arena source: `arena-source.png` and its
paired `arena-source.json` manifest. The extraction, validation and
preview workflow shares this artwork with the production 3D renderer; see the
[rendering guide](../../docs/game/art-atlas.md) for the build/runtime ownership and checks.

## Prerequisites

- Node.js 24.21.0, matching the repository toolchain.
- Installed workspace dependencies: `npm --prefix web ci` and `npm --prefix e2e ci`.
- The repository's installed Playwright Chromium (`npm --prefix e2e run install:browsers` when it
  is not already present).

Run commands from the repository root. The CLI builds the separate renderer-diagnostics target,
starts an ephemeral loopback server, and uses Chromium for Pixi extraction and browser PNG IO.

## Create the artist template

```sh
npm --prefix e2e run art:template -- --out target/art-template
```

The template command refuses to overwrite an existing output directory. It writes exactly five
files:

- `arena-source.png` — transparent editable artwork only;
- `arena-source.json` — stable frame IDs, rectangles, pivots, anchors and animation metadata;
- `arena-guides.svg` — scalable slot boundaries and complete stable-ID labels;
- `arena-guides.png` — raster copy of the separate guide/label layer;
- `arena-overview.png` — labelled visual inventory on a checkerboard background.

The 1120×2240 authoring sheet has 200 physical cells: 196 assigned entries (192 baseline
drawings and four blank reserved entries), plus four unassigned transparent cells. Cells are 112×112
pixels in ten columns. Logical arena tiles remain 48×48 pixels. Reserved and unassigned cells must
stay transparent. Do not move or rename assigned entries; new catalogue IDs must be added in code
before a source revision can use them.

The reviewed `arena-source.png` and `arena-source.json` in this directory are the canonical artist
inputs. Generate a template under `target/` to compare or start a deliberate replacement; the
command never overwrites this adopted pair. Guide and overview files remain reproducible review
outputs rather than canonical inputs.

## Edit and export

Edit `arena-source.png` in a pixel-art editor without resizing the sheet or frames. The sheet
includes all four uniform appearances, mapped to the configured player seats. Then export a
validated runtime pack:

```sh
npm --prefix e2e run art:export -- \
  --source assets/art/arena-source.png \
  --manifest assets/art/arena-source.json \
  --out target/art-atlas
```

Add `--slices` to emit optional named files under `target/art-atlas/slices/`. Runtime output is
generated and ignored rather than an artist source. Its main files are `arena-atlas.png`,
`arena-atlas.json`, `arena-overview.png` and `arena-preview.png`. The runtime texture contains the
192 baseline frames, unrotated and untrimmed, with two-pixel extrusion and two-pixel transparent
padding. Labels and guides never enter the runtime texture.

Both commands refuse an existing output directory, so an artist source or earlier review cannot
be silently replaced. Choose a new output path or deliberately remove/archive the generated
target after review.

## Pixel and anchor rules

- Keep pixels on integer coordinates with nearest-neighbour sampling and no antialiasing.
- Keep each player appearance within 48 opaque RGB colors across its sixteen directional frames;
  use deliberate shading ramps rather than diffuse generated color noise.
- Source alpha is binary: every pixel is either fully transparent (`0`) or fully opaque (`255`).
  Do not bake translucent shadows into the PNG. Shadow opacity remains manifest metadata (actor
  shadow `0.3`, bomb shadow `0.28`) and is applied exactly once by the region importer.
- Paint only inside each baseline frame rectangle. Slot gutters and reserved frames must remain
  transparent.
- A frame pivot preserves its drawing origin and must equal `(-localBounds.x,
  -localBounds.y)`. All directional and death actor/mob frames retain the shared `(40, 65)` foot
  anchor.
- Keep IDs path-safe and stable. The exporter rejects missing, duplicate, unexpected or path-like
  IDs, changed required animations, invalid dimensions and texture-budget overflow.
- Flame frames must reach all four edges so adjacent rays join. Opposing RGBA edge pixels must match,
  with a nonempty opaque connector. The adopted six poses use a narrow four-pixel connector;
  the procedural template retains its historical artwork.

Use the standalone `art-atlas.html` renderer diagnostic to load a source PNG and manifest, inspect
all labelled drawings, and see the representative arena rebuilt from actual Pixi atlas regions.
The diagnostic shows pickups, actor/cue compositions, bomb shells, spark, pause mark, Oil states
and adjacent flames. It is not linked from the gameplay or product flow.

## Animation editing

Each player/direction has poses `0`, `1`, `2` and `3`; the walking sequence is `0 → 2 → 1 → 3`
at 110ms per pose. Original poses retain their original slots; poses `2`/`3` are appended after
the four-player inventory. Use the manifest and guide labels rather than assuming adjacent
cells contain the entire cycle. Keep head size, central torso and the foot baseline stable.

Timed bombs use the original `bomb.shell.timed.<blast>` drawing plus appended `.1` and `.2`
poses. Keep their fuse attachment aligned with the separate spark. Gameplay selects these
poses from remaining fuse ticks; fixed manifest timing is an authoring preview. Remote bombs
and landmines retain their original single drawing. Reduced motion uses original pose zero.

### Death frame authoring

The twenty death slots contain `actor.<appearance>.death.0` through `.3` for all four
appearances, followed by `mob.death.0` through `.3`. Every death drawing stays inside its 80×80
frame with local bounds `(-40, -44, 80, 80)`, pivot `(40, 44)` and foot anchor `(40, 65)`.
Preserve all earlier frame rectangles and the four historical blank reservations; the new death
frames are appended baseline artwork and do not replace those reserved IDs.

Author four distinct stages that read as one fall/defeat sequence at 150ms per frame. Keep the
character identity, palette, hard pixel edges and ground contact coherent between stages. Frame
`.3` is the reduced-motion still, so it must be a stable terminal pose rather than an intermediate
blur or flash. The procedural template uses transformed existing actor and mob drawings only as
meaningful placement references; adopted source artwork remains authoritative.

Start an art revision from the canonical pair or the published artist bundle. `art:template`
produces the procedural reference; it does not reproduce or overwrite the adopted redraw.

## Provenance

The appended `effect.bloodPool` is a four-color procedural Pixi drawing for a small cosmetic
floor stain. Its reproducible Canvas import preserves every preceding frame and all pixels
outside the new rectangle; it introduces no generated character art. Reproduce the extension
from the prior manifest and sheet with:

```sh
node e2e/scripts/art-atlas-blood-pool.mjs \
  --source path/to/previous/arena-source.png \
  --manifest path/to/previous/arena-source.json \
  --out target/blood-pool-extension
```

Artists may repaint its 48×48 frame at slot 185, keeping the drawing centered and alpha binary.

The paired spike-pit extension appends three 48×48 terrain frames at slots 186–188:
`terrain.spikePit.idle`, `terrain.spikePit.warning` and `terrain.spikePit.raised`. Their stable
IDs and rectangles correspond directly to the authoritative gameplay phases. All preceding
assigned pixels, the four historical reserved entries and the final unassigned cell remain
unchanged. The repository append helper is used by the atlas acceptance scripts; ongoing artist
work continues through the normal template/export commands above.

The fire-artwork extension appends seven 48×48 effect frames at slots 189–195:
`effect.flame.2` through `.5`, `effect.ember`, `effect.smoke` and `effect.ignition`. The appended
flame poses share the same edge-join requirements as the original pair. The ember, smoke and
ignition drawings use binary alpha; runtime animation and opacity remain code-owned. Reproduce
the append-only source change and its decoded-RGBA proof with:

```sh
node e2e/scripts/art-atlas-fire-artwork.mjs \
  --source path/to/previous/arena-source.png \
  --manifest path/to/previous/arena-source.json \
  --out target/fire-artwork-extension
```

The owner-supplied retro-texture revision replaces all ten 3D material maps with tan sandstone,
gray masonry and red brick patterns. All other sprites and the paired manifest remain unchanged.
The PNG is adopted directly, without resizing, recoloring or metadata edits.

The owner-supplied polished 3D revision refines all ten material maps and the twenty death
poses within their existing rectangles. Directional player/mob artwork and all other sprites
remain unchanged. The PNG is adopted directly, without resizing, recoloring or metadata edits.

The twenty death poses were generated with OpenAI's built-in `image_gen`, using the existing
four uniforms and mob as identity references, then normalized with owner-authorized Canvas
processing. They retain each appearance's palette and the shared ground anchor. Existing
assigned artwork is unchanged. See the [death prompts and import rules](death-prompts.md);
repository licensing applies.

The adopted source is a Napoleonic redraw, AI-generated with ChatGPT (OpenAI) and supplied by
the repository owner. Player one wears an orange-and-white uniform with a plumed shako, player
two a blue uniform with a cockaded bicorne; terrain, bombs, flames, Oil, pickups and status cues
are repainted inside the previous silhouettes. The original redraw preserved the original 95 frame contracts, pivots, anchors,
binary alpha and flame edge bands; the appended variants below extend that inventory. Its new Oil palette is pinned by the Oil Barrel browser
scenario. Repository licensing applies.

Players three and four use forest-green/cream/silver and plum-purple/cream/gold uniform
variants with distinct headgear. Their current drawings were created in a separate ChatGPT
(OpenAI) session and supplied by the repository owner. They replace the initial generated
variants with pixel silhouettes and shading closer to the first two players. The supplied PNG
is adopted without resampling or repainting; its 16 directional frames retain 33-pixel character
height, the shared `(40, 65)` foot baseline, binary alpha and bounded palettes. The original 95
frame pixels and manifest are unchanged. These appearances prepare multiplayer artwork without
enabling additional live seats by themselves.

The [earlier variant prompts](player-variants-prompts.md) document the superseded generation,
not the owner-supplied revision; the prompt for the latter was not provided.

The additional walking and timed-bomb poses were generated with OpenAI's built-in imagegen
tool from the existing drawings, then processed with Pillow with the owner's explicit approval.
New actor poses retain the original heads and central torsos, combine generated passing-leg
phases with small counter-swinging arm edits, and use each uniform's existing palette. Bomb
poses retain the established fuse attachment. Every original frame rectangle is unchanged.
The [animation prompts and processing rules](animation-prompts.md) document this extension.

It replaces the v0.3.53 redraw, which was AI-generated: Claude (Anthropic, via Claude Code) wrote
a procedural Python/Pillow script that painted every frame, directed and reviewed by the
repository owner.

The v0.3.48 baseline, still produced by `art:template`, is rendered at resolution 1 from
Grenadiers' existing code-generated Pixi factories in `web/src/game/arena/`; it introduced no
AI-generated art, external artwork, external font download or new visual style. The generated
guide and preview files derive from those same repository sources. Record the author, tool and
license/provenance of future artist revisions alongside the adopted source; repository licensing
remains authoritative for distribution.

The roaming storm cloud occupies the final authoring slot as `environment.stormCloud`. Its
80×80 frame uses a centered `(40, 40)` pivot and contains a 48×37 drawing with a bounded
blue-gray palette and binary alpha. All preceding frame pixels and metadata remain unchanged.
The cloud was generated with OpenAI built-in imagegen using the atlas as a style reference,
then normalized with owner-authorized Pillow processing. See the [prompt and processing
notes](storm-cloud-prompts.md). Cloud opacity and authoritative movement belong to the renderer.

The lair and mob extension occupies the next row: `terrain.lair` is a 48×48 tile, followed by
`mob.north`, `mob.east`, `mob.south` and `mob.west` in 80×80 frames. Mobs share the actors’
`(40, 65)` foot anchor and `(40, 44)` frame pivot; their 33-pixel drawings use one bounded
48-color palette. The stone-gray imp has amber eyes and a brick-red scarf, distinct from
player uniforms. All preceding frame pixels, rectangles and reserved entries are preserved.
These sprites were generated with OpenAI built-in imagegen and normalized with owner-approved
Pillow processing; see the [lair/mob prompts and processing rules](lair-mob-prompts.md).

The owner-supplied stone-wall revision replaces the two solid-wall frames with gray stone
masonry. It also retouches appended walking and timed-bomb poses, the cloud, lair and mobs
inside their existing rectangles. The PNG is adopted without resampling or repainting;
frame IDs, dimensions, pivots, anchors and animation metadata remain unchanged.

## 3D material maps

The ten `material.*` slots are shared, artist-editable 48×48 maps for the Three.js arena:

- `material.floor.top.0` and `material.floor.top.1`;
- `material.wall.top` and `material.wall.side`;
- `material.block.top` and `material.block.side`;
- `material.lair.top` and `material.lair.side`;
- `material.rim.top` and `material.rim.side`.

Use opaque, hard-edged pixel patterns and deliberate shading. The current maps combine
tan sandstone, gray stone, red brick and brass accents. The 3D renderer keeps
the material color white so edits to these atlas rectangles remain the authoritative texture
colors. Top and side maps are intentionally distinct; do not derive private recolored copies in
renderer code.

The repository-only extension helper documents and reproduces how these slots were added to the
previous 1120×1792 source without resampling its assigned frames:

```sh
node e2e/scripts/art-atlas-materials.mjs \
  --source path/to/previous-arena-source.png \
  --manifest path/to/previous-arena-source.json \
  --out target/material-extension
```

It renders the ten placeholders through the existing Pixi/browser Canvas pipeline, copies only
their canonical rectangles, and writes `pixel-proof.json`. The previous assigned drawings stay
unchanged; acceptance evidence lives in the [release record](../../docs/releases.md). Use the normal
`art:template` and `art:export` commands for ongoing artist work; this helper is only for auditing
or reproducing the one-time append operation.

## Owner-supplied spikes and fire refresh

The owner supplied `arena-napoleonic-spikes-fire.png`, created in a separate ChatGPT session.
It repaints the fire family and related icons, three spike-pit phases and the cosmetic blood pool.
The adopted 1120×2240 sheet retains every stable rectangle, ID, pivot, anchor and animation.
Import processing reduces each pit to twelve existing colors without dithering and aligns only
the outer flame connectors for seamless neighboring tiles. All other supplied pixels are preserved;
no character or material textures change. Runtime animation and illumination remain code-owned.
See the [release record](../../docs/releases.md) for validation and the artist bundle.
