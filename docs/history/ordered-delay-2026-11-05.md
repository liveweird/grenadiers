# Ordered connection-delay acceptance — 2026-11-05

Stable v1.3.32 source `a81d14489f91147d1663cc3b0b3081a650ed5690` was checked against its
accepted image `sha256:74a44fac29bbb1f0e08f4cd618fd10740ce9daeb99b8c77ba5a91757207e0d79`.
An isolated Compose project mounted the declared `arena-v1-matches.seed` fixture read-only.
The primary localhost:8085 deployment remained unchanged and ready throughout.

```sh
E2E_BASE_URL=http://127.0.0.1:18410 E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:latency
```

All 22 tests passed in 21.8s, with retries and framework traces disabled. The real browser
scenarios use native controls and page-delivered authoritative peer state. The harness delays
original frames through bounded per-direction FIFO queues and checks chronology, overflow,
release witnesses and public-only diagnostics.

| Profile | Observations | Median | Maximum | Budget exceedances |
| --- | ---: | ---: | ---: | ---: |
| Unshaped local | 20 | 37.73ms | 46.94ms | 0 |
| 50ms each way, deterministic ±10ms jitter | 20 | 129.13ms | 139.41ms | 0 |

The shaped scenario retains the existing 250ms action/release budget. This is one Chromium
loopback/browser-route run, not a WAN-support claim, a general latency tail guarantee or a
supported-browser decision. Human feel and broader browser/network choices remain outstanding
in the [backlog](../../BACKLOG.md). The [scenario](../../e2e/scenarios/websocket-latency.md)
defines the method and limits.

The isolated project was removed. Raw allowed evidence is retained under ignored
`target/quality-review/baseline32-latency-*`; no credentials or raw framework traces are retained.
