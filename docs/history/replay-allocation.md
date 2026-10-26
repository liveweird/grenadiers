# Replay allocation observations — 26 October 2026

These isolated developer measurements describe the tested sources, not a hardware budget,
throughput improvement, whole-room memory measurement or registry-lock contention result.
The current [policy](../decisions/0025-production-replay-recording.md) and
[probe command](../game/performance.md#replay-allocation-probe) own current guidance.

Before source: `4f9933608961912aed735887db3b457668dcedcd` (eager recording).
After source: `6202bc86e3a16d275cae54f2f5facb85efc2f2ea` (production disabled, explicit test opt-in).
Both used rustc 1.99.1 (58cf8f9c3), `cargo test --release`, macOS/aarch64 and the ExpandedV5
profile. Construction setup and destruction were outside timing. Workload setup/inspection
were outside timing: two occupants/readiness, 220 ticks and periodic movement; 464 records
when enabled, well below truncation. Complete private state and publication revision matched
between after-change recording modes.

A record is 64 bytes and the explicit journal reserves 60,000 records: 3,840,000 vector bytes.
The production constructor reserves no journal vector. This allocation result is structural;
it is not a resident-set or total room heap result.

| Observation | Samples | p50 ns | p95 ns | p99 ns | Maximum ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before: construction | 512 | 84 | 125 | 167 | 83,708 |
| After: disabled construction | 512 | 41 | 42 | 42 | 26,375 |
| After: recorded construction | 512 | 83 | 125 | 125 | 41,167 |
| Before: workload | 128 | 416,792 | 672,042 | 721,417 | 1,662,125 |
| After: disabled workload | 128 | 266,791 | 426,250 | 438,791 | 1,066,417 |
| After: recorded workload | 128 | 263,208 | 389,583 | 444,625 | 529,625 |

The two after-change workload modes are similar; these samples do not establish a workload
speed improvement. Root coordinated quiet windows without competing build/browser work.
One initial invocation lacked environment metadata and failed before measuring; its diagnostic
is retained separately from the single completed before run. No timing retry selected samples.
Commands, full output and parsed JSON are retained locally as
`target/quality-review/replay-recording-{before,after}.*`; the initial failure has its own file.
