# Earlier player three and four uniform prompts

These prompts document the initial variants, superseded by the owner-supplied ChatGPT sheet
described in the [current provenance](README.md#provenance). They are retained as historical
authoring context and do not describe the current sprite pixels. Current appearances follow
the documented 48-color ceiling.

The initial variants were generated with OpenAI's built-in image generation tool using the existing atlas and player slices as style references. Existing artist pixels remain unchanged; new frames are mechanically sliced, scaled with nearest-neighbour sampling, anchored and packed into appended catalogue slots. Import standardizes character height to 33 pixels and the existing `(40, 65)` foot baseline using nearest-neighbour sampling, normalizes cutout alpha to the required binary mask, and maps only the new frames to bounded 28-color pixel-art palettes with forest/plum luminance ramps. Repository licensing applies.

## Player three

```text
Use case: stylized-concept. Asset type: production pixel-art game character walk sprite sheet.
Input images are STYLE AND SCALE REFERENCES ONLY: current Grenadiers Napoleonic infantry sprites and full atlas. Create one new character, not a redraw of the old atlas.
Create a transparent 4-column by 2-row sprite sheet, eight equal square cells, ideally 1024 by 512 pixels. Columns in exact order: north/back, east/right profile, south/front, west/left profile. Row 1 walking frame A, row 2 walking frame B with opposite foot leading. Same character, costume, scale and lighting in every cell; consistent head and foot baseline. No labels, borders, guides, terrain, ground shadows, weapons, text or extra characters.
Match the references' chunky low-resolution top-down Bomberman/Dynablaster character proportions, tiny readable face, dark pixel outlines, flat discrete pixel clusters and Napoleonic uniform detail. Character art should look native at approximately 24 pixels wide by 32 pixels high, not like a large painted illustration. Render enlarged hard square pixels, no anti-aliasing, no gradients, no soft lighting. Transparent background with only fully opaque character pixels; preserve clear gaps between cells. Eight separate complete uncut figures, ample transparent padding and feet aligned consistently.
Player THREE: forest-green infantry coat (#356044, highlighted #6f9270), cream cross-belts and trousers (#f4f0d2), silver plate/details (#c8cdd0), black boots. Distinct tall black grenadier shako with a silver front plate and short pale plume. Napoleonic-inspired but playful. Green is clearly dominant, distinguishable from existing orange-and-white player one and blue player two.
```

## Player four

```text
Use case: stylized-concept. Asset type: production pixel-art game character walk sprite sheet.
Input images are STYLE AND SCALE REFERENCES ONLY: current Grenadiers Napoleonic infantry sprites and full atlas. Create one new character, not a redraw of the old atlas.
Create a transparent 4-column by 2-row sprite sheet, eight equal square cells, ideally 1024 by 512 pixels. Columns in exact order: north/back, east/right profile, south/front, west/left profile. Row 1 walking frame A, row 2 walking frame B with opposite foot leading. Same character, costume, scale and lighting in every cell; consistent head and foot baseline. No labels, borders, guides, terrain, ground shadows, weapons, text or extra characters.
Match the references' chunky low-resolution top-down Bomberman/Dynablaster character proportions, tiny readable face, dark pixel outlines, flat discrete pixel clusters and Napoleonic uniform detail. Character art should look native at approximately 24 pixels wide by 32 pixels high, not like a large painted illustration. Render enlarged hard square pixels, no anti-aliasing, no gradients, no soft lighting. Transparent background with only fully opaque character pixels; preserve clear gaps between cells. Eight separate complete uncut figures, ample transparent padding and feet aligned consistently.
Player FOUR: rich plum-purple hussar-inspired coat (#704269, highlighted #a16991), cream trousers and decorative chest cording (#f4f0d2), warm gold buttons/hat badge (#d6a94e), black boots. Distinct rounded black fur busby with a small gold badge and pale side plume, different silhouette from the shako and bicorne in the references. Purple is clearly dominant, distinguishable from orange, blue and forest green.
```

