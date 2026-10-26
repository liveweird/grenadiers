# Game runtime reference measurements

The game has an explicit optimized reference harness for the synchronous match runtime and the
real asynchronous room owner. Run it with `scripts/measure-game.sh`. The script obtains the exact
release libtest executable from Cargo's compiler-artifact output, runs each ignored measurement in
a separate process, writes compact
credential-free JSON Lines to `target/m7-measurements/results.jsonl`, records the environment,
and captures process resource usage with the platform `time` tool. A selected budget failure or
missing result exits nonzero.

These tests exercise production `MatchRuntime` stepping, checked projection, JSON encoding, and
`PreparedMatchOwner` scheduling. They do not include HTTP, WebSocket, network, browser, container-startup,
or database costs and do not establish capacity at the configured 32-room ceiling.

## Selected budgets

- Runtime tick, including publication when due: p99 at most 8.33 ms, half the 60 Hz period.
- Real-owner observed publication deadline lag: p99 at most 50 ms.
- Encoded observed server frame: at most 65,536 bytes.
- Normal owner load: no rejected legal refresh, overload interruption, sustained tick debt, or
  incomplete cleanup.

The owner lag uses Tokio's monotonic clock for both the production absolute tick deadline and the
observer receive time. It includes observer scheduling and is therefore an upper bound on owner
lateness. Watch changes may coalesce and are not publication counts. Command occupancy is sampled
around submissions and is not a true peak. Existing deterministic tests remain the proof of
every-third-tick cadence, four-tick catch-up, the 500 ms overload boundary, 32 command slots,
32 staged admissions, and lifecycle cleanup ordering.

## Workloads

The synchronous case warms one generated room, refreshes safe movement, plants two real bombs,
observes a real draw and result hold, and constructs the next generated round. It reports ordinary,
publishing, and phase/round-transition samples separately.

Each owner case starts one or eight independently occupied generated rooms with two Tokio workers,
waits through the real three-second countdown, and then runs for 30 seconds on real timers. Both
seats refresh movement toward their adjacent border wall at 10 Hz, keeping actors alive. One watch
receiver per room is drained promptly while another remains unread. The result reports action
round-trip latency, authoritative tick progress, observed watch intervals and deadline lag, frame
sizes, and sampled command occupancy. Shutdown uses the production sticky watch and requires every
task and cleanup callback to complete. A fourth, separate process fills all 32 command slots and
requires the next submission to return `rateLimited`. Its allocations cannot affect the one-room
process's resident-set peak.

## Reproduce a constrained Linux reference

Use pinned Node 24.21.0 on PATH (or `NODE24_BIN`) for native runs. To prebuild the same
optimized harness in its pinned development image, then measure without network/build overhead:

```sh
reference_commit=$(git rev-parse HEAD)
docker build -f scripts/Dockerfile.measurements \
  --build-arg MEASUREMENT_SOURCE_COMMIT="$reference_commit" -t grenadiers-reference:local .
reference_image=$(docker image inspect grenadiers-reference:local --format '{{.Id}}')
docker run --name grenadiers-reference --cpuset-cpus=2,3 --memory=512m --memory-swap=512m \
  --pids-limit=128 --network=none -e CARGO_NET_OFFLINE=true \
  -e MEASUREMENT_IMAGE_IDENTITY="$reference_image" grenadiers-reference:local
docker cp grenadiers-reference:/build/target/m7-measurements ./target/linux-reference
docker rm grenadiers-reference
```

Choose two valid CPU IDs on the reference host and record them. Build from a clean committed
checkout/export; `MEASUREMENT_SOURCE_COMMIT` identifies that source, including the harness. This
is a development measurement image, separate from the application's runtime image.

## Recorded clean references

Historical measurements and environment identities are preserved in
[game acceptance history](../history/game-acceptance.md#runtime-reference-measurements).

## Static arena terrain

The Pixi scene caches only terrain as a texture at the initialized renderer's actual resolution
(DPR capped at two), with nearest sampling and antialiasing disabled. Blocks, pickups, bombs,
flames and actors remain live. The cache rebuilds when terrain dimensions/cells change, on a
new room/round or reset, and after WebGL context restoration. Identical snapshots and animation
ticks reuse it. Reset releases the cache; final renderer destruction removes the restoration
listener and releases GPU resources. A resumed renderer paints the current baseline before reveal.

The development-only exact-pixel regression compares this scene with an uncached terrain painter
at DPR one and two through initial paint, repaint, reset and actual context loss/restoration:

```sh
npm --prefix web run build:renderer-diagnostics
npm --prefix e2e run test:terrain-pixels
```

Use the pinned browser for reference results. Its separate config does not run the stopped
six-cell lifecycle experiment. All four phases require zero differing bytes at each resolution;
the development entries are excluded from the ordinary production build.

This optimization follows complete CPU-side submission measurements from the original reconnect
workload; its lossy browser trace did not establish a screenshot-delay cause. Evaluate the changed
workload using [the opt-in observer](renderer-diagnostics.md), reporting indices per observed
animation callback or per measured interval. Callbacks are not rendered frames, source-span bounds
are not actual uploads, and absolute CPU totals from different-duration windows are not comparable
per-frame costs. Run native application acceptance and use remote checks only for distinct
coverage, following the [local-first policy](../../CONTRIBUTING.md#local-first-verification).

## Replay allocation probe

Production construction reserves no semantic journal buffer; explicit recording fixtures retain
the bounded buffer. The separate ignored probe compares construction and identical deterministic
workloads, with full private-state equality outside timing. It reports reserved vector bytes,
not RSS or registry-lock contention. Run from a clean checkout on an otherwise quiet machine:

```sh
GRENADIERS_MEASUREMENT_SOURCE="$(git rev-parse HEAD)" \
GRENADIERS_MEASUREMENT_TOOLCHAIN="$(rustc --version)" \
cargo test -p grenadiers-server --release --locked \
  game::rooms::replay::recording_measurement_tests::production_journal_allocation_measurement \
  -- --ignored --exact --nocapture
```

The Cargo profile is declared by that invocation; the probe rejects builds with debug assertions enabled.
[Recorded observations](../history/replay-allocation.md) retain source, workload and timing limits.
This optional measurement is separate from routine correctness and exhaustive replay gates.

## Registry admission and write pressure

The ignored `measurement::construction`, `measurement::registry` and `measurement::entropy` profiles separately exercise prepared owners, real registry admissions and entropy requests. Use the reproduction commands and limitations in the [recorded registry reference](../history/registry-admission.md). Barrier release does not establish simultaneous mutex contention; first-poll duration is an upper bound around acquisition and guarded work. Keep timing collection separate from builds and browser workloads.

WebSocket pressure regressions use real non-reading TCP peers and independently verify recovery after draining, bounded shutdown, healthy-peer delivery and resource cleanup. They preserve the production deadlines and are ordinary platform unit tests, rather than timing-capacity benchmarks.
