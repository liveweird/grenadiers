# Storm-cloud artwork provenance

The cloud was generated with OpenAI's built-in imagegen tool. The canonical atlas was a style
reference, not an edit target. The selected output was copied into the workspace before
processing; no CLI or separate API workflow was used.

## Generation prompt

```text
Use case: stylized-concept. Asset type: ONE game-ready pixel-art sprite for Grenadiers, a top-down Napoleonic Bomberman arena. The supplied atlas is a STYLE REFERENCE ONLY; do not reproduce, edit, rearrange or include the atlas. Generate just ONE isolated small roaming storm-cloud weather hazard. Rounded asymmetric slate blue-gray cloud puffs, crisp dark indigo outline, pale silver-blue highlights, dense shaded underside, three short bright blue rain streaks beneath, one subtle wind curl. Visual should read as a slowing weather cloud, not a damaging lightning attack. Match detailed retro 16-bit hand-painted pixel art and compact chunky silhouette of reference terrain/items. Pixel-native design equivalent to about 42 pixels wide and 38 pixels high, enlarged nearest-neighbor for output; deliberately limited palette, hard stepped pixels, flat orthographic view, no perspective. Cloud fits centered in square image with generous transparent padding. Actual transparent background. No floor, no scenery, no characters, no text, no labels, no frame, no icon border, no drop shadow, no lightning bolts, no glow, no soft antialiasing, no gradient transparency. One cloud only, no contact sheet, no alternatives. The reference is solely color/shading style, not an edit target.
```

## Processing and preservation

The repository owner explicitly authorized Python/Canvas size, palette and transparency
cleanup. Pillow processing thresholds alpha at 128, crops the generated opaque bounds,
resizes with nearest-neighbor sampling to 48×37, quantizes to at most 24 colors without
dithering, then centers the drawing in an 80×80 frame. The adopted drawing has 23 opaque
RGB colors and binary alpha.

Only the previously unassigned final slot is painted. Every earlier frame rectangle retains
its exact RGBA pixels and manifest metadata. The cloud occupies row 15 of the sheet. Its stable ID
is `environment.stormCloud`, with local bounds `(-40, -40, 80, 80)` and pivot `(40, 40)`.

Edit the cloud in the canonical PNG and export it through the normal atlas workflow. Do not
move its slot, add baked translucency, draw contact signals for hidden players, or turn the
weather artwork into a pickup. Runtime opacity, layering and movement remain separate.
