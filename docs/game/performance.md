# Game runtime reference measurements

The game has an explicit optimized reference harness for the synchronous match runtime and the
real asynchronous room owner. Run it with `scripts/measure-game.sh`. The script obtains the exact
release libtest executable from Cargo's compiler-artifact output, runs each ignored measurement in
a separate process, writes compact
credential-free JSON Lines to `target/m7-measurements/results.jsonl`, records the environment,
and captures process resource usage with the platform `time` tool. A selected budget failure or
missing result exits nonzero.

These tests exercise production `MatchRuntime` capture, checked projection, JSON encoding, and
`PreparedMatchOwner` scheduling. They do not include HTTP, WebSocket, network, browser, container,
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

## Reference recording status

Earlier measurements came from an uncommitted harness and are provisional, so this document does
not publish them as a source reference. After the harness is committed, regenerate a clean native
record and a Linux record constrained to two allocated CPUs. Keep each run's `environment.txt`,
`results.jsonl`, and `*.time.txt` files together.

Every record includes the source commit, whether the source was clean, dirty, or a Git-free export,
the exact test binary's SHA-256, and an image identity. Native runs default to
`none-native`; image runs must set `MEASUREMENT_IMAGE_IDENTITY` to the immutable image reference.
A Git-free clean export must also set `MEASUREMENT_SOURCE_COMMIT` to its 40-character commit ID.
The runner uses the repository's pinned Node 24 executable; `NODE24_BIN` may point to that same
version inside a reference image.

Exact JSON and resource output is retained under `target/m7-measurements/` for this worktree. It
must be regenerated for a source revision because build artifacts are not committed. Process RSS
includes the test binary and runtime. Each normal-load profile and the queue-pressure check has a
separate process peak; none is a per-room heap estimate. Do not compare results across machines as
if they were the same capacity claim.

The private replay journal separately caps its record storage below 4 MiB. That bound does not
describe total room memory. A Linux reference with two allocated CPUs and explicit cgroup limits,
followed by exact-image browser acceptance, remains required before making deployment capacity or
playability claims.
