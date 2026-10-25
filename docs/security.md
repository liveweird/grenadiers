# Security and configuration

The game uses short-lived anonymous single-use session tokens for gameplay authentication and a
separate private capability for restoring one room seat. Neither establishes a durable user
identity. Origin checks remain a browser boundary; non-browser clients can supply Origin
themselves. Before public hosting, define durable identity policy if needed, TLS, proxy trust,
cross-process routing, and abuse controls.

`SERVER_ADDR` defaults to `127.0.0.1:8085`. Compose binds its published ports to loopback only.
`ALLOWED_ORIGINS` is a comma-separated exact allowlist; no wildcard or substring matching.
Vite connections originate at port 5177; built SPA connections originate at port 8085.
Use deployment-specific origins outside local development.

The diagnostic and gameplay WebSockets use separate connection and upgrade-admission budgets.
Both apply the same strict Origin validation and bound message/frame size, inbound rate, idle time,
write buffering, commands, and sends. Diagnostic connections are capped at 16; gameplay
connections are capped at 128.
Room create/join/resume operations have separate process-global budgets (32 burst and eight per
second each). A correlated operation throttle preserves authentication; per-socket inbound floods
still close with policy code 1008. Peer-keyed quotas and denial telemetry remain hosting work.
Malformed text messages yield a typed error and policy close. Control frames remain protocol
control frames. The diagnostic envelope remains v1; authenticated gameplay requires v11;
older versions are rejected before credential lookup.

Resume capabilities are independent 32-byte random values encoded as lowercase hexadecimal. They
exist only in the room owner and the admitted browser tab, remain stable across restore/takeover,
and are never placed in URLs, shared room state, logs, replay journals, local/session storage, or
database fields. Transport reconnects obtain a fresh ordinary session before presenting the
capability; a correlated resume throttle uses bounded retries on that authenticated socket.
All recovery shares one ten-second client deadline. See [browser recovery](game/reconnect-browser.md).
Explicit leave clears it only after acknowledgement; `resumeRejected`, replacement, and exhausted
retry terminate recovery. Playwright traces are disabled in every configuration because raw framework traces can
capture personal admission frames before redaction. Screenshots, public reports, and bounded
redacted summaries remain suitable artifacts.

`DATABASE_URL` opts into SQLx PostgreSQL readiness. Blank/unset means no database requirement;
a configured database outage must not make liveness fail. PostgreSQL migrations will be
append-only when introduced; runtime application code will not create tables opportunistically.

`.env.example` contains local demo values only. The Rust executable does not load dotenv files.
For Compose app+database use a network URL with hostname `postgres:5432`; for a host Rust process
use `127.0.0.1:5436`. Never reuse the local demo password in a deployment.

The application container runs without root, a writable root filesystem, Linux capabilities, or
elevated privileges. The optional Compose PostgreSQL service uses the upstream image defaults.
The manual `checks` workflow builds one Compose image in `browser-image` and scans it there, then
the `browser` jobs run the default, classic, and per-item suites against that image.
Publishing and deploying the application by immutable registry digest remains outstanding; exact release tags alone do not provide immutable provenance.

Rooms, capabilities, and private replay journals are in-memory and process-local. Restarting or
routing a reconnect attempt to another replica loses the room; multi-replica capability routing
and durable recovery are not implemented.
