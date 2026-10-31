# Death animation artwork

The player and mob knockout strips were generated with OpenAI's built-in `image_gen` tool,
using the repository owner's existing four uniforms and mob as identity references. They are
non-gory supplementary frames; the original sprites and material textures are preserved.
Repository licensing applies to this derived artwork.

## Reference preparation

Each appearance's south/east/north/west pose was copied from the canonical source into a
2×2 reference, enlarged six times with nearest sampling. Generation produced one four-pose
strip per appearance. No new environment or texture artwork was requested.

## Shared prompt

The prompt below was used separately for each appearance, substituting the identity description.
The orange soldier used the same four-stage sequence with its orange/white shako description.

```text
Use case: identity-preserve. Make FOUR NEW death animation poses of the ONE pixel-art character shown in the reference; identity: {identity}. The reference shows existing standing views ONLY. Output a supplementary sprite strip of exactly FOUR equal cells ONE horizontal row, four complete isolated figures. Left to right: 0 startled recoil, knees bent, arms raised; 1 knees buckling and head tilting; 2 tipping sideways and falling with bent limbs; 3 fully crumpled lying on its side, recognizable headgear and uniform (or horns and red scarf for the mob). Non-gory playful Bomberman-style knockout: no wounds, blood, gore or violence detail. Preserve reference proportions, shading and identity. Match crisp tiny logical pixel-art at roughly33-pixel standing height, enlarged with hard square pixels, small deliberate flat palette, no smooth illustration. Same character scale and ground baseline across all cells, progressively lower body heights during collapse. Generous genuinely transparent padding, no floor, cast shadow, environment, labels/text or extra characters. All four cells fully isolated, no clipping or overlaps. New poses only; do not reproduce reference layout or standing views.
```

| Appearance | Identity |
| --- | --- |
| One | Orange coat, white cross belts/breeches, dark shako with gold band and red-white plume, warm face/hair. |
| Two | Blue coat, white cross sash and breeches, blond hair, black gold-edged bicorne with red-white cockade; no shako or plume. |
| Three | Forest-green coat, cream breeches, silver trim and white hair band, green round shako with a small ivory plume. |
| Four | Plum-purple coat, cream breeches, gold trim, black headgear with purple sash and a small ivory plume. |
| Mob | Stout stone-gray horned imp, amber eyes, ivory horns, red scarf, slate-gray limbs; no human uniform or hat. |

## Normalization and import

Native browser Canvas processing splits each generated strip into four cells and uses one
scale per appearance, fitting the first recoil to 33 opaque pixels high. Subsequent falling
poses retain their relative heights. Nearest sampling preserves hard pixels; alpha is
thresholded at 128 and transparent RGB cleared. Each opaque RGB maps to the nearest existing
colour in that appearance's directional palette. Isolated single-pixel artifacts are removed.

Every new frame fits its own 80×80 source rectangle, centered at X=40 with the opaque ground
baseline immediately above Y=65. The shared foot anchor is `(40,65)`. Only the twenty newly
assigned rectangles are painted; existing assigned rectangles are copied without resampling.
Native inventory tests enforce binary alpha, nonempty/distinct poses and palette membership.
See the [artist workflow](README.md) for stable IDs, guides and export validation, and the
[release record](../../docs/releases.md) for acceptance evidence.
