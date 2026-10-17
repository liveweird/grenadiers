# Game runtime reference measurements

The game has an explicit optimized reference harness for the synchronous match runtime and the
real asynchronous room owner. Run it with `scripts/measure-game.sh`. The script obtains the exact
release libtest executable from Cargo's compiler-artifact output, runs each ignored measurement in
a separate process, writes compact
credential-free JSON Lines to `target/m7-measurements/results.jsonl`, records the environment,
and captures process resource usage with the platform `time` tool. A selected budget failure or
missing result exits nonzero.

These tests exercise production `MatchRuntime` capture, checked projection, JSON encoding, and
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

Both references use source `f1678dfc9d9255504fce8b0c8dcafdb2ac07f96f`, optimized Rust 1.99.1,
Node 24.21.0 and two Tokio workers. Native is a clean checkout on Darwin 27.0.0, arm64,
Apple M5 Max, 18 visible logical CPUs and 128 GB physical memory, without harness affinity or
memory constraints. Linux is a clean Git export on OrbStack Linux 7.2.9/aarch64 on the same host:
18 CPUs visible to `getconf`, **two allocated via affinity/cpuset 2–3**, no CFS quota,
512 MiB memory limit with swap disabled, 128 PID limit, and no network. Its reference image is
`sha256:b5629e98174e72e4ab2c1a510d2a80c1dc5659b4a94ad4b6b96c78e807a13607`.

| Observation | Native | Linux, two allocated CPUs |
| --- | ---: | ---: |
| Runtime all ticks, p99 / max (541 samples) | 15 / 63 µs | 6 / 29 µs |
| Publishing ticks, p99 / max (183 samples) | 39 / 63 µs | 11 / 29 µs |
| Phase/round transitions, max (3 samples) | 63 µs | 29 µs |
| One-room action round trip, p99 / max | 47 / 88 µs | 84 / 148 µs |
| One-room observed publication lag, p99 / max | 3.938 / 4.210 ms | 8.898 / 10.143 ms |
| Eight-room action round trip, p99 / max | 165 / 8,857 µs | 47 / 99 µs |
| Eight-room observed publication lag, p99 / max | 3.713 / 12.114 ms | 5.113 / 26.896 ms |

All chosen budgets pass. Each owner advances exactly 1,800 ticks during its 30-second window;
rooms start sequentially and the JSON records each starting tick. Largest runtime snapshot is
4,824 bytes; observed one/eight-room frames are 3,724/4,001 bytes. One/eight-room watch changes
are 601/4,808; these may coalesce. Observed interval p99 is native 52.314/51.655 ms and Linux
56.533/53.132 ms. Sampled normal-load queue occupancy is one; the separate pressure process
fills 32 slots, rejects command 33, and completes shutdown. Every owner cleanup completes.

| Separate process peak RSS | Native BSD `time`, bytes | Linux GNU `time`, KiB |
| --- | ---: | ---: |
| Runtime | 8,323,072 | 5,324 |
| One owner | 9,650,176 | 6,828 |
| Eight owners | 14,237,696 | 9,608 |
| Queue pressure | 8,306,688 | 6,336 |

RSS includes the test binary/runtime, not only room data; do not divide these process peaks into
per-room heap estimates. Displayed microseconds are integer-truncated; zero medians in raw records
mean below one microsecond. Three transition samples describe these observations, not a tail
reliability guarantee. The private journal's separate <4 MiB record-storage bound is not a
whole-room memory bound.

Every record includes exact source commit, clean/dirty/export state, binary SHA-256 and image
identity. Native defaults to `none-native`; Git-free exports require a 40-hex source commit and
image runs provide their immutable image identity. Both binary hashes, environments, all eight
records, process timing files and container inspection are retained in
`/tmp/grenadiers-v0311-reference`. The source/image/provenance and RSS units were independently
reviewed. These developer references exclude HTTP/WebSocket/network/browser costs and human
playability; exact-image application acceptance and the rest of M7 remain separate.
