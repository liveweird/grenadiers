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

The diagnostic and gameplay WebSockets bound message/frame size, connection count, idle time,
queues, commands, and sends.
Malformed text messages yield a typed error and policy close. Control frames remain protocol
control frames. The diagnostic envelope remains v1; authenticated gameplay requires v5.

Resume capabilities are independent 32-byte random values encoded as lowercase hexadecimal. They
exist only in the room owner and the admitted browser tab, remain stable across restore/takeover,
and are never placed in URLs, shared room state, logs, replay journals, local/session storage, or
database fields. Browser retries obtain a fresh ordinary session before presenting the capability.
Explicit leave clears it only after acknowledgement; `resumeRejected`, replacement, and exhausted
retry terminate recovery. Active-v5 Playwright traces are disabled because raw framework traces can
capture personal admission frames before redaction. Screenshots, public reports, and bounded
redacted summaries remain suitable artifacts.

`DATABASE_URL` opts into SQLx PostgreSQL readiness. Blank/unset means no database requirement;
a configured database outage must not make liveness fail. PostgreSQL migrations will be
append-only when introduced; runtime application code will not create tables opportunistically.

`.env.example` contains local demo values only. The Rust executable does not load dotenv files.
For Compose app+database use a network URL with hostname `postgres:5432`; for a host Rust process
use `127.0.0.1:5436`. Never reuse the local demo password in a deployment.

Containers run without root, writable root filesystems, Linux capabilities, or elevated privileges.
CI builds one Compose image, runs default, pickup, complete-match, and reconnect browser acceptance
against it, then scans that image. Publishing and deploying the application by immutable registry
digest remains outstanding; exact release tags alone do not provide immutable provenance.

Rooms, capabilities, and private replay journals are in-memory and process-local. Restarting or
routing a reconnect attempt to another replica loses the room; multi-replica capability routing
and durable recovery are not implemented.
