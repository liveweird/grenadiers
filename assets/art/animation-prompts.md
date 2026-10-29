# Animation artwork provenance

The supplementary walking and timed-bomb artwork was generated with OpenAI's built-in
`image_gen` tool, using the owner's existing Napoleonic sheet as reference. The original
frame rectangles remain unchanged. Repository licensing applies to the derived artwork.

## References

For each uniform, the existing north/east/south/west poses `0` and `1` were copied into a
four-column, two-row reference and enlarged four times with nearest-neighbour sampling.
The three original timed-bomb shells supplied a three-column, two-row bomb reference.
The current sheet retains those original reference drawings in their original slots.

## Player-one prompt

```text
Use case: identity-preserve. Asset type: eight NEW walking animation poses for the orange-and-white Napoleonic soldier in the reference. The reference shows existing standing/stride frames. Preserve EXACTLY the same short pixel-art soldier identity, orange coat, white breeches, black shako with red plume and gold trim, face/hat proportions and small pixel detail. Generate only a supplementary sprite sheet, NOT terrain or UI. Canvas 1024x1024, EXACTLY FOUR COLUMNS and TWO ROWS, exactly eight complete isolated soldiers. Columns left to right: NORTH (back view), EAST (right-facing side view), SOUTH (front view), WEST (left-facing side view). Row1 a new passing-leg pose with right foot raised and arms counter-swinging; row2 a new opposite passing-leg pose with left foot raised and opposite arm swing. Each sprite centered in its 256x512 cell; consistent head size and baseline within each row. Draw as crisp 33-pixel-tall logical sprites enlarged in solid 4x4 pixel blocks, not smooth illustration. Keep every sprite fully separate with generous transparent margin; all eight silhouettes fully inside their cells. Truly transparent background, no gradients/background tint/glow/floor/shadow/checkerboard, no text/labels. Restrict to the existing uniform's approximately 27 flat solid colors. No extra characters, no weapons. Different articulated legs and arms in every row, no identical standing poses.
```

## Other uniform prompts

The following prompt was used once per uniform, replacing `{uniform}` with its description:

```text
Use case: identity-preserve. Asset type: eight NEW walking animation frames for this reference pixel-art Napoleonic soldier. Preserve the EXACT uniform identity: {uniform}. The reference shows existing standing/stride poses; generate complementary passing-leg poses. A supplementary sprite sheet ONLY, no terrain or UI. Canvas square1024x1024, EXACTLY 4 COLUMNS x 2 ROWS = eight isolated complete characters. Columns left-to-right NORTH/back, EAST/right-facing side, SOUTH/front, WEST/left-facing side. Row1 right passing leg raised with counter-swinging arms; row2 left passing leg raised with opposite arm swing. Consistent proportions and baseline; each sprite centered in its own256x512 cell with generous transparent margin. Crisp tiny33-pixel logical character enlarged in solid4x4 pixel blocks, matching the reference's headgear/face/coat, not smooth illustration. Truly transparent empty background, no gradient/tint/glow/shadow/floor/checkerboard. No labels/text. Approximately32 flat colors with deliberate pixel shading. Eight distinct articulated leg/arm poses, not repeated standing pictures. All sprites fully in their cells. Do not introduce weapons or change headgear/color identity.
```

- Player two: `blue coat, white diagonal sash and breeches, black gold-edged BICORNE hat with red/white cockade, blond hair; no plume/shako`
- Player three: `forest-green coat, cream breeches, SILVER trim and white hair band, green round shako with a small ivory plume`
- Player four: `plum-purple coat, cream breeches, GOLD trim, black headgear with purple sash and a small ivory plume`

## Bomb prompt

```text
Use case: precise-object-edit. Asset type: SIX extra animated grenade shell poses for this existing pixel-art bomb sheet. Reference has ordinary dark iron sphere, piercing dark sphere with CYAN UP ARROW, power dark sphere with ORANGE DIAMOND. Create supplementary sheet exactly THREE COLUMNS x TWO ROWS, six complete separated bombs. Columns standard, piercing, power in that order. Row1 slightly COMPRESSED squat shell before pulse. Row2 slightly EXPANDED rounder shell at pulse peak. Match the reference small black/navy iron body and highlight/shading, preserve every special emblem identity, long thin brown fuse pointing upper right. No antenna, no remote trigger, no landmine. Keep consistent centered x and ground baseline. Crisp 8-bit pixel-art with deliberate pixel blocks, 40-pixel-tall logical sprites enlarged4x for readability. Each bomb within its own cell with lots of transparent padding; no overlap or clipping. Square or landscape canvas, uniform cell sizes. True transparent background, no gradients, cast shadows, glow, colored background, text or labels. No spark particles baked in: separate in-game spark layer supplies them. Exactly six bombs, not the original row plus six.
```

## Processing rules

Generated images are references, not ready-to-import source sheets. With the owner's approval,
Pillow processing extracted each cell, thresholded alpha at 128, removed transparent RGB,
scaled with nearest-neighbour sampling and selected colors from the original uniform/bomb
palette. Actor silhouettes remain 33 pixels tall with a `(40,65)` foot anchor.

To prevent head-size flicker, new actor frames reuse the original head and central torso;
generated passing legs occupy the lower rows. Outer arm strips counter-swing by one pixel.
The two original poses remain untouched. New compressed/expanded bomb bodies retain the
original top 30 rows so their fuse attachment stays stable.

Only the 38 appended rectangles were populated. Checks compare all 111 old frame rectangles
with the previous source, reject nonbinary alpha and gutter/reservation paint, enforce the
48-color uniform budget, and ensure every added actor pose differs from both old poses and
its new partner. The import/export pipeline validates the completed PNG/JSON pair.
