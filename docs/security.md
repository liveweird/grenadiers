# Security and configuration

The game uses short-lived anonymous single-use session tokens for gameplay authentication and a
separate private capability for restoring one room seat. Neither establishes a durable user
identity. Origin checks remain a browser boundary; non-browser clients can supply Origin
themselves. Before public hosting, define durable identity policy if needed, TLS, proxy trust,
cross-process routing, and abuse controls.

## Startup configuration

Startup reads and validates configuration once before binding the listener. The executable does
not load dotenv files. These are the intentional defaults; Compose binds its published ports to
loopback only, while its process listens on the container interface.

| Setting | Standalone executable default | Compose/Kubernetes default |
| --- | --- | --- |
| `SERVER_ADDR` | `127.0.0.1:8085` | `0.0.0.0:8085` |
| `WEB_DIST_DIR` | unset (API only) | `/app/web/dist` |
| `RUST_LOG` | `info` | `grenadiers_server=info,tower_http=info` |
| `ALLOWED_ORIGINS` | localhost and 127.0.0.1 on ports 5177 and 8085 | localhost and 127.0.0.1 on port 8085 only |
| `TRUSTED_PROXY_IPS` | empty | empty |
| `DATABASE_URL` | unset | empty (database readiness disabled) |
| `GAME_ROUND_SEED_FILE` | unset (fresh OS randomness) | unset (fresh OS randomness) |

`ALLOWED_ORIGINS` is a comma-separated exact allowlist; no wildcard or substring matching.
Vite connections originate at port 5177; built SPA connections originate at port 8085. An
explicit value containing no origins remains fail-closed and logs a startup warning. Use
deployment-specific origins outside local development. Invalid addresses and non-Unicode textual
state settings fail startup without including their values in diagnostics. Invalid or non-Unicode
`RUST_LOG` values retain the standalone `info` fallback. Path settings preserve non-Unicode OS
paths.

## Response security

Every response sets `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, and
`Referrer-Policy: no-referrer`, including API errors and static/SPA fallbacks. HTML and unhashed
static files use `Cache-Control: no-cache`; successful non-HTML files with the configured
eight-character Vite content-hash suffix under `/assets/` use a one-year immutable cache policy.
API and WebSocket responses retain their stricter `no-store` policy. Content Security Policy
is enforced on every response, including HTML HEAD and conditional responses. Scripts and
connections are restricted to the same origin; inline scripts, string compilation, workers,
objects, base URLs, form submissions and framing are denied. Images allow self and data URLs;
fonts and media fetches are denied. `style-src 'self' 'unsafe-inline'` deliberately supports
Mantine's style elements and attributes; nonce-only styling remains separate work.

Live schema validation is precompiled. Pixi's shared `pixi.js/unsafe-eval` compatibility import
uses static implementations despite its name; the script policy does not allow `unsafe-eval`.
Diagnostic pages use corresponding meta policies; `frame-ancestors` requires an HTTP header.
HTTP/WS and HTTPS/WSS same-origin behavior is verified in Chromium. Other browsers still need
acceptance before claiming support. The policy never derives permissions from Host, Origin or
forwarding headers. The incoming Origin allowlist remains a separate transport boundary.
See [CSP browser acceptance](../e2e/README.md#content-security-policy-acceptance).

The diagnostic and gameplay WebSockets use separate connection and upgrade-admission budgets.
Both apply the same strict Origin validation and bound message/frame size, inbound rate, idle time,
write buffering, commands, and sends. Diagnostic connections are capped at 16; gameplay
connections are capped at 128.
Room create/join/resume operations have separate process-global budgets (32 burst and eight per
second each). A correlated operation throttle preserves authentication; per-socket inbound floods
still close with policy code 1008. Peer admission below adds independent budgets without replacing
these global backstops.
Malformed text messages yield a typed error and policy close. Control frames remain protocol
control frames. The diagnostic envelope remains v1; authenticated gameplay requires v11;
older versions are rejected before credential lookup.

## Peer identity and admission

The transport address identifies an admission peer, not a durable user. IPv4-mapped IPv6 is
normalized to IPv4; budgets group IPv4 by address and native IPv6 by /64. Source ports, Origin,
credentials and room identifiers never determine this key. Missing transport metadata rejects
admission with a no-store 503 rather than sharing an unknown-peer budget.

`TRUSTED_PROXY_IPS` defaults empty: no proxy is trusted, including loopback. Configure at most
32 exact immediate proxy IP literals, comma-separated without whitespace, within 2,048 bytes.
Hostnames, CIDRs, ports and malformed/non-Unicode values fail startup. Full normalized addresses
determine proxy trust before /64 budget grouping. Untrusted forwarding headers are ignored.

A trusted immediate proxy must **overwrite** `X-Forwarded-For` with exactly one client IP.
Missing, duplicate, empty, oversized, non-UTF8, comma-separated, quoted, port-bearing or scoped
values reject with a no-store 400. Alternative identity headers do not establish trust. Multi-hop
chains require a separate explicit design; never trust every private network automatically.

| Peer budget | Default |
| --- | --- |
| Mint, gameplay upgrade, create, join, resume | Separate burst 8, refill 2/second each |
| Diagnostic upgrade | Burst 4, refill 1/second |
| Outstanding issued plus claimed sessions per minting peer | 16 |
| Gameplay sockets, including upgrade/unauthenticated lifetime | 8 |
| Diagnostic sockets | 4 |
| Concurrent room-admission attempts | 4 |

These are engineering defaults, not measured public-hosting capacity. Peers sharing NAT or a
local proxy share budgets. Session claiming does not release its minting quota; expiry or lease
drop does. A credential can be claimed from a different peer: address quotas do not change bearer
authentication. Socket and room-attempt permits release on failure, cancellation and disconnect.
Room-attempt permits remain held through the response attempt, so stalled responses stay within
the concurrent-handler bound.

Gameplay and diagnostics have independent tables capped at 4,096 peer keys each. Only entries
without live permits and inactive for at least 60 seconds can be evicted; attempts and permit
release refresh activity. Sweeping is bounded and at most once per second. Full tracking rejects
new keys with 503 while existing keys remain eligible. Peer checks precede global token use.
HTTP rate and per-peer outstanding-session denials use 429 and `Retry-After: 1`. Socket,
tracking and global registry capacity, or unavailable identity, use 503. WebSocket
room throttles remain correlated and non-closing, preserving bounded browser resume recovery.

Denial counters use finite operation/reason labels and bounded aggregate tracing summaries.
Addresses, headers, credentials, room IDs and arbitrary error strings are not telemetry labels.
Restart clears process-local counters and quotas. Public exposure still requires actual proxy
addresses/overwrite behavior, restricted backend reachability, TLS and ingress connection/header
limits; IP budgets do not establish authentication or solve distributed attacks.

## Credentials and persistence

Resume capabilities are independent 32-byte random values encoded as lowercase hexadecimal. They
exist only in the room owner and the admitted browser tab, remain stable across restore/takeover,
and are never placed in URLs, shared room state, logs, replay journals, local/session storage, or
database fields. Transport reconnects obtain a fresh ordinary session before presenting the
capability; a correlated resume throttle uses bounded retries on that authenticated socket.
All recovery shares one ten-second client deadline. See [browser recovery](game/reconnect-browser.md).
A leaked resume capability together with its room identifier lets another authenticated holder
restore or take over that seat. Anonymous session issuance does not establish durable ownership;
peer quotas do not prevent use of a stolen capability. Recoverable rotation needs a separate
protocol design so a lost response does not lock out the legitimate holder.

Explicit leave clears it only after acknowledgement; `resumeRejected`, replacement, and exhausted
retry terminate recovery. Playwright traces are disabled in every configuration because raw framework traces can
capture personal admission frames before redaction. Screenshots, public reports, and bounded
redacted summaries remain suitable artifacts.

`DATABASE_URL` opts into SQLx PostgreSQL readiness. Blank/unset means no database requirement;
a configured database outage must not make liveness fail. PostgreSQL migrations will be
append-only when introduced; runtime application code will not create tables opportunistically.

`.env.example` contains local demo values only.
For Compose app+database use a network URL with hostname `postgres:5432`; for a host Rust process
use `127.0.0.1:5436`. Never reuse the local demo password in a deployment.

The application container runs without root, a writable root filesystem, Linux capabilities, or
elevated privileges. Kubernetes also disables service-account token automounting and uses the
runtime-default seccomp profile; the namespace applies restricted Pod Security enforcement,
auditing, and warnings. The optional Compose PostgreSQL service uses the upstream image defaults.
The manual `checks` workflow builds one Compose image in `browser-image` and scans it there, then
the `browser` jobs run the default, classic, and per-item suites against that image.
The checked-in mutable image tag is for local developer clusters. Other environments supply their
exact accepted image reference; immutable registry digests identify exact image content, while
release tags alone do not provide immutable provenance.

Rooms, capabilities, and private replay journals are in-memory and process-local. Restarting or
routing a reconnect attempt to another replica loses the room; multi-replica capability routing
and durable recovery are not implemented.
