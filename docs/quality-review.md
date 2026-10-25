# Quality review: assessment and remediation

This responds to Claude's 25 October 2026 review of `2f2b680`. All ten review documents were
read. Findings were checked against source, imports, configuration and tests; cached coverage
figures remain historical observations. Unmeasured risks below are not confirmed defects.
The original review remains unchanged in the local review folder.

## First implementation batch

- **BE-02 / SEC-02 / API-04:** isolate diagnostic connection permits and upgrade-rate tokens
  from gameplay; reuse strict Origin checks and inbound limits; bound writes and sends;
  apply `no-store` to upgrades/rejections; drain both pools under the same shutdown deadline.
  Update OpenAPI/generated types and add real-socket isolation/flood regressions.
- **FE-05:** catch render/lifecycle failures outside the application/provider subtree, show
  an accessible EN/PL fallback, and allow explicit retry or reload. Recovery does not promise
  preservation of an in-memory room capability.
- **TL-01:** run workflow contracts in the repository gate and the real coverage-runner
  regression suite before `server-parallel`. The latter needs `cargo-llvm-cov`, so placing it
  in every repository/web check would introduce an inappropriate tooling dependency.
- **Additional finding:** installed Vite/happy-dom differed from the pinned manifest during
  verification. Restore dependencies with `npm ci`; check installed direct versions before
  web/E2E-static/browser/atlas gates. The interrupted stale-tool coverage run is not accepted.
- **DOC-06, partial:** add the [contributor index](README.md). The broader history split and
  item-development guide remain follow-up work.

## Subsequent HTTP contract repair

**API-02/03 and HTTP examples from API-07:** unknown API paths return no-store problem 404s;
unsupported methods return problem 405s with route-specific `Allow`. WebSocket HEAD rejects
with problem representation headers and no body, while health/readiness HEAD still succeeds.
Both WebSocket routes apply no-store even to framework-generated errors and successful upgrades.
Fallback statuses are documented separately from supported OpenAPI operations; shared cache/retry
headers and problem examples match the implementation. Game-message descriptions and close-code
documentation remain separate follow-up work.

## Finding dispositions and proposed solutions

“Agree” accepts the observed issue, not every implementation proposed in the review.
Related IDs are grouped to avoid treating one architectural problem as several independent fixes.

| IDs | Disposition | Solution / next action |
| --- | --- | --- |
| DOC-01/02/04/05/07/08/09/14 | Agree: contributor guidance is buried in repeated release evidence. | Separate current procedures from history; retain old acceptance facts unchanged, use one release summary, and replace repeated per-item testing prose with a linked table. Adopt an Unreleased changelog section for future work. |
| DOC-03 | Agree on organization, not the claimed stale checkpoint status. | Move dated release narrative out of backlog; its explicitly historical “unreleased” state must not be rewritten as a current fact. |
| DOC-06/10/11/13 | Agree: discovery and document structure need improvement. | Start with the new index; add an item how-to, ADR index and retention ADR, consistent item headings, and implemented/future rule markers. Move archived protocol guides with link checks. |
| DOC-12 | Factually stale, deliberately preserved. | Keep `AGENTS.md` untouched under the user's instruction; point readers to current conventions and the index. |
| API-01, CX-01/02, BE-07, FE-01, TS-08 | Agree on duplication; blanket deletion is unsafe. | Define live-wire and replay retention separately. Extract the v8/v9 helpers used by v11, remove demonstrably unused client UI, then retire obsolete adapters/tests under the stated policy. Parameterize retained frame-bound tests. |
| API-02/03 | Agree; HTTP repair implemented. | Problem 404/405 fallbacks, route-specific `Allow`, shared cache/retry headers and API/SPA/HEAD tests are in place; fallback responses are separate from supported operation responses. |
| API-04, BE-02, SEC-02 | Agree; first batch implements this. | Independent diagnostic permits/rate budget and shared strict transport guards, with saturation, Origin, flood, cache-header and shutdown tests. |
| API-05 | Optional improvement, not a demonstrated defect. | Add stable Problem codes only when a consumer needs distinctions beyond HTTP status. Do not unify diagnostic and gameplay error vocabularies merely for symmetry. |
| API-06/07/08 | Agree on missing documentation/enforcement. | Document actual close codes; describe live messages and add valid examples. Enforce useful API conventions incrementally with positive/negative lint fixtures. |
| API-09 | Agree on duplicated contract inventories. | Share contract discovery with explicit filename/version validation; unexpected files must not silently become supported contracts. |
| API-10, CX-09 | Coverage question, not yet established as a defect. | Enumerate live message variants against schema conformance tests, then add any missing cases. |
| BE-01 | Confirmed cost, intentional documented behavior. | Decide whether production replay recording has an operational consumer. If not, feature-gate allocation/recording while retaining replay acceptance tests; document memory effects. |
| BE-03, SEC-01 | Agree: public-hosting blocker. | Design trusted peer/proxy identity, bounded keyed mint/upgrade/room budgets, per-client outstanding limits and global backstops together. Keep denial telemetry credential-free. |
| BE-04 | Agree on repeated mechanics. | Share the bucket implementation and timeout definitions while preserving distinct budget instances and clock behavior. Avoid a generic clock abstraction unless it improves actual callers. |
| BE-05, TS-14 | Agree for active startup/configuration paths. | Centralize typed parsing and defaults; test invalid configuration and empty-origin behavior without logging credentials. Revisit archived low-coverage code with the retention decision. |
| BE-06, CX-06 | Agree on stale exemptions; removals need usage checks. | Remove definition-only `restore_punch_cooldown`; gate test-only restoration/observation helpers; retain production cooldown getters used transitively by v11. Treat admission-argument refactoring separately. |
| BE-08, CX-04/05 | Agree on test wiring/helper repetition. | Move simulation test-module declarations into a test aggregator; consolidate genuinely equivalent fixtures; extend source-size discovery to item E2E directories. Keep responsibility-based production modules. |
| BE-09, CX-07 | Mixed: maintainability concerns, not proven reachable panics. | Replace transport-state assumptions with typed authenticated commands where useful; split orchestration around domain operations. Audit narrowing conversions before changing public APIs. |
| BE-10, TL-04 | Agree on rationale/update-management gaps. | Document deterministic/entropy dependency constraints, group compatible updates, and review incompatible upgrades deliberately. Exact pins do not justify indefinitely suppressing upgrade/security notices. |
| BE-11, TS-09/10 | Mostly intentional tests/benchmarks. | Use virtual time where it preserves the real socket deadline being tested. Keep performance tests separate from ordinary suites. Do not replace useful waits or ignored benchmarks solely to improve counts. |
| BE-12/13 | Plausible shutdown/latency concerns. | Reproduce shutdown with a non-reading client; measure registry-lock work before moving bounded construction/entropy outside it. No demonstrated hang yet. |
| CX-03 | Agree on cross-layer item duplication, not the proposed enum choice. | Define one domain item specification and derive configurable wire catalogs from each schema's `ItemId`, not `PickupKind`; preserve archived subsets. Parameterize configuration scaffolding independently. |
| CX-08, TS-06, TL-05 | Agree on E2E/configuration sprawl. | Introduce a config factory, fixture parameter and shared behavioral helpers; prove representative item and privacy suites retain their distinct setup/assertions before deleting wrappers. |
| FE-02 | Agree: no bundle budget. | Measure a fresh production build, set an honest budget, then test CSS/import reductions. Do not adopt an already-failing target or assume a specific reduction without measuring it. |
| FE-03 | Partly disagree; see corrections below. | Typed i18next consolidation is reasonable cleanup. Add plural forms where grammar needs them, not for multiplication/count notation. |
| FE-04 | Mechanism confirmed; performance cost unmeasured. | Profile 20 Hz gameplay and commit durations first. Change arena/HUD subscriptions only for a demonstrated budget problem. |
| FE-05 | Agree; first batch implements this. | Localized accessible error recovery with explicit actions and component/parity tests. |
| FE-06 | Type-hygiene observation, not a broad defect. | Replace the isolated double cast when touching binding construction; no repository-wide type rewrite. |
| FE-07, SEC-04/10 | Agree on hardening/cache gaps; suggested CSP is incompatible as written. | Add basic response hardening and correct HTML/hashed-asset cache policy; precompile validators before enforcing a restrictive CSP. Verify Mantine/Pixi and connection policy in real browsers. |
| FE-08 | Selector counts do not describe the current web tests. | Current web tests use role queries rather than test IDs. Assess E2E selectors in their own scope; a selector ratio alone is not an accessibility defect. Retain actual accessibility audits. |
| FE-09/10 | Agree, lower-priority consistency work. | Consolidate copied scene mocks, preserving specialized renderer/atlas mocks; introduce semantic color tokens incrementally. |
| TS-01/02 | Agree on cost; sampling is a policy change. | Measure a structured fast feedback suite while retaining exhaustive acceptance. Compare fault detection before proposing a new cadence; keep current assertions and exhaustive coverage meanwhile. |
| TS-03/04 | Agree on denominator hygiene; thresholds require recalibration. | Exclude narrowly identified test support, collect fresh coverage, then choose floors. Distinguish `all` from optional full/tool-dependent checks; do not require coverage tooling for every quick gate. |
| TS-05, TL-01/02/03, SEC-05 | Partly agree: orphan callers and gate discoverability need work. | Wire the orphan script tests now; audit manual diagnostic-suite coverage and document aggregate scope. Preserve local-first verification; duplicated automatic remote gates are not a default requirement. |
| TS-07 | Agree: global title sets miss duplicates/mappings. | Reject duplicate titles and validate documented scenario-to-spec mappings; support the actual title syntax through parsing or explicit metadata. |
| TS-11/12 | Factual suggestions, not established missing-risk coverage. | Target parser mutation/property checks and simultaneous competing-resume tests at explicit invariants; existing concurrency tests already use synchronized claimants. |
| TS-13 | Disagree with sibling-test filenames as a coverage measure. | Use behavior and coverage of actual call paths. Indirectly covered components do not require duplicate tests solely to match filenames. |
| TL-06 | Agree on release bookkeeping cost. | Reduce duplicated release facts first; then automate version/changelog updates and applicable checks, without introducing another mandatory evidence pipeline. |
| TL-07 | Root fixture clutter is real; local checkpoint claim is historical. | Consolidate Compose fixtures. Local scratch/worktrees/images were already cleaned separately; retained history is compressed and recoverable. |
| TL-08/09 | Useful incremental guardrails, not all immediate defects. | Add selected shell/workflow checks with documented tool versions; evaluate stricter TS/lint rules by actionable findings rather than enabling everything at once. |
| TL-10/11 | Agree on visibility/cache opportunities, not mandatory policy changes. | Review moderate advisories by impact; reuse verified tool/build caches. Keep security failures visible and pin tool provenance. |
| TL-12 | Agree on build-cache/reproducibility tradeoffs. | Add dependency caching and OCI metadata. Replace unbounded package upgrades with a documented base-image/security-refresh policy; simply dropping upgrades can retain known vulnerabilities. |
| TL-13/14/15/16 | Mixed low-priority tooling/documentation concerns. | Scope the peer-dependency workaround, document supported Python/tooling prerequisites, name gate commands in the PR template, and retain existing pin consistency checks. |
| SEC-03 | Agree with the defect, not a server-only fix. | Pair reply-only throttling with bounded same-socket client resume retries, increasing sequences and preserved capability/cancellation fencing. Keep per-socket flood closure. |
| SEC-06/07 | Documented capability tradeoff; missing denial visibility. | Explain capability-leak/takeover consequences and add redacted counters. Rotation requires a recoverable protocol design, not a silent behavior change. |
| SEC-08/09/11 | Sensible deployment hardening; some risks remain unmeasured. | Test container/pod limits and security-context requirements; define proxy/header/idle/connection limits before public hosting. NetworkPolicy needs an actual network/DB model. |

## Corrections that change the implementation plan

1. `web/src/i18next.d.ts` uses `typeof en`, and `i18n.ts` includes both `app` and `game`.
   Game resources already have types. The raw JSON interpolation path is separate design debt.
2. `web/src/game/client/client.ts` clears pending admission on an error and schedules recovery
   on closure. Changing only the server's `RateLimited` close behavior would strand auto-resume.
3. Ajv compiles live validators at module load. A restrictive `default-src 'self'` policy needs
   compatible script/style handling; lazily importing diagnostics does not fix gameplay codegen.
4. `ItemId` is the configurable catalog; `PickupKind` also contains basic upgrades. Generated
   TypeScript aliases are not runtime enum objects. Old validators intentionally have smaller catalogs.
5. Exhaustive acceptance is an existing requirement in [testing](testing.md#verification-policy).
   Oil-specific placement filtering and mask-dependent random bounds invalidate a claim that
   sampled execution is automatically proof-equivalent. The test cost still deserves reduction.
6. Historical checkpoint status/counts stay historical. Move them for usability, preserving
   source attribution. Raise coverage floors only after measuring the cleaned denominator.
7. Repeated automatic remote checks conflict with the current single-maintainer policy unless
   they supply distinct coverage. Dependabot validation can be designed without restoring the
   old duplicate branch/PR/merge/tag pipeline.

## Next bounded batches

1. Complete close-code/live-message documentation and basic browser response headers.
2. Coordinated room rate-limit/client recovery, followed by peer-keyed hosting admission design.
3. Safe dead-code/test-wiring cleanup and denominator-correct coverage measurement.
4. Contributor/history separation and shared E2E fixture/configuration helpers.
5. Retention ADR, then historical protocol reduction and a single item specification.
6. Measured bundle/render improvements and compatible dependency updates.

Public-hosting identity, historical replay retention and any reduced exhaustive-test cadence
need explicit requirements. None blocks the isolated fixes in the first batch. Fresh profiling,
fuzzing, a stuck-client shutdown reproduction and a complete protocol-deletion audit remain
outstanding; the review's baseline is not evidence those experiments ran.
