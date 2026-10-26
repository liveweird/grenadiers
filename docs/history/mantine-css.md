# Mantine CSS import reduction

This record captures the production-bundle and browser comparison performed on
2026-10-26 against source `e34bc505` with Node 24.21.0, npm 11.19.0, Vite 8.3.4,
and `@mantine/core` 9.6.2. It is historical measurement evidence, not a standing
claim about future component usage.

## Change

The application previously imported Mantine's complete `styles.css`. The measured
candidate imports the three global sheets and the 24 component sheets required by
the application's direct and transitive component graph. `InlineInput.css` is
required transitively by `Checkbox`; `Combobox.css` by the `NativeSelect` chevron;
and the modal path requires `ModalBase`, `Paper`, `Overlay`, `CloseButton`,
`ScrollArea`, and `VisuallyHidden` through its focus trap.

The component sheets remain in the same relative order as Mantine's complete
stylesheet. This is a correctness requirement: an early candidate put
`UnstyledButton.css` after `Button.css`, and the later unstyled rules overrode the
button presentation. That candidate was rejected before acceptance testing.

## Production build measurements

Both builds used the same source, lockfile, dependency tree, Node version, and
`npm run build` command. The preserved baseline contains the full-style import;
the candidate differs only in `web/src/main.tsx` style imports.

| Production output | Full Mantine CSS | Selected CSS | Difference |
| --- | ---: | ---: | ---: |
| CSS, raw | 242,988 B | 84,822 B | -158,166 B (-65.09%) |
| CSS, gzip | 36,695 B | 15,050 B | -21,645 B (-58.99%) |
| Entry JavaScript, raw | 716,254 B | 716,254 B | unchanged |
| Entry JavaScript, gzip | 172,307 B | 172,304 B | -3 B |
| All JavaScript, gzip | 337,188 B | 337,178 B | -10 B |
| JavaScript + CSS, gzip | 373,883 B | 352,228 B | -21,655 B (-5.79%) |

`web/scripts/check-bundle.mjs` passed for both builds. The candidate remains below
the existing 400,000-byte JavaScript and 40,000-byte CSS gzip budgets; no budget
or lazy-loading behavior changed during that A/B comparison. The integrated candidate subsequently
ratcheted the CSS gzip budget to 20,000 bytes, preserving 4,950 bytes of headroom above the
measured output; the JavaScript budget remains 400,000 bytes.

## Rendered A/B comparison

Two local server processes served the preserved baseline and candidate builds on
separate loopback ports. Playwright Chromium 1.63.0 used a light color scheme,
English locale, reduced motion, fixed viewports, and a settled modal transition.
Each scenario was exercised independently against the real HTTP, session, and
WebSocket endpoints. No source fixtures, capabilities, frames, or room seeds were
captured.

The PNG bytes were identical for all five stable scenarios:

- English desktop lobby with visible keyboard focus and a disabled Join button
- English desktop connection diagnostics with available health and disabled Ping
- English desktop editable match settings, including selects, checkboxes, and a
  disabled Apply button
- English desktop keyboard-controls modal with a focused Change button
- Polish 320 px lobby with a disabled Join button

The captured bounding boxes and computed presentation properties for representative
headings, inputs, selects, checkboxes, buttons, badges, modal elements, focus, and
disabled states were also identical. Both 320 px renders reported no horizontal
overflow, and neither build produced a page error.

Evidence was generated under `target/quality-review/css-imports-*` and is ignored
by version control. The repeatable source checks were:

```sh
npm --prefix web run lint
npm --prefix web test -- --run src/App.test.tsx src/game/client/GamePanel.test.tsx \
  src/game/client/GameplaySettingsPanel.test.tsx src/game/audio/SoundControl.test.tsx
npm --prefix web run build
```

At this checkpoint lint passed, all 51 selected tests passed, TypeScript and the
production build passed, and the bundle check passed.
