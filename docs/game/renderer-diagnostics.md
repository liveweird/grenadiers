# Isolated renderer capture diagnostics

This development-only experiment isolates the actual Pixi arena renderer from authentication,
WebSockets and game authority. It is separate from release browser acceptance and does not
change its assertions, screenshot options, deadlines or retry policy.

The fixture builds separately from the ordinary application:

```sh
npm --prefix web run build:renderer-diagnostics
npm --prefix web run preview:renderer-diagnostics
RENDERER_DIAGNOSTICS_BASE_URL=http://127.0.0.1:5188 npm --prefix e2e run test:renderer-diagnostics
```

Use pinned Node, Playwright and the reference Chromium image/launch flags. The fixture output
is `web/renderer-diagnostics-dist/`; ordinary deployment still uses `web/dist/`. Verify the
fixture marker `grenadiers-renderer-diagnostic` is absent from ordinary production artifacts.
The fixture permits only its static origin and has `connect-src 'none'`. It contains synthetic
public presentation data and no game client, credentials, sockets or production endpoint.

Each predeclared cell uses a fresh browser process and two isolated contexts: primary
1440×1050/document1440×1334, peer320×844/document320×1842 with reduced motion. Every initialized
renderer uses the same planned twenty-second,20Hz public snapshot deadline sequence. Late update deadlines
are counted and skipped; updates never catch up in bursts. RAF observations are scheduling
observations, not GPU frame counts. Synthetic updates stop before audit and capture; an active
renderer retains its real ticker, while a suspended renderer uses its actual `suspend()` method.

| Cell | Primary | Peer |
| --- | --- | --- |
| E/E | Never initialized | Never initialized |
| A/E | Active | Never initialized |
| S/E | Suspended | Never initialized |
| E/A | Never initialized | Active |
| A/A | Active | Active |
| S/A | Suspended | Active |

A/E versus S/E uses the same planned primary sequence; A/A versus S/A uses the same planned
two-renderer sequence. Record each delivered deadline ordinal (at most400) and the final motion
ordinal. Interpret a paired contrast only when these executed histories match exactly for each
initialized renderer; differing histories make that contrast inconclusive. Do not rerun cells
to manufacture a match. Empty controls are pristine and do not share renderer history. Each cell keeps the same guarded Axe
check, width assertion and one explicit full-page primary capture. Its120-second budget includes
warm-up; retries are zero. Only this isolated diagnostic config disables automatic failure
screenshots, keeping additional capture pipelines out of the measurement.

Retain exact source/build/browser identities, launch arguments, lifecycle/counter/geometry
summaries, original screenshot substages, heartbeat/event-loop observations, explicit PNGs and
original errors. Check retained canvas identity, connection and actual visibility before and after capture without extra
GL calls or canvas/ticker manipulation. Destroy renderers only during teardown.

Run the matrix once. A slow suspended/active cell alone does not prove GPU backlog: compare
matched active/suspended histories and peer-only activity. If the pristine baseline is slow or
no slow capture reproduces, record that result and stop this experiment. A production lifecycle
change requires a supported contrast and an independently reviewed intervention/confirmation
pair; this fixture itself changes no rendering policy.
