# Grenadiers

A browser-based multiplayer game inspired by classic Bomberman/Dynablaster.
The first playable milestone will support two human players. Bots and larger matches follow later.

This is the **engineering scaffold**, not a playable game. It provides a React shell, HTTP probes,
and a versioned WebSocket diagnostic exchange. Rooms, controls, maps, bombs, authentication,
matchmaking, stats, and simulation remain unimplemented. The [draft game specification](docs/game/README.md)
describes the classic reference and proposed rules. The
[implementation and release plan](docs/game/implementation-plan.md) breaks delivery into
verified foundation, playable duel, classic matches, and resilient playtest releases.

## Stack and layout

| Directory | Responsibility |
| --- | --- |
| `core/` | Rust wire types; future deterministic game rules, independent of I/O |
| `server/` | Axum/Tokio HTTP and WebSocket adapters, tracing, optional SQLx readiness |
| `web/` | React, TypeScript, Vite, Mantine, EN/PL localization |
| `api/` | Authoritative OpenAPI and realtime JSON Schema contracts |
| `api-guidelines/` | Stable API rules and Spectral configuration |
| `e2e/` | Standalone Playwright blackbox tests and matching scenario documents |
| `docs/` | Architecture decisions, conventions, testing, and operational guidance |
| `k8s/` | Single-replica local Kubernetes deployment and Kustomize composition |
| `scripts/` | Local gates shared with CI |

See [architecture](docs/architecture.md), [Flow equivalents](docs/decisions/0001-stack.md),
[testing](docs/testing.md), [deployment](docs/deployment.md), and [contributing](CONTRIBUTING.md).
Product versioning and candidate checks are documented in [releases](docs/releases.md).

## Prerequisites

Use `mise install` for Node **24.21.0** and Rust **1.99.1**, then `mise exec -- <command>`
or activate mise in your shell. `rust-toolchain.toml` also selects the Rust toolchain.
Docker is optional for local development and required for the complete Compose stack.

Cargo and npm dependencies are resolved in repository lockfiles. Normal builds use locked
dependencies; review lockfile changes together with dependency updates.
See [verification status](docs/verification.md) for checks actually performed.

```sh
npm --prefix web ci
npm --prefix e2e ci
cargo fetch --locked
```

## Run locally

Run these in separate terminals:

```sh
cargo run --locked -p grenadiers-server
npm --prefix web run dev
```

Open <http://localhost:5177>. Vite proxies `/api`, including WebSockets, to `127.0.0.1:8085`.
The page's Connect/Ping controls verify connectivity; they do not join a match or reserve a player slot.

For a single-origin production-style local stack:

```sh
docker compose up --build
```

Open <http://localhost:8085>. The server serves the built SPA and API. Stop with
`docker compose down`; add `--volumes` only when you intend to delete local database data.

For local Kubernetes, use the manifests under `k8s/` after building and loading the application
image. See [deployment instructions](docs/deployment.md) and the
[verification record](docs/verification.md) for runtime results.

## Optional PostgreSQL

The scaffold needs no database. Start one only when working on durable persistence:

```sh
docker compose --profile persistence up -d postgres
export DATABASE_URL=postgres://grenadiers:grenadiers_local_only@127.0.0.1:5436/grenadiers
cargo run --locked -p grenadiers-server
```

Without `DATABASE_URL`, readiness has no database dependency. With it set, database failure makes
`/api/v1/ready` return 503; `/api/v1/health` remains a liveness probe. There are no tables or
migrations yet. PostgreSQL is intended for users/results, not the per-tick game loop.

## Checks

```sh
scripts/gates.sh all           # Rust, contracts, web, and E2E static gates
scripts/gates.sh server        # rustfmt, Clippy, tests, build (locked)
scripts/gates.sh web           # contract drift, lint, dead code, coverage, build
scripts/gates.sh e2e           # lint, typecheck, dead code, scenario parity
scripts/gates.sh browser       # Playwright; requires the running composed stack
scripts/gates.sh k8s           # Kustomize render + strict Kubernetes schema validation
scripts/gates.sh security      # cargo-deny and npm dependency audits
scripts/gates.sh coverage      # cargo-llvm-cov line coverage floor
```

Generate frontend contracts with `npm --prefix web run gen:api` and
`npm --prefix web run gen:realtime`; commit generated files with contract changes.
Coverage tooling installation and test conventions are in [docs/testing.md](docs/testing.md).

## Configuration

`.env.example` documents development defaults. The Rust process reads environment variables;
it does not automatically load `.env` files. Compose reads `.env` for interpolation.
Host ports **8085**, **5177**, and **5436** avoid Flow's defaults.
See [security and configuration](docs/security.md) before exposing the scaffold beyond localhost.

## Contributor guidance

The original `AGENTS.md` still describes the empty workspace. It is preserved under the
initial instruction not to modify an existing file; use `CONTRIBUTING.md` and `docs/` for
current commands and conventions.
