# ADR 0001: Rust transport with Flow engineering conventions

Status: accepted for the initial scaffold.

## Context and decision

Grenadiers reuses the concepts of the local Flow repository (`~/Sources/flow`,
<https://github.com/liveweird/flow>), replacing Kotlin/JVM with Rust. It keeps the monorepo
boundaries, contract-first API workflow, test layers, standards, and topic documentation.
Flow's product features and accumulated exceptions are not part of this foundation.

| Flow | Grenadiers |
| --- | --- |
| Gradle modules/catalog | Cargo workspace and workspace dependencies |
| Ktor, Netty, coroutines | Axum, Hyper, Tokio |
| Ktor plugins | Tower/tower-http middleware |
| kotlinx.serialization | Serde and serde_json |
| Explicit application wiring | Typed application state and constructors |
| Exposed R2DBC, Flyway | SQLx PostgreSQL; migrations when a schema exists |
| SLF4J/Logback | tracing/tracing-subscriber |
| detekt | rustfmt, Clippy with warnings denied |
| Kover | cargo-llvm-cov line gate |
| kotlin.test/testApplication | Rust tests, Tower oneshot, real ephemeral WebSocket sockets |
| OpenAPI and generated TS | OpenAPI HTTP plus JSON Schema realtime and generated TS |
| React/Vite/Mantine/i18next | Same frontend families, with only scaffold dependencies |
| Vitest/Testing Library/Playwright | Same testing layers and scenario parity |
| Dependency verification/audits | Cargo/npm locks, cargo-deny, npm audit |

## Consequences

These are equivalents of responsibility, not identical tools. Stable Rust coverage does
not provide Kover-equivalent branch coverage here. Cargo lock checksums do not authenticate
publishers. PostgreSQL Testcontainers will arrive with persistence tests, not before a
schema exists. Local Kubernetes manifests are included for deployment verification; production
cluster provisioning, mail, admin/auth features, worker processes, charts, and Flow's Jira
domain are excluded. Mature-suite performance tweaks and coverage percentages are
not copied without measurement.

Source references: Flow `CLAUDE.md`, `.claude/docs/testing.md`,
`.claude/docs/dependency-reproducibility.md`, `web/package.json`, `scripts/gates.sh`,
`.github/workflows/ci.yml`, and `e2e/README.md`.

Primary library references: [Axum](https://docs.rs/axum/latest/axum/),
[SQLx](https://docs.rs/sqlx/latest/sqlx/),
[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), and
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny).
