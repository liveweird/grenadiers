# Security and configuration

This is a localhost scaffold with no identity or match admission. Origin checks are a browser
boundary, not authentication; non-browser clients can supply Origin themselves. Before public
hosting, define authentication/admission, per-client rate limits, TLS, proxy trust, and abuse policy.

`SERVER_ADDR` defaults to `127.0.0.1:8085`. Compose binds its published ports to loopback only.
`ALLOWED_ORIGINS` is a comma-separated exact allowlist; no wildcard or substring matching.
Vite connections originate at port 5177; built SPA connections originate at port 8085.
Use deployment-specific origins outside local development.

The diagnostic WebSocket bounds message/frame size, connection count, idle time, and sends.
Malformed text messages yield a typed error and policy close. Control frames remain protocol
control frames. The JSON envelope's version is announced in welcome; breaking changes require
a versioned contract. Never place credentials in URLs or log message bodies or database URLs.

`DATABASE_URL` opts into SQLx PostgreSQL readiness. Blank/unset means no database requirement;
a configured database outage must not make liveness fail. PostgreSQL migrations will be
append-only when introduced; runtime application code will not create tables opportunistically.

`.env.example` contains local demo values only. The Rust executable does not load dotenv files.
For Compose app+database use a network URL with hostname `postgres:5432`; for a host Rust process
use `127.0.0.1:5436`. Never reuse the local demo password in a deployment.

Containers run without root, writable root filesystems, Linux capabilities, or elevated privileges.
Image digest pinning and end-to-end container validation remain outstanding until registry and
Docker access are available; exact release tags alone do not provide immutable provenance.
