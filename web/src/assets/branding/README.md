# Grenadiers branding

These are the original user-supplied transparent artwork files, preserved without resampling:

- `grenadiers-logo-1536x384.png`: combined desktop logo.
- `grenadiers-wordmark-1024x256.png`: compact header wordmark.
- `grenadiers-icon-512.png`: square PNG browser/touch icon.
- `grenadiers-favicon.ico`: multi-size browser icon.

`App.tsx` selects the combined logo or compact wordmark with a responsive picture. Header CSS
clips only transparent source padding and uses pixelated scaling; keep that crop aligned if an
artist changes the canvas padding. The translated image alternative supplies the accessible
page heading. `index.html` references the icons, which Vite exports to hashed asset URLs.

Edit these canonical files for future branding changes. Keep transparency and pixel edges;
check desktop/mobile header framing, accessible names, and production icon URLs after replacement.
The arena sprite sheet remains separately maintained under `assets/art/`.
