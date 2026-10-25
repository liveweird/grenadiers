# Contributing

Read [README.md](README.md), [architecture](docs/architecture.md), and the topic documents
relevant to your change. Executable configuration wins when documentation disagrees; correct
both in the same change. This document and `docs/` are the current reference for the
implemented stack.

## Conventions

- Use four spaces in Rust and two in TypeScript, JSON, YAML, and Markdown. Rust formatting
  comes from rustfmt; TypeScript follows ESLint and the surrounding style.
- Rust modules/functions use `snake_case`, types use `PascalCase`. React components use
  `PascalCase`; hooks use `useCamelCase`. JSON fields are `camelCase`.
- Keep source files at most 400 lines. Split by responsibility rather than moving complexity
  into a helper with no domain meaning. The repository gate checks this ceiling for `.rs`,
  `.ts`, and `.tsx` files under `core/src`, `server/src`, `web/src`, `e2e/tests`, and every
  immediate `e2e/*-tests` directory, including nested source files. It skips directories named
  `node_modules`, `dist`, `coverage`, `target`, `playwright-report`, or `test-results`, plus
  generated `*schema.ts` and `*realtime.ts` files.
- Keep `core/` independent of networking, async runtimes, storage, and UI libraries.
- Fix lint warnings; do not add baselines or lower coverage floors to accommodate new code.
  Any deliberate rule exception needs a local explanation and review.
- All visible UI text belongs in EN/PL locale resources; add matching keys together.

## Changes and verification

### Local-first verification

This is a single-maintainer project. Successful local checks are as authoritative as remote
checks for the same tested source and environment. Run remote checks when they add distinct
coverage, such as Linux behavior not exercised locally; do not require duplicate full suites
for branch pushes, pull requests, merges, and release tags.

Run independent checks and isolated test groups in parallel. Reuse dependency/build caches and
verified artifacts, while retaining assertions, coverage floors, source identity and failure
reporting. Keep latency measurements isolated from competing test workloads.

Record the tested commit, commands, outcomes, environment and deployed image digest concisely.
Repeat checks for relevant changes, failures or an identified coverage gap. Do not add repeated
approval, evidence-generation or review cycles to routine changes. See [testing](docs/testing.md)
and [release verification](docs/releases.md#release-verification-policy).

Update contracts first for transport changes. Regenerate types, add runtime conformance and
behavioral tests, and update API guidance in the same change. Add a scenario document with
verbatim test titles for every Playwright spec. Never edit generated types directly.

Run the relevant `scripts/gates.sh` targets before submitting. The server gate tests both debug
and release profiles; required state changes must execute outside assertions. Use ephemeral listeners and
independent test state; never rely on test order or shared developer database contents.
Update the canonical topic document when behavior or operational requirements change.

Use Conventional Commits, following Flow's convention: `feat:`, `fix:`, `docs:`, `chore:`,
`test:`, or `ci:`, optionally scoped (`fix(server): reject invalid input`). No local Git
history existed at scaffold creation; this is an adopted convention.

Pull requests explain the concrete problem and resulting behavior, link relevant issues,
list verification performed and remaining limitations, and include screenshots for visible
UI changes. Keep unrelated refactors separate. Do not commit secrets, generated build output,
or credentials. Commit dependency lockfiles and review updates intentionally.
