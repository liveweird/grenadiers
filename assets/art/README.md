# Grenadiers arena art handoff

This directory contains the reviewed, artist-editable arena source: `arena-source.png` and its
paired `arena-source.json` manifest. ART-ATLAS v0.3.48 supplies the extraction, validation and
preview workflow. Published v0.3.49 adopts these same inputs in gameplay; see the
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

The 1120×1344 authoring sheet has 120 physical cells: 111 assigned entries (107 baseline
drawings and four blank reserved entries) followed by nine unassigned cells. Cells are 112×112
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
107 baseline frames, unrotated and untrimmed, with two-pixel extrusion and two-pixel transparent
padding. Labels and guides never enter the runtime texture.

Both commands refuse an existing output directory, so an artist source or earlier review cannot
be silently replaced. Choose a new output path or deliberately remove/archive the generated
target after review.

## Pixel and anchor rules

- Keep pixels on integer coordinates with nearest-neighbour sampling and no antialiasing.
- Keep each player appearance within 48 opaque RGB colors across its eight directional frames;
  use deliberate shading ramps rather than diffuse generated color noise.
- Source alpha is binary: every pixel is either fully transparent (`0`) or fully opaque (`255`).
  Do not bake translucent shadows into the PNG. Shadow opacity remains manifest metadata (actor
  shadow `0.3`, bomb shadow `0.28`) and is applied exactly once by the region importer.
- Paint only inside each baseline frame rectangle. Slot gutters and reserved frames must remain
  transparent.
- A frame pivot preserves its drawing origin and must equal `(-localBounds.x,
  -localBounds.y)`. All 32 directional actor poses retain the shared `(40, 65)` foot anchor.
- Keep IDs path-safe and stable. The exporter rejects missing, duplicate, unexpected or path-like
  IDs, changed required animations, invalid dimensions and texture-budget overflow.
- Flame frames must reach all four edges so adjacent rays join. The procedural template's
  `effect.flame.1` keeps its historical vertical two-pixel gap; the adopted source fills a
  continuous 22-pixel band on every edge of both frames.

Use the standalone `art-atlas.html` renderer diagnostic to load a source PNG and manifest, inspect
all labelled drawings, and see the representative arena rebuilt from actual Pixi atlas regions.
The diagnostic shows pickups, actor/cue compositions, bomb shells, spark, pause mark, Oil states
and adjacent flames. It is not linked from the gameplay or product flow.

## Provenance

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

It replaces the v0.3.53 redraw, which was AI-generated: Claude (Anthropic, via Claude Code) wrote
a procedural Python/Pillow script that painted every frame, directed and reviewed by the
repository owner.

The v0.3.48 baseline, still produced by `art:template`, is rendered at resolution 1 from
Grenadiers' existing code-generated Pixi factories in `web/src/game/arena/`; it introduced no
AI-generated art, external artwork, external font download or new visual style. The generated
guide and preview files derive from those same repository sources. Record the author, tool and
license/provenance of future artist revisions alongside the adopted source; repository licensing
remains authoritative for distribution.
