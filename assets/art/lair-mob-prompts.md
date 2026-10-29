# Lair and mob artwork provenance

The artwork was generated with OpenAI’s built-in imagegen tool. The canonical atlas supplied
style reference only. Original outputs were copied into the workspace before normalization.
No CLI or separate API workflow was used. Repository licensing applies.

## Generation prompts

### Lair tile

```text
Use case: stylized-concept. Asset type: ONE game-ready pixel-art 48x48 terrain tile for Grenadiers, a top-down Napoleonic Bomberman arena. The supplied atlas is a STYLE REFERENCE ONLY, not an edit target. Generate only ONE indestructible monster lair tile: a square weathered sandstone block matching the reference's warm cream/gold stone walls, with a dark recessed iron-barred arched opening, cracked stone frame, a pair of tiny muted amber eyes deep inside, dark moss at lower corners. Read clearly as a solid impassable stone wall with a spawning doorway, not walkable floor or a pickup. Flat orthographic top-down game view with the same slightly visible front face as the reference walls. Crisp hand-painted retro 16-bit pixel art, pixel-native detail equivalent to 48x48 enlarged with nearest-neighbor, bounded palette, strong dark outline, readable compact silhouette. Square tile fills its square logical bounds edge to edge, no external shadow. Actual transparent background outside the tile silhouette. No environment, no characters outside doorway, no text or labels, no alternatives, no contact sheet, no icon border, no bloom, no antialiasing, no gradient transparency. Preserve reference style only; do not reproduce the atlas.
```

### South-facing mob

```text
Use case: stylized-concept. Asset type: ONE game-ready enemy mob sprite for Grenadiers, a top-down Napoleonic Bomberman arena. Supplied atlas is STYLE REFERENCE ONLY, do not alter or reproduce it. One squat little stone-gray dungeon imp facing SOUTH toward the viewer in the same 3/4 overhead perspective as the reference soldiers. Chunky compact body, two short horn nubs, large amber eyes, dark charcoal face, weathered brick-red shoulder scarf, short clawed arms, sturdy little feet on a shared horizontal baseline. No gun, sword, bomb or uniform/headgear. A readable playful dangerous monster distinct from all four soldiers. Detailed retro 16-bit pixel art with crisp stepped pixel contours, dark indigo outline, deliberate limited gray/ochre/brick palette, no smooth rendering. Pixel-native drawing equivalent to around 30 pixels wide and 33 pixels high, enlarged nearest-neighbor for output. Entire figure centered with generous transparent padding and no cropped parts. Actual transparent background, no floor, no cast shadow, no text, no icon frame, no scene, no particles, no glow, no antialiasing, no gradient transparency. One SOUTH/front-facing pose only, no alternate poses or contact sheet.
```

### North-facing mob

```text
Use case: identity-preserve. Asset type: ONE game-ready NORTH/back-facing directional pose of the exact same stone-gray dungeon imp shown in the supplied mob reference. The mob reference is the identity reference; the arena atlas is only a scale/style reference. Keep the same squat compact body, two short ivory horn nubs, weathered gray skin, brick-red shoulder scarf with two short tails, short clawed arms and sturdy feet. Rotate the same creature to face NORTH away from the viewer in the same slight top-down camera; show back of head/body and scarf knot/tails. No visible front eyes, no additional gear. Maintain the same apparent height/width, head/body proportions, horn positions and ground baseline as the South pose. Crisp retro 16-bit game pixel art, pixel-native detail equivalent to 30 by 33 pixels enlarged nearest-neighbor, dark outline and limited gray/ochre/brick palette. Entire isolated figure centered with generous actual transparent padding. No floor, no cast shadow, no text, no scene, no extra poses, no icon frame, no glow, no antialiasing, no gradient transparency. ONE NORTH pose only.
```

### East-facing mob

```text
Use case: identity-preserve. Asset type: ONE game-ready EAST/right-facing directional pose of the exact stone-gray dungeon imp in the supplied mob identity reference. Arena atlas is scale/style reference only. Rotate same squat compact imp to face right/east in the same slight overhead camera as the soldiers. Same two short ivory horn nubs, gray skin/head proportions, large amber eye visible in profile, charcoal face, weathered brick-red shoulder scarf, short clawed arms and sturdy feet. Full body profile, feet on same ground baseline as reference, same height and similar width. No weapons or uniform/headgear. Crisp detailed retro 16-bit pixel art, pixel-native detail equivalent to 30 by 33 pixels enlarged nearest-neighbor, strong dark outline, limited gray/ochre/brick palette. One isolated figure centered with generous actual transparent padding, all extremities visible. No floor, cast shadow, scene, extra poses, text, labels, icon frame, glow, antialiasing or gradient transparency. ONE EAST pose only.
```

## Processing and preservation

The owner explicitly authorized Python/Canvas size, palette and transparency cleanup. Pillow
thresholds alpha at 128, crops opaque bounds and resizes with nearest-neighbor sampling. The
lair is normalized to 48×48 with at most 32 colors. Mob drawings retain their aspect ratios at
33 pixels high, use a shared palette of at most 48 colors without dithering, reserving three amber/ivory eye
colors so rare identifying highlights survive quantization, and sit inside
80×80 frames on foot baseline `(40, 65)`. West is a horizontal reflection of East. All source
alpha is binary; runtime shadows and translucency are separate.

The extension appends one authoring row. Stable IDs are `terrain.lair`, `mob.north`, `mob.east`,
`mob.south` and `mob.west`. Existing frame rectangles, pivots, anchors, animations, reserved
entries and their RGBA pixels remain unchanged. The five unused cells in the new row stay
transparent. Edit the canonical PNG and use the normal validated atlas export workflow.
