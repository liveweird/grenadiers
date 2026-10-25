# Quality review assessment

<a id="quality-review-assessment-and-remediation"></a>
<a id="first-implementation-batch"></a>
<a id="further-reviewed-fixes"></a>
<a id="remaining-agreed-work-before-cpu-mode"></a>

This is the technical response to Claude's 25 October 2026 review of `2f2b680`. It records
dispositions and constraints, not delivery status. [The product backlog](../BACKLOG.md) is the
sole source for current status and priority. The detailed in-progress remediation narrative is
preserved as a [dated historical snapshot](history/quality-review.md).

When the agreed review work is complete, move this assessment to history and leave a stable
pointer if existing links still need one. That archival step is documentation maintenance, not
a new approval, evidence, or release gate.

<a id="finding-dispositions-and-proposed-solutions"></a>

## Accepted architectural findings

- **Documentation:** separate current guidance from dated acceptance evidence, retain incoming
  fragments when moving material, and keep protocol archives available for compatibility.
  `AGENTS.md` is a deliberate exemption: it was factually stale but explicitly outside the
  authorized change scope, so current guides point readers to maintained conventions instead.
- **Protocol and duplication:** reduce duplicated live browser and Rust protocol mechanics while
  preserving archived schemas, conformance tests, replay identities, item bits, activation
  subsets, validation order, and privacy. Retention policy means old adapters are removed only
  after unique regressions move and live consumers no longer need them.
- **Transport and server boundaries:** isolate diagnostic and gameplay admission budgets, keep
  strict Origin/input/write limits, derive peer identity at the transport boundary, use bounded
  peer tables, and bound HTTP serving plus application drain by one signal-time deadline.
- **Frontend resilience and security:** keep recovery correlated and bounded, preserve the last
  valid world while controls are disabled, provide localized accessible fatal-error recovery,
  enforce measured bundle budgets, and apply CSP/cache/header changes only with actual renderer,
  style, HTTP, and WebSocket compatibility evidence.
- **Tests and tooling:** centralize explicit protocol/scenario inventories, keep equivalent
  fixtures shared only when semantics match, and preserve local-first verification. Exhaustive
  mask acceptance is the proof requirement; sampling may improve feedback time but is not a
  proof-equivalent replacement.
- **Deployment and dependencies:** pin reproducible inputs, group only compatible routine
  updates, preserve visibility of major/security proposals, and base public-ingress or network
  controls on a real deployment topology rather than an invented one.

## Qualified agreements and disagreements

- Stable Problem codes are useful only when a consumer needs distinctions beyond HTTP status;
  diagnostic and gameplay error vocabularies need not be unified for symmetry.
- Production replay allocation has a real memory cost, but recording policy depends on an
  operational consumer. Durable replay export is a separate product decision.
- Render subscription and CSS/import cleanup require measurements at the real publication rate;
  mechanism or bundle size alone does not establish a performance defect.
- Plural forms should follow grammar, not multiplication/count notation. Typed translation access
  is worthwhile without manufacturing plural keys.
- Selector ratios and sibling-test filenames are not accessibility or coverage measures. Review
  user-visible accessibility and exercised behavior instead.
- Virtual time is appropriate only when it preserves the real deadline under test. Useful waits,
  socket deadlines, and isolated benchmarks should not be rewritten merely to improve counts.
- Additional CI, lint, mutation, property, or shell checks need an identified coverage gap and
  actionable findings; duplicate remote pipelines conflict with the single-maintainer,
  local-first policy when they add no distinct coverage.
- Capability rotation, public-hosting policy, NetworkPolicy, and broader room controls remain
  requirements-dependent decisions, not implicit consequences of accepting the review.

<a id="corrections-that-change-the-implementation-plan"></a>

## Corrections to the review premise

1. Game translation resources are already typed: `web/src/i18next.d.ts` uses `typeof en`, and
   `i18n.ts` includes both `app` and `game`. Raw JSON interpolation is separate design debt.
2. Admission recovery is coordinated across server and client. The client clears pending
   admission on an error and schedules recovery on closure, so changing only the server's
   `RateLimited` close behavior would strand automatic resume.
3. Ajv compiles live validators at module load. CSP therefore needs compatible script and style
   handling; lazily importing diagnostics does not remove gameplay code generation.
4. `ItemId` is the configurable catalog. `PickupKind` also contains basic upgrades, generated
   TypeScript aliases are not runtime enum objects, and archived validators intentionally expose
   smaller catalogs.
5. Exhaustive acceptance is required by the [testing policy](testing.md#verification-policy).
   Oil placement filtering and mask-dependent random bounds mean sampled execution cannot be
   assumed proof-equivalent, even though feedback cost is worth measuring.
6. Historical checkpoint status and counts remain historical evidence. Coverage floors should
   be raised only after measuring the cleaned denominator.
7. Automatic remote checks are justified by distinct environment coverage, not by repeating the
   same branch, pull-request, merge, and tag work.

<a id="bounded-invariant-audit"></a>

## Bounded invariant conclusion

The targeted BE-09 audit found no reachable defect in the cited owner and projection paths.
One serial runner owns `Driver`; synchronous operations validate a live or pending slot and use
that same slot without intervening mutation. Resume preparation and commit change runtime state,
not the validated driver slot. Projection narrowings follow explicit bounds: player counts are
1–8, event indices are bounded to 63, and the generation conversions receive fixed escape-route
cells. Existing rollback, departure, and takeover tests cover the behavior. This is a bounded
assessment, not a claim that every assertion or cast in the repository was reviewed.
