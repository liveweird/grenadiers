# React gameplay publication measurement — 26 October 2026

> Historical evidence: this records one bounded local measurement at source
> `e34bc505906eec657bf03d899ecc1b1eea2a7fef`. It is not a standing performance
> guarantee or a replacement for release acceptance.

## Question and disposition

The review observed that the gameplay client publishes a new React state for each accepted
snapshot and asked whether arena and HUD subscriptions should be split. The measured 20 Hz
workload did not demonstrate a render-budget problem. Keep the current publication model; do not
introduce a subscription refactor solely to reduce one publication per accepted snapshot.

Across four 200-snapshot sequences, inclusive React Profiler `actualDuration` had a 2.1 ms or
lower p95, a 2.4 ms or lower p99, and a 2.6 ms or lower maximum. Every accepted snapshot reached
the real renderer exactly once, duplicate revisions caused no React or renderer work, and the
renderer was constructed once and never remounted. These observations support the disposition,
but do not prove capacity beyond this workload or hardware.

## Method

An ignored, temporary Vite entry rendered the actual `App` under the actual Mantine and locale
providers and a React `Profiler`. The production build aliased `react-dom/client` to
`react-dom/profiling`. It retained the real `GameClient`, generated validation, HUD, `GameArena`,
and Pixi renderer. Only API health/session responses and the native WebSocket server boundary
were simulated. No production source or tracked configuration was changed.

Each profile sent 200 separately validated snapshots against an absolute monotonic schedule at
50 ms intervals. Revisions increased by one, logical and physical ticks by three, and remaining
round/effect ticks decreased consistently. The four profiles covered English and Polish with a
steady actor, then English and Polish with movement plus a 600-tick Golden Flame countdown.
The driver did not batch snapshots into one React action.

The first English profile then resent the final revision 200 times as a negative control. Final
HUD text, presented actor/effect state, canvas output, renderer update count, and renderer
construction/destruction counts were checked. Only aggregate timings and final-state hashes were
retained; the evidence contains no session capability, raw frame, or fixture seed.

The final run used macOS 27.0.1 on an Apple M5 Max (`Mac17,7`, arm64, 18 physical/logical CPUs,
128 GiB memory), Node 24.21.0, Vite 8.3.4, React/React DOM 19.3.0, Pixi 8.22.0,
Playwright 1.63.0, and its Chromium 153.0.8010.12. Repository build and browser work was paused
for the measurement window, but ordinary host and external application services were not isolated
and no CPU affinity was applied.

## Results

All durations are milliseconds. Duration columns are React Profiler `actualDuration`; cadence
is the interval between browser-driver dispatches or observed real-renderer updates.

| Profile | React callbacks | React p50 | p95 | p99 | max | dispatch p50 / p95 / max | renderer delivery p50 / p95 / max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| steady, English | 201 | 1.5 | 1.8 | 2.1 | 2.2 | 49.89 / 51.92 / 52.99 | 49.9 / 52.2 / 53.4 |
| steady, Polish | 201 | 1.7 | 2.1 | 2.4 | 2.6 | 49.97 / 51.90 / 52.85 | 49.9 / 54.4 / 55.5 |
| movement/effect, English | 201 | 1.5 | 2.1 | 2.4 | 2.5 | 49.92 / 51.97 / 52.86 | 49.9 / 51.5 / 53.1 |
| movement/effect, Polish | 201 | 1.6 | 2.1 | 2.4 | 2.5 | 49.89 / 51.85 / 53.04 | 49.8 / 54.0 / 55.8 |

Every profile ended at physical tick 603 with the HUD clock at `2:50`, 200 renderer updates,
one renderer construction, and zero destructions. Steady profiles ended at x=512 with no effect;
movement profiles ended at x=3712 with the effect expired. English and Polish canvas hashes were
stable across the diagnostic and final runs. The duplicate-revision control produced zero React
callbacks and zero renderer updates.

Each sequence recorded 201 inclusive Profiler callbacks for 200 accepted publications. The same
fixed extra callback occurred in steady and movement/effect profiles, while the renderer still
received exactly 200 updates. It is therefore not a second snapshot publication or an
effect-expiry-specific commit. The aggregate-only probe did not retain callback phase/timestamp
correlation, so attributing this child/provider subtree commit more narrowly would be speculative.
Its duration remains included in the reported distribution.

## Pacing correction and evidence

An initial run waited 50 ms after each dispatch. Its measured dispatch p50 was 51.3–51.9 ms,
approximately 19.3 Hz, because dispatch overhead accumulated. That complete run is retained as a
relative-pacing diagnostic and was not selected or relabeled as 20 Hz. The driver was corrected
to wait against absolute deadlines, including dispatch overhead; the four profiles were then run
exactly once. Final dispatch and renderer-delivery medians were 49.8–50.0 ms as shown above.

The ignored reproduction probe remains local rather than becoming a mandatory framework. At the
time of measurement, evidence hashes were:

- absolute-20-Hz aggregate JSON: `77196b4e65d253bafcbb597f630fd8b7f9e3c03b1f0af2e4ffd1c143588af211`
- relative-pacing aggregate JSON: `fc90455419130cfadfd373134902fb636525e435aab9a503ddd0c346247b6d90`
- profiling entry: `c8e1b19db9ffaf47a0b96da2bb4fb63abf18892b6ec853d61bf2587b79a9587d`
- final driver: `95a6adfc4b213c8a3ebd9d9eb6558a9e37f7947eefc2c3992fdfc36b38792899`
- built entry HTML: `5e5b0fde73844884cdc955503ac74a23ebf656b2a53992cfca9bd894bde80393`

React `actualDuration` measures component render work, not full frame presentation. It excludes a
complete accounting of Pixi/GPU work, networking, server validation, and end-to-end input latency.
Likewise, this single local host run is not a throughput ceiling or a tail-latency guarantee.
Revisit subscription architecture only with a reproduced user-visible issue or a representative
measurement that exceeds an explicit frame/render budget.
