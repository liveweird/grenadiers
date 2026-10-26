# Registry admission reference — 2026-10-26

Clean native reference on macOS 27.0.1 / Apple M5 Max / arm64, 18 logical CPUs, without imposed affinity or competing agent builds/browser work. Cargo release profile, Rust 1.99.1. Source `aa66d310fe7a4059b98b8deac9815f73553b65b1`; compiled server libtest SHA-256 `a49506f3ca8d7d68189918578c304e11e10386a6d303ff2ae2702179d9a65bcf`. All five selected profiles ran once; no product retry. Nanosecond report units do not imply clock accuracy.

## Construction and admissions

Actual ExpandedV5 `PreparedMatchOwner::new`, production recording disabled: 512 samples. Construction p50/p95/p99/max: 0.250/0.292/0.416/120.084 microseconds; separate destruction 0.500/0.583/0.708/51.250 microseconds. Identity/seed preparation and destruction are outside construction timing. No room task is spawned and no round seed is requested.

Sequential fresh registries use batches of 1, 8 and 32, repeated 16 times. First admissions are reported separately; all succeed and cleanup leaves both registry maps empty. For the 32-room batches, first admission p50/p95/p99/max is 15.916/21.750/21.750/21.750 microseconds; the remaining 496 admissions are 15.500/18.625/21.917/24.958 microseconds. These are idle lobby admissions, not 32 running matches.

Barrier-released admissions use four executor threads, eight waves per level. Per-task timing starts when scheduled after the barrier and excludes barrier-to-scheduling delay; released tasks do not establish simultaneous execution or mutex contention.

| Released tasks | Successful admissions | Entire await p50/p95/p99/max (µs) | First poll p50/p95/p99/max (µs) |
| --- | --- | --- | --- |
| 2 | 16 | 15.458/84.083/84.083/84.083 | 7.125/41.709/41.709/41.709 |
| 4 | 32 | 18.334/32.209/32.625/32.625 | 9.708/25.458/27.709/27.709 |
| 8 | 64 | 18.541/38.958/50.958/50.958 | 8.375/30.667/46.000/46.000 |
| 16 | 128 | 19.208/40.792/53.500/62.875 | 8.042/28.709/44.625/55.459 |
| 32 | 256 | 14.792/49.375/57.667/66.583 | 6.417/41.792/51.083/59.833 |

First-poll duration bounds acquisition plus guarded identity/owner construction, insertion, task spawn and admission submission; it is not exact mutex hold time.

## Separate contention observer and entropy

A separate 32-task/four-thread observer across 16 waves makes 1,749 real `try_lock` attempts: 1,640 acquire immediately and 109 report `WouldBlock`. Subsequent blocking acquisition p50/p95/p99/max: 18.833/61.542/92.667/114.792 microseconds. The observer perturbs the workload; its window begins before task allocation/spawn and ends after admissions complete. Attempt proportions are neither per-wave contention rates nor production caller wait samples. All 512 admissions succeed and cleanup empties the registry.

Real entropy requests are measured separately in 128 batches; no random bytes are recorded. Five-byte requests p50/p95/p99/max: 1.084/1.167/1.458/1.584 microseconds; eight-byte requests: 1.542/2.333/2.750/4.000; 32-byte requests: 1.375/1.542/1.667/1.958. No round-seed request occurs before actual match start.

## Disposition

Keep the current short synchronous registry lock and entropy boundaries. These local lobby samples do not demonstrate a material problem requiring a cross-cutting concurrency refactor, establish running-match capacity, or promise production latency. Re-measure under a deployment workload if admission pressure becomes observable.

The non-reading WebSocket regressions use actual TCP backpressure and bounded send buffers. They require pending-write recovery after draining without advancing virtual time, bounded shutdown with an independently healthy peer, stream cleanup, empty room state and restored connection permits. They do not change production timeout values. Platform unit acceptance remains separate from this native timing reference.

## Reproduce

Build once from a clean committed checkout:

```sh
cargo test -p grenadiers-server --lib --release --locked --no-run --message-format=json
```

Use the emitted executable, one fresh process per ignored exact test, with `--ignored --exact --nocapture --test-threads=1`:

- `game::rooms::measurement::construction::expanded_v5_prepared_owner_construction_measurement`
- `game::rooms::measurement::registry::sequential_registry_admission_measurement`
- `game::rooms::measurement::registry::barrier_released_registry_admission_measurement`
- `game::rooms::measurement::registry::actual_registry_mutex_contention_observation`
- `game::rooms::measurement::entropy::real_entropy_requests_by_size_measurement`

Record source, executable hash, toolchain, host, executor threads and competing workloads. Keep compile and browser work outside timing. A collector parsing error after the first completed profile was repaired by parsing its preserved log; the completed profile was not rerun. A prior sandbox metadata failure occurred before measurements began.
