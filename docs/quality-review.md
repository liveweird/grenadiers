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

## Further reviewed fixes

**API-02/03 and HTTP examples from API-07:** unknown API paths return no-store problem 404s;
unsupported methods return problem 405s with route-specific `Allow`. WebSocket HEAD rejects
with problem representation headers and no body, while health/readiness HEAD still succeeds.
Both WebSocket routes apply no-store even to framework-generated errors and successful upgrades.
Fallback statuses are documented separately from supported OpenAPI operations; shared cache/retry
headers and problem examples match the implementation.

**API-06/07:** the live v11 message unions, principal messages/actions/settings and Oil/privacy
overview have descriptions. HTTP status/session examples are valid, and the active close-code
table distinguishes application policy from transport termination. Schema constraints and wire
types are unchanged; deeper definition documentation can be added as those areas evolve.

**BE-06 / CX-06:** stale dead-code exemptions are removed. Test-only restoration and Oil-grace
observation helpers compile only in core unit tests; the unused punch-cooldown restoration
method is deleted. Production cooldown observations used by the live projection are retained.
No physics, wire representation or public admission API changed.

**SEC-03:** operation throttles preserve the socket and are isolated across create/join/resume.
The browser performs at most three correlated same-socket resume retries within one ten-second
recovery budget. Pending recovery keeps the last world visible with controls disabled; stale
callbacks and late responses cannot revive it. Per-socket flood closure remains unchanged.

**SEC-04, partial / SEC-10:** global nosniff, frame-denial and referrer headers cover API,
static and fallback responses. HTML revalidates; recognized eight-character Vite hashes use
immutable caching, including hashes containing hyphens. API/WebSocket no-store is preserved.
That initial v0.3.50 change did not enable CSP; v0.3.52 follows below.

**DOC-01/02/04/14, partial:** the current verification policy and release procedures are now
short entry points. Dated verification and older release records live in `docs/history/`, with
their facts and headings preserved and relative links rebased. Known incoming fragments retain
aliases. The latest release is v0.3.55, sharing browser-test setup after test-guard and
admission hardening.
Testing, deployment and backlog consolidation remain separate work. Preserve the merged artist
changes and reconcile any concurrent screenshot documentation before editing overlapping files.

**API-08/09, TS-07 and BE-10/TL-04:** API lint now enforces closed object schemas, camelCase
properties and shared problem responses, with a narrow documented upgrade exception and
positive/negative fixtures. Gameplay generation, drift checks and reproducibility tests share
one explicit inventory that also checks each schema's identity. Browser scenarios are paired
with specific specs; duplicates, missing files and unsupported ambiguous declarations fail
explicitly. Dependabot groups routine version updates without suppressing major or security
proposals, and exact entropy/generator pins have documented rationale.

**DOC-06/10 and API-01 policy:** an [item-development guide](game/adding-items.md) maps current
domain, contract, presentation and test extension points. The [ADR index](decisions/README.md)
includes [retention policy](decisions/0021-gameplay-retention.md): one live network version,
explicit archived wire contracts and independent private replay compatibility. Unused historical
applications can be removed after moving unique regressions; adapters still serving the live
path require helper extraction first.

**FE-01 / API-01, first retirement:** remove the isolated v3 browser application and its
obsolete UI/client tests while retaining the v3 schema and conformance suite unchanged. Move
unique socket-factory, session POST/abort, native-fetch receiver and phase-error/correlation
regressions to the active v11 client. The production bundle is unchanged because that application
was already unreachable; this reduces maintained source rather than claiming a download saving.

**BE-04, bucket mechanics:** session minting/upgrades, room admission and inbound limits share
one configurable token bucket. Each caller retains its own budget, rates, lock and timeline;
the Tokio inbound wrapper is tested with paused time. Shared timeout policy remains follow-up.

**API-01 / CX-01, shared browser foundation:** profile/effect/item and freeze checks live in
`web/src/game/validation/` with minimal structural types. Active v11 no longer imports archived
browser helper namespaces; per-version AJV schemas/catalogs and v8/v9 compatibility wrappers
retain their distinct behavior.

**FE-02 and TS-03/04, partial:** normal production builds enforce measured aggregate gzip
budgets (400,000 JavaScript bytes, 40,000 CSS bytes). Coverage excludes test-only helper files
without changing executed tests or exhaustive acceptance. A fresh web baseline is
90.90/89.40/89.56/93.31% (statements/branches/functions/lines); floors rise to 90% lines and 88%
for the other metrics. Rust's shared helper exclusion was verified by full debug/release/coverage
acceptance over all 4,194,304 masks at `9122e33` (6m40s, no retries). The official LCOV contains
16,391/17,631 hit lines (92.97%) and no helper records. That measurement passed the previous
80% gate and supports raising both Rust reporting paths to 90%; command and real-LLVM fixture
tests verify the new threshold without repeating unchanged exhaustive execution.

**TL-10, audit findings:** update the aligned Vitest/coverage pair to 5.0.5 and Ajv's transitive
`fast-uri` to 3.1.9, with no unrelated direct dependency upgrades. Full and production npm
audits report zero vulnerabilities after a clean install. The URI parser is a production
dependency, so its patch is included in build/contract and final web verification.

**Integration test reliability:** full acceptance exposed a race in the new real-socket flood
test: per-frame flushing could either allow refill or race the server's close. Buffer a
policy-derived 151-frame burst, flush once and bound the complete exchange by the existing
two-second deadline. The test still requires an actual rate-limit error, policy close and permit
release; no retry, timeout increase or production hook is introduced.

**FE-07 / SEC-04, v0.3.51:** live gameplay/realtime validators are compiled at build
time into self-contained ESM, preserving schema, semantic/privacy and complete ordered error
behavior. Generation/drift and executable-syntax checks guard the artifacts; archived compilers
remain tested. The measured candidate bundle is 333,442 JavaScript gzip bytes versus 357,324 in
v0.3.50. That release did not enforce CSP; compatible Pixi/style and transport trials follow
in v0.3.52.
The integrated web gate passes 6,789 tests across 88 files; coverage is
90.95/89.49/89.78/93.32% (statements/branches/functions/lines). Lint, contract and validator drift,
unused-code checks, TypeScript and the production build pass. Subsequent comment-only
portability corrections pass 13 focused tests and fresh lint/drift/build checks. Exact artifacts
match across checkout depths, invocation directories and pinned Linux. Real development and
production browser flows pass, as do eight native cases each in Docker and Kubernetes.
The [v0.3.51 record](history/releases.md#v0351--2026-10-25) corrects the original Docker/Compose
attribution: direct Docker was used for those eight cases; promotion used Compose separately.

**FE-07 / SEC-04, v0.3.52:** shared Pixi static compatibility initializes before every
renderer. A static enforcing response policy restricts scripts/connections to self while allowing
Mantine's inline styles; diagnostic policies permit their real atlas fetches. Header tests cover
fallbacks, HEAD, revalidation and untrusted request sources. Pre-header enforced browser trials
cover HTTP/WS, HTTPS/WSS, actual blocked execution/connections, WebGL/context restoration,
artist upload/export, controls, audio and reconnect. The relay reset/config regressions pass.
Final same-image acceptance passes eight cases each in Compose and Kubernetes plus six
HTTP/WS + HTTPS/WSS policy cases, without relay injection.
Full Rust exhaustive/90%-floor coverage and the 6,790-test web gate pass; localhost:8085 serves
the accepted v0.3.52 image at that release. See
[ADR 0023](decisions/0023-browser-content-security-policy.md).

**BE-03 / SEC-01/07, v0.3.53:** add transport-derived peer identity with strict opt-in
immediate-proxy forwarding, independent bounded gameplay/diagnostic tables and per-peer budgets
before global backstops. Minting quotas count issued and claimed sessions without binding bearer
authentication to an address. Typed permits cover socket lifetime and complete room-admission
handlers, including response attempts. Fixed-label readable denial summaries exclude request
data and credentials. Independent review, full source-bound acceptance, final ordinary tests,
26 same-image browser cases and real proxy/quota/logging proofs pass. Exact source attribution
is recorded in the [release record](releases.md#v0353--2026-10-25). Artist PR #123 is preserved in
the same release: the new image passes 6,791 web tests, 26 native cases and fourteen separate
renderer/artist cases; Rust inputs and the server binary remain unchanged. See [ADR 0024](decisions/0024-peer-admission.md).

**BE-08 / CX-04/05, v0.3.54:** the source-size guard discovers every current/future
`e2e/*-tests` directory, preserving existing limits/exclusions; eight fixture regressions run in
the repository gate. Simulation test declarations move to a test-only include with the exact
same 926 test names and private namespace. Repository, ordinary debug/release, Linux, image
scan and Compose/Kubernetes acceptance pass; all fifteen production module declarations retain
their names, paths and order. Broader fixture sharing
remains separate work.

**CX-08 / TS-06 / TL-05, v0.3.55:** equivalent gameplay scenarios share only
native responsive browser-context construction. Desktop/mobile ordering, independent sessions,
viewport overrides and mobile reduced motion remain explicit; each scenario still owns its
pages, game setup, observations, assertions and cleanup. Independent review, static checks and
twenty exact-image native cases pass. Configuration factories and broader behavioral fixtures
remain separate work.

## Finding dispositions and proposed solutions

“Agree” accepts the observed issue, not every implementation proposed in the review.
Related IDs are grouped to avoid treating one architectural problem as several independent fixes.

| IDs | Disposition | Solution / next action |
| --- | --- | --- |
| DOC-01/02/04/05/07/08/09/14 | Agree: contributor guidance is buried in repeated release evidence. | Verification/release history is separated from current procedures; the index links both archives. Unreleased changelog organization is in place. Continue with testing/deployment/backlog and replace repeated per-item testing prose with a linked table. |
| DOC-03 | Agree on organization, not the claimed stale checkpoint status. | Move dated release narrative out of backlog; its explicitly historical “unreleased” state must not be rewritten as a current fact. |
| DOC-06/10/11/13 | Agree; index, item guide, ADR index and retention policy implemented. | Continue consistent item headings and implemented/future rule markers; move archived protocol guides with link checks. |
| DOC-12 | Factually stale, deliberately preserved. | Keep `AGENTS.md` untouched under the user's instruction; point readers to current conventions and the index. |
| API-01, CX-01/02, BE-07, FE-01, TS-08 | Agree on duplication; browser retirement and helper extraction implemented. | ADR 0021 separates retention. V3 UI is removed with unique regressions moved to v11; its schema/conformance remain. Shared browser semantics are neutral; extract Rust foundations before assessing obsolete adapters and parameterizing frame-bound tests. |
| API-02/03 | Agree; HTTP repair implemented. | Problem 404/405 fallbacks, route-specific `Allow`, shared cache/retry headers and API/SPA/HEAD tests are in place; fallback responses are separate from supported operation responses. |
| API-04, BE-02, SEC-02 | Agree; first batch implements this. | Independent diagnostic permits/rate budget and shared strict transport guards, with saturation, Origin, flood, cache-header and shutdown tests. |
| API-05 | Optional improvement, not a demonstrated defect. | Add stable Problem codes only when a consumer needs distinctions beyond HTTP status. Do not unify diagnostic and gameplay error vocabularies merely for symmetry. |
| API-06/07 | Agree; primary live-reference gaps addressed. | Actual close behavior, principal live-message descriptions and valid HTTP examples are documented without changing schemas or wire types. |
| API-08 | Agree; initial enforcement implemented. | Spectral checks error media/shared Problem references, object closure and camelCase, including referenced fallback responses; mutation fixtures guard enforcement and exceptions. |
| API-09 | Agree; shared inventory implemented. | Generation/check/reproducibility use explicit v2–v11 discovery with filename and declared version/ID checks; unexpected or incomplete inventories fail. |
| API-10, CX-09 | Live receive variants enumerated in v0.3.51; broader coverage remains a question. | Standalone parity tests explicitly accept all eight gameplay-v11 and three diagnostic server-message variants through both compilation routes. Audit client-message and archived-contract coverage separately before claiming a complete protocol matrix. |
| BE-01 | Confirmed cost, intentional documented behavior. | Decide whether production replay recording has an operational consumer. If not, feature-gate allocation/recording while retaining replay acceptance tests; document memory effects. |
| BE-03, SEC-01 | Agree; implemented in v0.3.53. | Strict transport/proxy identity, bounded independent peer tables, rate/concurrency/session quotas and global backstops are integrated. Source-bound/runtime acceptance passes; actual public proxy topology and ingress configuration remain operator requirements. |
| BE-04 | Agree; bucket implementation shared. | Independent caller budgets and clock/lock behavior are preserved. Consolidate repeated transport timeout definitions separately; no generic clock abstraction is introduced. |
| BE-05, TS-14 | Agree for active startup/configuration paths. | Centralize typed parsing and defaults; test invalid configuration and empty-origin behavior without logging credentials. Revisit archived low-coverage code with the retention decision. |
| BE-06, CX-06 | Agree; safe exemption cleanup implemented. | Removed the unused restoration method and stale exemptions; gated test-only hooks and preserved production cooldown observations. Admission-argument refactoring remains separate. |
| BE-08, CX-04/05 | Agree; aggregation and source-size coverage shipped in v0.3.54. | Test-only declaration aggregation preserves every test name and production module. Dynamic item-tree discovery and eight wired regressions close the guard gap; consolidate genuinely equivalent fixtures separately. |
| BE-09, CX-07 | Mixed: maintainability concerns, not proven reachable panics. | Replace transport-state assumptions with typed authenticated commands where useful; split orchestration around domain operations. Audit narrowing conversions before changing public APIs. |
| BE-10, TL-04 | Agree; rationale and grouping implemented. | Cargo patch, per-workspace npm minor/patch and Actions minor/patch groups preserve individual pinned RNG, Docker, major and security proposals. Document compatibility review; no version ignores or automatic merges. |
| BE-11, TS-09/10 | Mostly intentional tests/benchmarks. | Use virtual time where it preserves the real socket deadline being tested. Keep performance tests separate from ordinary suites. Do not replace useful waits or ignored benchmarks solely to improve counts. |
| BE-12/13 | Plausible shutdown/latency concerns. | Reproduce shutdown with a non-reading client; measure registry-lock work before moving bounded construction/entropy outside it. No demonstrated hang yet. |
| CX-03 | Agree on cross-layer item duplication, not the proposed enum choice. | Define one domain item specification and derive configurable wire catalogs from each schema's `ItemId`, not `PickupKind`; preserve archived subsets. Parameterize configuration scaffolding independently. |
| CX-08, TS-06, TL-05 | Agree; native context reuse shipped in v0.3.55. | Twenty-six equivalent setups share one constructor; representative default/390px/private native suites pass with assertions unchanged. Configuration factories and genuinely equivalent behavioral fixtures remain next. |
| FE-02 | Agree; measured bundle budget implemented. | Initial baseline was 356,970 JS / 36,292 CSS gzip bytes; limits remain 400,000/40,000 and the URI security patch also fits them. Profile CSS/import reductions separately; no reduction is claimed by the guard itself. |
| FE-03 | Partly disagree; see corrections below. | Typed i18next consolidation is reasonable cleanup. Add plural forms where grammar needs them, not for multiplication/count notation. |
| FE-04 | Mechanism confirmed; performance cost unmeasured. | Profile 20 Hz gameplay and commit durations first. Change arena/HUD subscriptions only for a demonstrated budget problem. |
| FE-05 | Agree; first batch implements this. | Localized accessible error recovery with explicit actions and component/parity tests. |
| FE-06 | Type-hygiene observation, not a broad defect. | Replace the isolated double cast when touching binding construction; no repository-wide type rewrite. |
| FE-07, SEC-04/10 | Basic headers/cache, standalone validators and compatible CSP implemented. | Pixi static paths, Mantine styles and same-origin HTTP/WS + HTTPS/WSS pass enforced trials. Actual server-header and same-image Compose/Kubernetes acceptance pass; retain the explicit style exception and Chromium support boundary. |
| FE-08 | Selector counts do not describe the current web tests. | Current web tests use role queries rather than test IDs. Assess E2E selectors in their own scope; a selector ratio alone is not an accessibility defect. Retain actual accessibility audits. |
| FE-09/10 | Agree, lower-priority consistency work. | Consolidate copied scene mocks, preserving specialized renderer/atlas mocks; introduce semantic color tokens incrementally. |
| TS-01/02 | Agree on cost; sampling is a policy change. | Measure a structured fast feedback suite while retaining exhaustive acceptance. Compare fault detection before proposing a new cadence; keep current assertions and exhaustive coverage meanwhile. |
| TS-03/04 | Agree; helper exclusions and threshold recalibration implemented. | Fresh application coverage supports web floors of 90% lines / 88% other metrics and Rust 90% lines. The full Rust measurement ran at the prior floor before ratcheting; execution and exhaustive masks remain unchanged. Optional tool-dependent gates stay explicit. |
| TS-05, TL-01/02/03, SEC-05 | Partly agree: orphan callers and gate discoverability need work. | Wire the orphan script tests now; audit manual diagnostic-suite coverage and document aggregate scope. Preserve local-first verification; duplicated automatic remote gates are not a default requirement. |
| TS-07 | Agree; mapped AST-based check implemented. | Explicit document/spec inventory and title parity reject duplicates and unknown/missing files; static string/template/alias forms work, while ambiguous modifiers fail with documented guidance. |
| TS-11/12 | Factual suggestions, not established missing-risk coverage. | Target parser mutation/property checks and simultaneous competing-resume tests at explicit invariants; existing concurrency tests already use synchronized claimants. |
| TS-13 | Disagree with sibling-test filenames as a coverage measure. | Use behavior and coverage of actual call paths. Indirectly covered components do not require duplicate tests solely to match filenames. |
| TL-06 | Agree on release bookkeeping cost. | Reduce duplicated release facts first; then automate version/changelog updates and applicable checks, without introducing another mandatory evidence pipeline. |
| TL-07 | Root fixture clutter is real; local checkpoint claim is historical. | Consolidate Compose fixtures. Local scratch/worktrees/images were already cleaned separately; retained history is compressed and recoverable. |
| TL-08/09 | Useful incremental guardrails, not all immediate defects. | Add selected shell/workflow checks with documented tool versions; evaluate stricter TS/lint rules by actionable findings rather than enabling everything at once. |
| TL-10/11 | Moderate audit findings patched; cache opportunities remain. | Vitest/coverage and transitive URI patches yield zero npm findings. Reuse verified tool/build caches, keep security failures visible and pin tool provenance. |
| TL-12 | Agree on build-cache/reproducibility tradeoffs. | Add dependency caching and OCI metadata. Replace unbounded package upgrades with a documented base-image/security-refresh policy; simply dropping upgrades can retain known vulnerabilities. |
| TL-13/14/15/16 | Mixed low-priority tooling/documentation concerns. | Scope the peer-dependency workaround, document supported Python/tooling prerequisites, name gate commands in the PR template, and retain existing pin consistency checks. |
| SEC-03 | Agree; coordinated server/client fix implemented. | Correlated throttles retain the socket; bounded same-socket retries use fresh sequences and fenced timers within one deadline. Initial admission remains manually retryable; per-socket floods still close. |
| SEC-06/07 | Capability tradeoff documented; denial visibility implemented in v0.3.53. | Fixed operation/reason counters and readable bounded summaries work under the default filter and exclude request data. The security guide explains leaked-capability takeover; recoverable rotation remains separate protocol work. |
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

1. Consolidate repeated testing/deployment documentation and share E2E context/configuration helpers.
2. Apply the retention policy to historical protocol reduction and a single item specification.
3. Measure bundle/render and lock/shutdown behavior before performance changes.
4. Continue compatible dependency/tooling hardening and capability/replay operational decisions.

Public-hosting topology, durable replay export and any reduced exhaustive-test cadence need
explicit requirements. None blocks the isolated fixes in the first batch. Fresh profiling,
fuzzing, a stuck-client shutdown reproduction and a complete protocol-deletion audit remain
outstanding; the review's baseline is not evidence those experiments ran.
