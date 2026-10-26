# Release history

This archive preserves the older release records originally maintained in
[`docs/releases.md`](../releases.md). Records remain in their original order; the current
release procedure and current release record stay in the [release guide](../releases.md).

## v0.3.78 — 2026-10-26

[v0.3.78](https://github.com/liveweird/grenadiers/releases/tag/v0.3.78) shares equivalent Rust
frame-bound builders and open-border arenas plus timed-effect browser helpers. Independent review
confirms identical helper contents, per-version largest states, test names/assertions, seeds,
routes and screenshots. V8–v11 encoded frame sizes remain exact; the browser helper binds each
assertion to the bomb planted by that native key action.

Full source-bound Rust acceptance at `936f4e627fa3ec101f759b02f717b745d0d550f2` passes in
449.469s, including every 4,194,304 mask in debug, release and official coverage with
zero retries. Coverage is 18,049/19,738 lines (91.4429%).
Focused debug/release, Clippy, repository and E2E static checks pass. Application web acceptance
reuses the identical v0.3.77 input; existing Linux acceptance covers unchanged production code.

Accepted image source is `f4fd65220cf4fbdc7322fe54bca36af2c5abd288`, whose Docker inputs match
the tested source; image is `sha256:5281acc1e3a9b2f7959e2952edf57ee8bbee2eaf645fff81648ae3cedf175b5d`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.1s and
eight Kubernetes cases in 36.5s, with effective memory/PID/ephemeral-storage limits, security and
probes verified. Golden Flame, Short Fuse and Short Flame cases also pass through the new helpers
and exact read-only fixture mounts. Temporary resources are removed. Localhost:8085 has verified
health/readiness, CSP/cache and the exact accepted web bytes. Control is
`.local-checkpoints/runtime/compose-v0.3.78.yaml`; `rollback-v0.3.77.yaml` retains the prior image.

## v0.3.77 — 2026-10-26

[v0.3.77](https://github.com/liveweird/grenadiers/releases/tag/v0.3.77) enables exact optional
properties in application/tooling TypeScript, omits absent request/UI properties and uses correctly
typed diagnostic state and test events. Wire and visible behavior remain unchanged.

Independent review and all three TypeScript projects pass. Full web acceptance at
`e34bc505906eec657bf03d899ecc1b1eea2a7fef` passes 6,861 tests / 94 files in 249.42s with
91.24/89.88/90.12/93.60% coverage. Local gzip JavaScript/CSS is 337,188/36,695 bytes; the Docker
build is 337,956/36,695 bytes. Repository, lint, drift and dead-code checks pass. Unchanged Rust,
exhaustive, Linux and dependency acceptance are reused.

Accepted image source is that tested commit; image is
`sha256:46079733b7f6682f8460ba438a766c632a90a00f0d6af26f0a57e5a0fb43e166`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.8s and
eight Kubernetes cases in 36.7s, including exact-image/security/probe/effective resource checks.
Polish 320px locale switching and room creation pass; temporary resources are removed.
Localhost:8085 has verified health/readiness, CSP/cache and the exact new HTML/JS plus unchanged
CSS/atlas bytes. Control is `.local-checkpoints/runtime/compose-v0.3.77.yaml`;
`rollback-v0.3.76.yaml` retains the prior image. No gameplay, protocol, artwork or migration changes.

## v0.3.76 — 2026-10-26

[v0.3.76](https://github.com/liveweird/grenadiers/releases/tag/v0.3.76) adds optional pinned
ShellCheck/actionlint checks, verified tool/dependency cache restoration, selected terrain-pixel
acceptance, exact failure-artifact allowlists and controlled runtime OS refresh guidance.
Cache hits never skip verification; no hosted-cache timing benefit is claimed.

Independent review, repository/workflow contracts and pinned tooling checks pass. The terrain
comparison passes at DPR 1 and 2: baseline, repaint, reset and context restoration each have zero
differing bytes. The separate six-cell lifecycle experiment remains opt-in. Unchanged Rust,
exhaustive, Linux, application web and dependency acceptance reuse the published baseline.

Accepted image source is `8af26e21b420b2f609fc6b5cccf2303a339608c1`; image is
`sha256:f98095085aa41ba03e97a25523a9682393864dc7a798151f066c22d8151e8424`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 35.5s and
eight Kubernetes cases in 36.1s, with exact image, effective v0.3.75 resource limits, restricted
security and probes verified. Temporary projects/namespaces and diagnostic preview are stopped.
Localhost:8085 has verified health/readiness, CSP/cache and exact unchanged web bytes. Control is
`.local-checkpoints/runtime/compose-v0.3.76.yaml`; `rollback-v0.3.75.yaml` retains the prior image.
No gameplay, wire, artwork or migration changes; distribution remains source prereleases.

## v0.3.75 — 2026-10-26

[v0.3.75](https://github.com/liveweird/grenadiers/releases/tag/v0.3.75) bounds the Compose app to
256 MiB and 128 PIDs and requests/limits Kubernetes ephemeral storage at 16/64 MiB. Deployment
checks reject weakened effective security contexts and invalid resource quantities.

Independent review, 42 focused resource/security tests, repository checks, rendered-manifest
validation and strict Kubernetes schema validation pass. Rust, exhaustive, Linux and application
web inputs reuse accepted v0.3.74 results; the resource-checker cases are exercised separately.

Accepted image source is `bfb0141e0373a75073d7ab922990f87a570e776a`; image is
`sha256:2ce17b3d8712817628504d6d472fe2a2a938290388e462e9eec08b479c01ea4b`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose cases pass in 36.0s and
eight Kubernetes cases in 36.5s. Docker HostConfig and live cgroups enforce 268,435,456 bytes and
128 PIDs; the live Pod has exact CPU/memory/ephemeral-storage requests/limits, restricted namespace,
security contexts and probes. No destructive resource stress was introduced. Temporary resources
are removed. Localhost:8085 has verified identity, security, health/readiness, CSP/cache and exact
accepted web bytes. Control is `.local-checkpoints/runtime/compose-v0.3.75.yaml`;
`rollback-v0.3.74.yaml` retains the prior image. No gameplay, protocol, artwork or migration changes.

## v0.3.74 — 2026-10-26

[v0.3.74](https://github.com/liveweird/grenadiers/releases/tag/v0.3.74) shares 16 Rust-produced
admission/snapshot frames across retained v8–v11 adapters, schemas and browser validators.
Cases retain freeze, teleport and Oil differences, recipient privacy, identity and metadata
negative controls. Production projection, schemas and gameplay remain unchanged. Independent review is clear.

Full source-bound Rust acceptance at `28ab639e6dd1a7def1147b514480d8341d6c2163` passes in
488.608s: 955 core / 316 ordinary server tests plus two doctests, all 4,194,304 masks in debug,
release and official coverage, zero retries. Coverage is 18,049/19,738 lines (91.4429%).
Full web acceptance passes 6,828 tests / 94 files in 286.25s with 91.24/89.87/90.12/93.60%
coverage and local gzip JavaScript/CSS 337,162/36,695 bytes.
[Integrated Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40373348609)
pass at `5db1d0e46c14b7278fe854af9578567999b3ebdf`, with identical core/server/API inputs.
Repository, lint, types and contract drift pass; dependency acceptance is unchanged.

Accepted image source is `28ab639e6dd1a7def1147b514480d8341d6c2163`, image
`sha256:4d4189e6dd868fb831f5aea82371320ba0d44b78da70fc77f406f6faeb4bf469`.
Docker gzip JavaScript/CSS is 337,939/36,695 bytes. Trivy 0.74.0 reports zero fixable HIGH/CRITICAL
findings. Eight Compose and eight Kubernetes cases each pass in 36.9s with exact-image/security/
probe/resource checks; temporary projects/namespaces are removed. Localhost:8085 has verified
health/readiness, CSP/cache and unchanged accepted HTML/JS/CSS/atlas bytes. Control is
`.local-checkpoints/runtime/compose-v0.3.74.yaml`; `rollback-v0.3.73.yaml` retains the prior image.
No gameplay, wire, artwork or migration change; distribution remains source prereleases.

## v0.3.73 — 2026-10-26

[v0.3.73](https://github.com/liveweird/grenadiers/releases/tag/v0.3.73) adds competing-resume
and cancellation regressions, including real socket takeover fencing. Two valid queued resumes
may both commit in order; the last binding is authoritative. Canceled admissions do not commit.
Production ownership and wire behavior remain unchanged.

The parallel Rust runner now fails early when Cargo rejects a restored coverage cache and
isolates official reporting to every instrumented workspace executable selected from one Cargo
JSON stream. Binary/profile identities, the 90% floor, exclusions and every mask assertion remain.
Independent review covers stale-object reproduction, complete lib/main selection, artifact mutation,
preexisting-target preservation, owned cleanup and symlink rejection. A scratch execution record
is written before reporting. This closes the [earlier report-contamination failure](coverage-cache-recovery.md).

Full source-bound acceptance at `822991f6e22817d525d4a300ebb24e21f48bac49` passes in 452.372s:
951 core / 316 ordinary server tests and two doctests per profile, all 4,194,304 masks in debug,
release and official coverage, zero retries. Coverage is 17,864/19,553 lines (91.3619%).
[Integrated Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40373348609)
pass with identical competing-resume tests. Unchanged web/dependency acceptance reuses v0.3.72.

Accepted image source is `806c13d0f25ac7327df49237f08fddd50ea94259`, whose Docker inputs equal
those of the tested source; runner-only changes are outside the image. Accepted image is
`sha256:06d7c75d4148ee2ff9897d984407450a465f4b34ac4ca404ef9473a9ec616121`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight Compose and eight Kubernetes
cases each pass in 36.6s, including exact-image/security/probe/resource checks; temporary resources
are removed. Localhost:8085 has verified health/readiness, CSP/cache and unchanged served web bytes.
Control is `.local-checkpoints/runtime/compose-v0.3.73.yaml`; `rollback-v0.3.72.yaml` retains the
prior image. No gameplay, protocol, artwork or migration change; distribution remains source prereleases.

## v0.3.72 — 2026-10-26

[v0.3.72](https://github.com/liveweird/grenadiers/releases/tag/v0.3.72) shares expanded client
conformance cases across v8–v11 Rust parsers/schemas and the browser, with deterministic malformed
frame, retired-action, required-field and byte-boundary regressions. Production parsers and
schemas remain unchanged. Independent review is clear.

At `f8457ef351af5822996b4f143efe0831abe45909`, ordinary profiles pass 951 core / 312 server
tests plus two doctests; all 4,194,304 masks complete in debug, release and official coverage
with zero product retries. The final runner report initially failed because stale cache objects
contaminated discovery. An official isolated report using the same three executed binaries and
38 profiles passes the unchanged 90% floor: 17,863/19,553 lines (91.356825%). The
[recovery record](coverage-cache-recovery.md) preserves the original failure and provenance;
no successful canonical runner summary or Rust duration is invented.

Full web acceptance passes 6,822 tests / 93 files in 264.03s with 91.07/89.40/90.12/93.48%
coverage. Local gzip JavaScript/CSS is 337,162/36,695 bytes; the accepted Docker build is
337,939/36,695 bytes, within the same budgets. Repository, lint, types and drift pass.
[Integrated Linux units](https://github.com/liveweird/grenadiers/actions/runs/40373348609)
pass at `5db1d0e46c14b7278fe854af9578567999b3ebdf`, whose client corpus/test inputs are identical.

Accepted image source is the tested `f8457ef351af5822996b4f143efe0831abe45909`, image
`sha256:e4989f32b30bb78400a12229dd3eb13c86a0a7f4ffd4bfea2bf3c943a286b5c1`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings; eight Compose cases pass in 35.4s and
eight Kubernetes cases in 35.9s with exact-image/security/probe/resource checks. Temporary resources
are removed. Localhost:8085 has verified health/readiness, CSP/cache and exact accepted web bytes.
Control is `.local-checkpoints/runtime/compose-v0.3.72.yaml`; `rollback-v0.3.71.yaml` retains the
prior image. No gameplay, wire, artwork or migration change; distribution remains source prereleases.

## v0.3.71 — 2026-10-26

[v0.3.71](https://github.com/liveweird/grenadiers/releases/tag/v0.3.71) adds isolated owner,
registry and entropy measurements plus real non-reading TCP regressions. The regressions verify
write recovery, bounded shutdown, healthy-peer delivery and cleanup. Production registry locking,
entropy boundaries and timeout values remain. The [measured reference](registry-admission.md)
does not justify a locking refactor or establish running-match capacity.

Independent review is clear. Full source-bound Rust acceptance at
`0a3c586cd58ce3938763502017e13fb9e8fdd2c9` passes in 490.564s: every 4,194,304 mask in debug,
release and official coverage, zero retries, 940 core / 312 ordinary server tests and two doctests
per profile. Coverage is 17,847/19,553 lines (91.2750%) at the 90% floor.
[Distinct Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40370445863)
pass, including the real socket-pressure cases. Integrated image source
`56639524da946f6d3cab3185fc58df045c766a1c` preserves the tested core/server/API inputs.
Unchanged v0.3.70 web and dependency acceptance are reused.

Accepted image is `sha256:fe1dbe95746a3a608e4070c4045d0d725ff606209f76c336301d7503fedff8ca`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight fresh Compose cases pass in
35.4s and eight Kubernetes cases in 37.7s, with exact-image/security/probe/resource checks.
Temporary acceptance projects and namespaces are removed.

Localhost:8085 serves the accepted image. Health/readiness, runtime security, CSP/cache and exact
HTML/JavaScript/CSS/atlas equality with v0.3.70 pass. Control is
`.local-checkpoints/runtime/compose-v0.3.71.yaml`; `rollback-v0.3.70.yaml` retains the prior image.
There is no gameplay, protocol, artwork or migration change. Distribution remains source
prereleases; measurement profiles are test-only and do not add production endpoints.

## v0.3.70 — 2026-10-26

[v0.3.70](https://github.com/liveweird/grenadiers/releases/tag/v0.3.70) derives game-copy types
from the canonical locale registry, returns stable raw copy objects and uses public i18next
interpolation. Existing EN/PL text, fallback, timers and stack notation remain unchanged.
Literal templates carry typed interpolation names; widened dynamic strings retain their dynamic
parameter map rather than claiming compile-time placeholder validation.

Independent review is clear. Full web acceptance at
`cdd6226ea6fb990f210ac14ac119045336a05443` passes 6,817 tests / 92 files in 226.02s with
91.07/89.40/90.12/93.48% coverage; gzip JavaScript/CSS is 337,939/36,695 bytes. TypeScript, lint,
contract drift, dead-code and repository checks pass. Integrated source
`1890db16d9f4eec5190390fe664a8897733d167b` preserves the tested web/assets/API inputs. Backend,
exhaustive Rust, Linux and dependency acceptance reuse v0.3.68/v0.3.69 unchanged inputs.

Accepted image is `sha256:8f913babc354d219c9d7b2a69669fc79f6e65f2981314a5fcbbeacee8c6a33bc`.
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight native Compose cases pass in
35.7s and eight Kubernetes cases in 36.3s, with exact image/security/probe/resource checks.
A Polish 320px responsive locale switch and native create-room smoke also pass. Temporary
acceptance projects/namespaces are removed.

Localhost:8085 serves the accepted image with verified health/readiness, CSP/cache and exact
served HTML/JavaScript/CSS/atlas bytes. Control is `.local-checkpoints/runtime/compose-v0.3.70.yaml`;
`rollback-v0.3.69.yaml` retains the previous image. There is no gameplay, wire, artwork or migration
change. Distribution remains source tags and GitHub prerelease notes.

## v0.3.69 — 2026-10-26

[v0.3.69](https://github.com/liveweird/grenadiers/releases/tag/v0.3.69) replaces 24 repeated
Compose seed overlays with `e2e/compose/fixture.yaml`. Set `GRENADIERS_FIXTURE_SEED_FILE` to the
exact seed path; the read-only bind rejects missing files instead of creating host directories.
Feature seeds, browser acknowledgements, scenario identities and base-only teardown remain.
See [fixture usage](../deployment.md).

Independent review and repository/workflow contracts pass. The 96 rendered-model comparisons
cover 24 features, default/custom environment and persistence off/on; the only intentional
model difference is `create_host_path: false`. Missing/unset/empty selectors fail safely.
Integration preserves the v0.3.68 backend and v0.3.65 web inputs; their full Rust, exhaustive,
Linux, web, coverage and dependency results are reused without duplicate suites.

Accepted image source is `137bdd68a4cae1d751bc98183c5fb5b9e1829ae0`, image
`sha256:9eade53a5b910f55627d6ef8f8d06a726698e7879316e41242142122e0266c95`.
Its OS layer was refreshed for the newly reported OpenSSL fix. Trivy 0.74.0 reports zero fixable
HIGH/CRITICAL findings. Fresh exact-image Compose acceptance passes eight default cases in 35.4s
and Heart, Kick, Cloak and Oil cases through the new shared overlay with exact read-only mounts.
Eight Kubernetes cases pass in 36.5s with exact-image/security/probe/resource checks. Temporary
Compose projects and Kubernetes namespaces are removed. Earlier runs using superseded overlays
are retained separately and are excluded from these shared-fixture results.

Localhost:8085 serves the accepted image; runtime/security, health/readiness, CSP/cache and exact
served HTML/JavaScript/CSS/atlas checks pass. Control is
`.local-checkpoints/runtime/compose-v0.3.69.yaml`; `rollback-v0.3.68.yaml` retains the prior image.
No gameplay, protocol, artwork or migration changes. Distribution remains source prereleases.

## v0.3.68 — 2026-10-26

[v0.3.68](https://github.com/liveweird/grenadiers/releases/tag/v0.3.68) makes replay recording
explicitly opt-in for test fixtures. Production rooms reserve no journal vector and skip replay-only
capture work; live snapshots, replay compatibility, reconnect and exhaustive assertions remain.
The allocation observation is 3,840,000 fewer reserved vector bytes per default room, not an RSS
or throughput claim; see [the measurements](replay-allocation.md).

Independent review is clear. Full Rust acceptance at
`6c01cc4c97324ddef87e588a1bdbe8fb1deea135` passes in 431.697s: every 4,194,304 mask in debug,
release and official coverage, zero retries, 940 core / 309 ordinary server tests and two doctests
per profile. Coverage is 17,833/18,933 lines (94.1900%), above the 90% floor. Integrated image
source `6f443a2056b91b713b5fd45d3e2e8de9a7e40056` preserves those core/server/API inputs.
[Distinct Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40369574894)
pass. Unchanged web, dependencies and audits reuse v0.3.65/v0.3.67 acceptance.

The runtime OS layer was refreshed after a current scan found fixable OpenSSL findings in the
older candidate. The replacement image is
`sha256:0bb13e287162a7ba79937dba4f95dd161bb343248ea4e0cbf026f9c74c2d5fc3`;
Trivy 0.74.0 reports zero fixable HIGH/CRITICAL findings. Eight fresh native Compose cases pass
in 35.3s on this exact replacement. Eight fresh Kubernetes cases pass in 36.9s, including exact-image, security, readiness, probe
and resource checks; the temporary namespace is removed.

Localhost:8085 serves the replacement image. Image identity/security, health/readiness, CSP/cache
and served HTML/JavaScript/CSS/atlas bytes match the accepted web artifacts. Control is
`.local-checkpoints/runtime/compose-v0.3.68.yaml`; `rollback-v0.3.67.yaml` retains the previous
release. There is no migration or public replay export. Source tags and GitHub prerelease notes
are the distribution; no production registry is configured.

## v0.3.67 — 2026-10-26

[v0.3.67](https://github.com/liveweird/grenadiers/releases/tag/v0.3.67) adds named admission
projection and personal-metadata inputs for protocol v8–v11. Legacy Rust wrappers remain
available, and the live caller uses the named input without changing wire validation, ownership,
ordering, privacy, schemas or gameplay behavior. The existing v0.3.65 web build is reused.

Fresh source-bound Rust acceptance at `46a1e3320e2ecf5e8c6291884d1304d0dfaa6d22` completes every
4,194,304 mask in debug, release and official coverage in 432.208s with zero retries. The ordinary
profiles each pass 940 core / 307 server tests plus two doctests. Coverage is 17,809/18,909 lines
(94.1827%) at the 90% floor. Integrated source `43dcd78050d5a306b8822b62a17728cb56c8c926`
has identical core, server and API inputs. Full v0.3.65 web acceptance is reused: 6,812 tests / 91
files with 91.10/89.47/90.12/93.47% coverage. Dependency inputs and accepted audits are unchanged.
[Distinct Linux debug/release units](https://github.com/liveweird/grenadiers/actions/runs/40256918030)
pass on the final candidate.

Accepted image source is `43dcd78050d5a306b8822b62a17728cb56c8c926`; its image is
`sha256:0845bc797cc6082e9c331f9d0b0c871695c13e206cdcc8ddb1b9bf525f9c7f84`.
The pinned image scan reports zero fixable HIGH/CRITICAL findings. Eight fresh native Compose
cases plus Invisibility Cloak and Oil scenarios pass. Eight Kubernetes cases pass in 36.6s,
including exact-image, runtime-security, readiness, probe and resource checks; the temporary
namespace is absent afterward. An initial attempt passed a bad helper argument and stopped before
any test ran; the corrected acceptance above passed without product retries.

Localhost:8085 serves the accepted image. Exact image, runtime security, health/readiness,
CSP/cache and served HTML/JavaScript/PNG equality with the accepted unchanged web artifacts pass,
plus one fresh native authentication case in 364ms. Release-documentation changes preserve Docker
and application inputs. Control is `.local-checkpoints/runtime/compose-v0.3.67.yaml`;
`rollback-v0.3.66.yaml` retains the prior image. Temporary acceptance resources are removed.
There is no migration.

## v0.3.66 — 2026-10-26

[v0.3.66](https://github.com/liveweird/grenadiers/releases/tag/v0.3.66) derives reserved item
identities, bits, pickup declarations and total mappings from one private 22-item specification.
Generation order, activation subsets, replay values, public schemas and gameplay behavior remain
unchanged. The existing v0.3.65 web build is reused.

Independent review is clear. Fresh source-bound Rust acceptance at
`ba7d6eea4728f4feae6c60ddb67c45451c39b8c0` completes every 4,194,304 mask in debug, release and
official coverage in 429.862s with zero retries. The ordinary profiles each pass 934 core / 307
server tests plus two doctests. Coverage is 17,615/18,793 lines (93.7317%) at the 90% floor.
Rustfmt, Clippy, repository checks and
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40240422294) pass.
Full v0.3.65 web acceptance is reused: 6,812 tests / 91 files with
91.10/89.47/90.12/93.47% coverage.

Integrated image source `c244193ae048f3dea2864bd120511be60981bb09` preserves those tested
core, server and API inputs. Its accepted image is
`sha256:d4906dd81f0c86803e8d7a96bd890723ec180718d9cc015c286d7d94ebfc941d`.
Eight fresh native Compose cases and eight Kubernetes cases pass in 36.1s and 36.9s,
respectively, including exact-image, runtime-security, origin, readiness/probe/resource and
gameplay checks. Heart, Kick, Invisibility Cloak and Oil browser scenarios pass. The pinned image
scan reports zero fixable HIGH/CRITICAL findings.

Localhost:8085 serves the accepted image. Runtime security, health/readiness, CSP/cache and exact
served-artifact checks pass, plus one fresh native authentication case. An initial attempt found
missing E2E dependencies before any case ran; after installing the declared dependencies, the
fresh acceptance above passed without product retries. Release-documentation changes preserve
Docker and application inputs and skip duplicate CI. Control is
`.local-checkpoints/runtime/compose-v0.3.66.yaml`; `rollback-v0.3.65.yaml` retains the prior image.
Temporary acceptance resources are removed. There is no migration.

## v0.3.65 — 2026-10-26

[v0.3.65](https://github.com/liveweird/grenadiers/releases/tag/v0.3.65) names recurring UI color
roles in `App.css`. All 252 existing resolved declarations, selectors, dimensions and visual
values remain unchanged; distinct one-off shades remain explicit. No artwork or gameplay changes.

Independent review checks resolved values, scope, undefined variables and cycles. Full web
acceptance at `9e4b020cada15c5e36f2e2dd297e4cb7eecc437e` passes 6,812 tests / 91 files in
226.72s with 91.10/89.47/90.12/93.47% coverage. JavaScript/CSS gzip sizes are 337,921/36,695 bytes,
within existing budgets. Repository, TypeScript, lint, drift and dead-code checks pass.

Integrated image source `0a46613dde32585dcf2a08faea98a271d2e10db9` preserves that tested web tree
and the v0.3.64 backend. Accepted image is
`sha256:6aed11a165885225691ae10b70ab349999853a7c14e398a38d12b65b4b90b596`.
Eight native Compose cases and eight Kubernetes cases pass, with exact-image/runtime security,
origin, readiness/probes/resources and gameplay checks. The pinned image scan reports zero
fixable HIGH/CRITICAL findings. Reuse v0.3.64 backend/exhaustive/Linux and dependency acceptance;
there is no distinct Rust behavior to recheck remotely.

Localhost:8085 serves the accepted image. Exact served HTML, JavaScript, CSS and atlas bytes match
the tested build; CSP/cache, health/readiness and runtime security pass, plus one fresh native
authentication case. Release-documentation changes preserve Docker inputs and skip duplicate CI.
Control is `.local-checkpoints/runtime/compose-v0.3.65.yaml`; `rollback-v0.3.64.yaml` retains the
prior image. Temporary acceptance resources are removed. There is no migration.

## v0.3.64 — 2026-10-26

[v0.3.64](https://github.com/liveweird/grenadiers/releases/tag/v0.3.64) adds a validated,
dry-run-first release preparation helper and Cargo caches for container builds. The helper updates
aligned Cargo versions and dated changelog entries without running Git, publication or checks.
BuildKit reuses locked registry/Git caches and a platform-specific target cache; touching the
production crate roots forces changed workspace code to rebuild even when source mtimes are old.
The binary is copied outside the cache mount. Static OCI metadata identifies the project.
The existing bundled build frontend and runtime security policy are retained.

Independent helper/cache review is clear. All fourteen helper cases pass on macOS and Linux;
repository checks pass. Actual cold, older-mtime mutant and restored-source builds execute the
compile step: the mutant changes health to its probe value, and restoring source reproduces the
original binary. These are cache-correctness observations, not capacity or performance guarantees.

Accepted image source `332e3608a64cf9832e62afd6c80a7e9dcb2c7f49` builds
`sha256:1c6bd4094db67619c4f2c77c234bbaee34e5b4cccc17e5cb49bd8f38b0d2e27a`.
Eight native Compose cases and eight Kubernetes cases pass, including actual image identity,
startup/readiness, security, origins, probes/resources and browser gameplay. The pinned image
scan reports zero fixable HIGH/CRITICAL findings; one additional post-promotion native
authentication case passes. [Linux units](https://github.com/liveweird/grenadiers/actions/runs/40245664447) pass.

Application sources, contracts, web assets and external dependency versions are unchanged from
v0.3.63. Reuse that source's full Rust/exhaustive acceptance and dependency audits, plus the
unchanged v0.3.62 web acceptance. Release-documentation edits do not change Docker inputs.
Localhost:8085 serves the accepted image, with exact HTML/JavaScript/atlas equality and runtime,
CSP/cache, health/readiness checks passing. Control is
`.local-checkpoints/runtime/compose-v0.3.64.yaml`; `rollback-v0.3.63.yaml` retains the prior image.
Temporary acceptance resources are removed. There is no migration.

## v0.3.63 — 2026-10-25

[v0.3.63](https://github.com/liveweird/grenadiers/releases/tag/v0.3.63) separates authentication
from the nine sequenced commands before dispatch. Authenticated dispatch now accepts a private
sequenced type, preserving high-water advancement before semantic checks, error correlation,
quotas, admission ownership and every close sequence. Gameplay and diagnostics share the same
five-second per-frame send policy; whole final-delivery sequences retain their independent outer
budget. No wire, schema, replay, dependency or client behavior changes.

Independent review and focused real-socket regressions pass, including all nine pre-auth command
variants, semantic-failure sequence consumption, reauthentication closure and distinct counters.
Fresh source-bound Rust acceptance at `c8cc672812ea04d74a074a9570c9f473bd91dd62` completes every
4,194,304 mask in debug, release and official coverage in 428.571s with zero retries. Each ordinary
profile passes 930 core / 307 server tests plus two doctests; four existing performance cases
remain ignored. Coverage is 17,514/18,719 lines (93.56%) at the 90% floor. Rustfmt, Clippy,
runner/repository checks and [Linux units](https://github.com/liveweird/grenadiers/actions/runs/40237196707) pass.

Integrated image source `c50773d764ea0fd897de809de746719f9a7cde8e` has identical Rust inputs and
the unchanged v0.3.62 web tree. Accepted image is
`sha256:efa9be5a847ed7334388b1129a6be980e5df5fe25c72fc3d7d7620bc9dfc87f9`.
The image passes eight native Compose cases and eight Kubernetes cases, with exact image,
security/probes/resources/origin/CSP/cache and readiness checks. A fresh pinned image scan reports
zero fixable HIGH/CRITICAL findings. Reuse v0.3.62 web acceptance and unchanged dependency audits.

Localhost:8085 serves the accepted image. Runtime security, health/readiness, CSP/cache and exact
HTML/JavaScript/atlas equality with accepted unchanged assets pass, plus one fresh native
authentication smoke. Temporary acceptance resources are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.63.yaml`; `rollback-v0.3.62.yaml` retains the prior image.
There is no migration. Core admission argument grouping and other remaining review work stay
separate; CPU implementation remains deferred.

## v0.3.62 — 2026-10-25

[v0.3.62](https://github.com/liveweird/grenadiers/releases/tag/v0.3.62) removes 120 copied mock
lines from Diarrhea, Slow Foot and Reverse Controls scene tests. All three reuse the existing
Vitest-safe Pixi mock; test bodies, titles and assertions remain unchanged. Specialized renderer
and atlas mocks remain intentional. Production behavior, protocols and dependencies are unchanged.

Fresh full web acceptance at `50e57080aaf90fc03f2675f940be3d4adac9ca95` passes 6,812 tests /
91 files in 307.97s, with 91.10/89.47/90.12/93.47% coverage and 337,937/36,505-byte gzip JS/CSS.
Focused tests, lint, TypeScript, Knip and repository checks pass. Integration of v0.3.61 changes
no tested web files. Application image inputs match `ea90eaf91390db05dfdc0398be0328e1ccc7bede`;
publication adds documentation. Accepted image is
`sha256:a5025d0406d61cdaf1bfd6cd3800a7440f72b39c04c003c5dd3b4a6460ff11af`.

Rust/runtime sources match v0.3.61 apart from product-version metadata, so reuse its complete
source-bound 4,194,304-mask acceptance (93.49% lines), distinct Linux units, 16 native cases
and five startup probes. Dependency inputs also reuse the existing Cargo/npm audit results.
A fresh pinned exact-image scan reports zero fixable HIGH/CRITICAL findings; the project-owned
Trivy cache avoids repeated database downloads without bypassing updates or failures.

Localhost:8085 serves this image. Health/readiness, runtime security, CSP/cache and exact
HTML/JavaScript/atlas equality with the accepted v0.3.60 web artifacts pass, along with one
fresh native admission/room smoke. Control is `.local-checkpoints/runtime/compose-v0.3.62.yaml`;
`rollback-v0.3.61.yaml` retains the previous image. No migration is required. Remaining agreed
Claude review work still precedes CPU implementation.

## v0.3.61 — 2026-10-25

[v0.3.61](https://github.com/liveweird/grenadiers/releases/tag/v0.3.61) centralizes production
startup in a typed configuration. Invalid address/origin settings fail with value-free errors;
OS-native paths, proxy → seed → lazy database validation, existing AppState constructors and
invalid-log fallback remain supported. An explicitly empty origin list still denies browser
access and now emits one warning. Docker logging matches Compose/Kubernetes; the
[defaults table](../security.md#startup-configuration) distinguishes intentional development and
container values. No gameplay, protocol, replay or dependency changes are introduced.

Tested application source is `29093f3a50297f97cba0033a0f34b2244872c98b`; publication adds only
documentation. Localhost:8085 serves accepted image
`sha256:a22899d0d0e0c45063bf658c089ce55428ac02f37d6417a4270705cc02c42696`.
Independent review, 12 focused startup cases and nine origin cases pass. Fresh source-bound Rust
acceptance completes every 4,194,304 mask in debug, release and official coverage in 499.886s,
with zero retries. Ordinary profiles pass 930 core / 304 server tests and two doctests; four
existing performance cases remain ignored. Coverage is 17,388/18,598 lines (93.49%) at the 90%
floor. Rustfmt, Clippy, runner fixtures, repository checks and
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40234533129) pass.

The exact image passes eight native browser cases through Compose and eight through Kubernetes.
Five actual-image startup probes verify typed/redacted address, database, seed and proxy errors,
plus healthy startup/log fallback, one empty-origin warning and browser-session denial.
A fresh pinned Trivy scan reports zero fixable HIGH/CRITICAL findings. Web/contracts/dependency
inputs are unchanged; reuse v0.3.60 web acceptance and v0.3.57 Cargo/npm audits. Served HTML,
JavaScript and atlas bytes match the accepted v0.3.60 artifacts exactly.

Initial temporary checks needed corrections: Compose override setup, browser sandbox permission,
the absent host Trivy binary (use the pinned container), and a helper expecting API cache policy
for HTML. These were setup failures; final checks use documented policies and unchanged product
assertions. Failed outputs or tool excerpts are retained; no hidden retries were added.

Promotion passes image/runtime security, health/readiness, CSP/cache/artifact equality and one
additional native room-admission smoke. Temporary acceptance resources are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.61.yaml`; `rollback-v0.3.60.yaml` retains the prior image.
Restarts still end process-local matches. The review status now separates shipped fixes from
remaining agreed work; CPU implementation waits for that work to finish.

## v0.3.60 — 2026-10-25

[v0.3.60](https://github.com/liveweird/grenadiers/releases/tag/v0.3.60) enables all 22 configurable
items in newly created live rooms, including curses and Oil. All on/All off edit only the item
draft; numeric settings, editing locks and explicit Apply remain intact. Restore match defaults
enables all available items; Classic disables optional items. Existing rooms, rematches and
historical replay defaults retain their settings. Live replay headers record explicit defaults;
no wire, rules, generator or replay format version changes.

A responsive own-player bar shows upgrade totals, equipment/weapon slots, every effect timer,
all Slow Foot stack timers, freeze and damage grace. Counts come from authoritative snapshots;
an opponent's Stopwatch freeze has no invented countdown. Recent own pickups, including basic
upgrades and Teleporter, are announced once, with reconnect/round/death boundaries respected.
Oil remains a world object. English/Polish text and icons reuse the current sheet. See
[settings](../game/settings.md) and [browser presentation](../game/browser.md).

Application inputs match `c5b668f9fddf8cf95a582c499192a92f658a0e95`; later source changes only E2E
assertions, artifact ignores and documentation. Accepted image is
`sha256:ab7cf594b833c7d5c9b3c7dec15a34292dc22854affa00a949b3a309b7df1e8b`.
Independent server, web and scenario reviews are clear. Corrections prove the active collector's
Polish 320px bar and expose remaining grace time in its accessible name. Own/rival numeric and
label-geometry assertions are preserved after moving the own bar.

Fresh Rust acceptance at clean source `5e4743d8d868ee86cf7a078aa88af150688d8d1b` has identical
Cargo/core/server inputs. All 4,194,304 replay masks pass in debug, release and official coverage
in 544.996s, with 32 complete partitions per profile and zero retries. Ordinary profiles pass
930 core and 296 server tests plus two doctests; four existing performance cases stay ignored.
Coverage is 17,290/18,492 lines (93.50%) at the 90% floor, with helper exclusions intact. The
initial sandbox denied local listener binds; the unchanged suite passed with socket permission.
Rustfmt, Clippy and the runner fixture checks pass.

The fresh web gate passes 6,812 tests / 91 files in 292.82s. Coverage is
91.10/89.47/90.12/93.47% above the existing floors; gzip JavaScript/CSS are 337,937/36,505 bytes.
API/schema/standalone drift, lint, unused-code, build, repository and full E2E static checks pass.
The pre-review coverage attempt was interrupted for the accessible-name fix; this final run
uses the corrected source. Dependency inputs are unchanged; reuse v0.3.57's fresh Cargo-deny
and both zero-finding npm audits. A fresh pinned Trivy scan of this exact image reports zero
fixable HIGH/CRITICAL findings under the existing policy.

Exact-image native Chromium passes 32 Compose cases: all 22 item scenarios, generated upgrades,
a complete classic match/rematch, and eight default cases. Eight more default cases pass
through Kubernetes (40 total), with zero configured retries. Collector-side Polish 320px Heart
capture, pickup feedback, replacement/consumption, expiry, privacy and own/rival geometry are
covered. The [distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40232475529)
pass. Two temporary Kubernetes helper preflights failed (resource path and omitted empty env
values) before browser execution; corrected helpers pass, and failed logs remain retained.

Localhost:8085 serves the accepted image. Promotion verifies runtime security, health/readiness,
CSP/cache and exact HTML/JavaScript/atlas equality with the image/local build; one additional
native configured-rules smoke verifies fresh defaults and EN/PL bulk controls. Temporary
Compose projects, namespaces and port-forwards are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.60.yaml`; `rollback-v0.3.59.yaml` retains the prior image.
No migration is required; restarts end process-local matches. The accepted CPU design is next;
this release does not implement single-player mode or finish every quality-review follow-up.

## v0.3.59 — 2026-10-25

[v0.3.59](https://github.com/liveweird/grenadiers/releases/tag/v0.3.59) generates browser item
catalogs from each v7–v11 schema's own `ItemId` enum. Live settings and validation reuse those
tuples; retained validators preserve their narrower activation subsets. V2–v6 generated output,
schema JSON, standalone validators, rules, generation, replay identities and dependencies are
unchanged. An E2E privacy assertion now checks the actual unique secret and redaction markers
rather than rejecting a harmless `/private/tmp` checkout path; sanitizer behavior is unchanged.

Application inputs match `473d1e6e889cfecd5c08178f3b881ca74d775fae`; later source adds only the
E2E test correction and release documentation. Accepted image is
`sha256:946a5f6fbab0c8b573c66badff34157b445cc57cc65798d519bc7fdc6a303fc0`.
Independent catalog review is clear. Fresh full web acceptance passes 6,799 tests / 88 files
in 314.30s; coverage is 90.94/89.49/89.85/93.30% above the unchanged floors. Production gzip
JavaScript/CSS sizes are 336,404/36,292 bytes. All retained generation/standalone drift checks,
API conventions, lint, unused-code, repository and corrected full E2E static checks pass.

Rust behavior is unchanged: reuse v0.3.58's complete 4,194,304-mask debug/release/official-coverage
acceptance (93.50% application lines), plus its shutdown and separate graphics diagnostics.
Dependency inputs are unchanged, so v0.3.57's fresh Cargo-deny and both zero-finding npm audits
remain applicable. The exact new image receives its own scan and runtime acceptance below.

Fresh pinned Trivy 0.74.0 acceptance reports zero fixable HIGH/CRITICAL vulnerabilities.
Native Chromium passes eight default cases through Compose (36.4s) and eight through Kubernetes
(40.3s), with unchanged assertions and no configured retries. The first Kubernetes setup used an
8085 origin allowlist with a browser on 18086; correcting only that isolated environment fixed
session admission. Its original failure log was overwritten; the verifier's command-output
excerpt is retained as a limitation. The initial scan lacked Docker socket access; the corrected
exact-image scan is authoritative. Health/readiness, runtime security and CSP/cache headers pass.

Localhost:8085 now serves that accepted image. Promotion verifies exact HTML/JavaScript/atlas
bytes against the local build and image, runtime security, headers and one additional native
realtime smoke. Temporary acceptance services, namespaces and forwards are removed. Control is
`.local-checkpoints/runtime/compose-v0.3.59.yaml`; `rollback-v0.3.58.yaml` retains the previous
image. No migration or gameplay configuration change is introduced by this release.

## v0.3.58 — 2026-10-25

[v0.3.58](https://github.com/liveweird/grenadiers/releases/tag/v0.3.58) bounds HTTP serving and
application drain with one five-second deadline measured from the shutdown signal. Incomplete
HTTP requests cannot postpone the game/session drain budget. Graceful final state, socket
closure and startup/I/O errors are preserved. On expiry, main returns and runtime teardown
releases remaining asynchronous I/O; this is an asynchronous budget, not a hard real-time bound.
The release also adopts the merged Napoleonic sheet and refreshed gameplay captures without
changing manifest slots, anchors, gameplay or texture budgets.

Application inputs match tested source `e758ad249a066854c35d200cf8e7ce866d8bda74`; the release
source adds documentation. Localhost:8085 serves accepted image
`sha256:cc8f5d0308b8e1d6b774f20dfcee20fdb49691c080e95ac95624d6c2bd3ce0af`.
Independent code, timing and artwork reviews are clear. Nineteen focused shutdown tests pass;
a deliberate nine-second mutant fails all three deadline cases. A real Linux container with
incomplete headers held until process exit terminates successfully in 5.127s with one deadline
warning. A graceful room/diagnostic probe exits in 0.115s, publishes `serverStopped` and closes
member, unseated and diagnostic sockets with code 1001. No frames or credentials are recorded.

Fresh source-bound Rust acceptance completes all 4,194,304 replay masks in debug, release and
official coverage in 433.085s, with zero retries. Each ordinary profile passes 930 core and
293 server tests plus two doctests; four existing performance tests remain ignored and the
heavy case runs separately across 32 complete partitions. Coverage is 17,278/18,480 lines
(93.50%) at the unchanged 90% floor. Rustfmt, Clippy, repository checks and the
[distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40228900957) pass.
The first runner-fixture setup failed from an inherited target-directory override; the unchanged
fixture suite passed with its isolated environment before the complete runner passed.

The fresh web gate passes 6,791 tests / 88 files with 90.95/89.49/89.80/93.32% coverage.
JavaScript/CSS gzip sizes are 336,430/36,292 bytes within their budgets; contract, standalone
validator, lint and unused-code checks pass. Dependency inputs are unchanged from v0.3.57,
whose fresh Cargo-deny and both zero-finding npm audits are reused. A fresh pinned Trivy scan
of this exact image reports zero fixable HIGH/CRITICAL findings under the existing policy.

Exact-image Chromium acceptance passes nine Compose cases (eight default plus Oil Barrel)
and eight Kubernetes cases. Separate diagnostics under the enforced browser policy pass
14 artist, terrain, atlas and isolated renderer-lifecycle cases. These are representative
checks, not a rerun of every configured browser suite. Post-promotion runtime/security,
health/readiness, CSP/cache, exact HTML/JavaScript/atlas equality and one additional browser
smoke pass. Temporary acceptance resources are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.58.yaml`; `rollback-v0.3.57.yaml` retains the previous
image. No migration is needed; restarts still end process-local matches. Keep external stop grace.

## v0.3.57 — 2026-10-25

[v0.3.57](https://github.com/liveweird/grenadiers/releases/tag/v0.3.57) gives shared Rust item/
effect DTOs, common item projections and bounded-number input neutral protocol ownership.
V7–V11 retain their public names and shared type identity. Reserved bits, activation subsets,
validation stages/errors, generation order, schema JSON and replay identities are unchanged.
Four independent regressions pin identity, retained schemas, serialization and numeric boundaries.
Application inputs match `a94aaddea8211e262c1d0acf3cf3103db797db05`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:170e5d071feeabf6d62c55dcb1dce6b6191d7bcdbeccee511e1c9a1c8a72b9bf`.

Independent review is clear; moved bodies and mappings match baseline after path relocation.
Fresh source-bound Rust acceptance passes all 4,194,304 replay masks in debug, release and
reported coverage in 8m06s, without retries. Each ordinary profile passes 930 core and 287
server tests plus two doctests; four existing performance tests remain ignored, and the heavy
case runs separately across 32 complete partitions. Official line coverage is 17,248/18,445
(93.51%) at the unchanged 90% floor, with helper exclusions intact. Clippy/rustfmt, repository
checks and the [distinct Linux units](https://github.com/liveweird/grenadiers/actions/runs/40222285097) pass.

The fresh web gate passes 6,791 tests / 88 files with 90.95/89.49/89.80/93.32% coverage.
JavaScript/CSS gzip sizes remain 336,437/36,292 bytes within their budgets. All retained
contract/standalone drift checks pass. Fresh Cargo-deny, both zero-finding npm audits and the
pinned exact-image scan pass; the latter reports zero fixable HIGH/CRITICAL findings.

The exact image passes twenty native Chromium cases without retries: eight default cases
through Compose, eight through Kubernetes, and seeded Landmine, Heart, Cloak/reconnect/expiry
and Oil Barrel scenarios through Compose. Post-promotion checks pass runtime/image/security,
health/readiness, enforced CSP/cache headers, exact served HTML/JavaScript/atlas equality and
one additional native diagnostic/gameplay smoke. Claude's redraw is unchanged. Temporary
acceptance services and port-forwards are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.57.yaml`; `rollback-v0.3.56.yaml` retains the prior image.
No migration is needed; restarts still end process-local matches.

A separate quiet probe of the unchanged shutdown path in v0.3.56 confirms a follow-up issue:
an incomplete HTTP request keeps the process running beyond seven seconds after SIGTERM,
before the application drain timer is reached. This release does not fix that ordering;
bounding total shutdown is the next isolated fix. Keep the configured external stop grace.

## v0.3.56 — 2026-10-25

[v0.3.56](https://github.com/liveweird/grenadiers/releases/tag/v0.3.56) shares the common
Playwright configuration across 24 simple gameplay wrappers. Public config names and commands,
artifact paths, worker/retry limits and base browser/environment settings are unchanged.
Audio keeps its 60-second timeout; the other 23 keep 90 seconds. Trace capture remains disabled.
Specialized configurations and the pickup/matches/reconnect chain remain separate.
Application inputs match `d7849292fed30d728bffcf0c106aff7d4a1cf6f8`; the source tag adds release
documentation. Localhost:8085 serves accepted image
`sha256:5f1e0627b21d18162c6b12e53516a934ea4d370a0e1a285b75bff2d551a6e603`.

Independent review, fresh types/lint/unused-code/scenario/repository checks and the wired
configuration regression pass. Complete before/after exports match in all 96 comparisons
(24 wrappers across four simulated environment/platform profiles), using pinned Playwright
`defineConfig` and device defaults, including undefined/symbol keys. This source evaluation
preserves Linux ANGLE and capture-profile settings; it is not native Linux browser execution.

The exact image passes nineteen Chromium cases without retries: eight default cases through
Compose, eight through Kubernetes, plus controls, natural headed audio and a seeded Landmine
scenario through Compose. The pinned image scan reports zero fixable HIGH/CRITICAL findings.
Post-promotion checks pass image/runtime security, health/readiness, CSP/cache headers, exact
served HTML/JavaScript/atlas bytes and one additional native diagnostic/gameplay smoke.
Claude's redraw is unchanged. Temporary acceptance services are removed.

Production Rust, API, frontend, asset and deployment sources are unchanged; Cargo changes only
local package versions. Reuse ordinary Rust/Linux results from v0.3.54, frontend/dependency/
renderer evidence from v0.3.53 and exhaustive replay/coverage measured at `055d1d2`
(all 4,194,304 masks, 93.15% line coverage at the 90% floor). These results are not claimed as
freshly rerun for configuration reuse. The new production image is built and accepted separately.

Durable control is `.local-checkpoints/runtime/compose-v0.3.56.yaml`; `rollback-v0.3.55.yaml`
retains the previous accepted image. No migration is needed; restarts end process-local matches.
The testing index/archive cleanup is included; its historical labels do not impose current gates.

## v0.3.55 — 2026-10-25

[v0.3.55](https://github.com/liveweird/grenadiers/releases/tag/v0.3.55) shares native responsive
browser-context setup across 26 gameplay scenarios. Two independent contexts still start
concurrently and return in desktop/mobile order. Viewports, mobile reduced motion, pickup's
390px override, privacy observations, assertions and caller-owned cleanup remain unchanged.
Application inputs match `8b7bacb82de085988e1592a280c0119ba3a41bfa`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:a053abd96387ebe50f85b68aeecffcaa48160ccacb70a2f26f7808c1d7cee6d2`.

Independent review and fresh E2E types, lint, unused-code, repository and scenario checks pass.
The scenario gate includes the new exact-options, concurrency, reverse-resolution ordering and
native-identity regression. All 46 spec title inventories are unchanged; every migrated spec
is byte-identical outside its new import and equivalent setup block. Seventy-four mapped
scenarios remain registered; this static count is not a claim that all ran natively.

The exact image passes twenty Chromium cases without retries: eight default cases through
Compose, eight through Kubernetes, and four seeded Compose cases covering Landmine, 390px
pickups, recipient-private Cloak/reconnect/expiry and Invisible Bombs' live pixel/privacy window.
The pinned image scan reports zero fixable HIGH/CRITICAL findings. Post-promotion checks pass
health/readiness, runtime security, CSP/cache headers, exact served HTML/JavaScript/atlas bytes
and one fresh real-browser diagnostic/gameplay smoke. Claude's graphics are preserved.

Production Rust, API, frontend and asset sources are unchanged from v0.3.54; Cargo changes only
local package versions. Reuse v0.3.54 ordinary Rust/Linux results and v0.3.53 frontend/security/
renderer evidence. Exhaustive replay/coverage remains attributed to measured `055d1d2`
(all 4,194,304 masks, 93.15% line coverage at the 90% floor); none is claimed as freshly rerun
for test setup reuse. The production image is rebuilt and accepted independently.

Temporary acceptance services are removed. Durable control is
`.local-checkpoints/runtime/compose-v0.3.55.yaml`, with `rollback-v0.3.54.yaml` retaining the
previous accepted image. No migration is needed; restarting still ends process-local matches.
Configuration factories and broader fixture sharing remain separate work.

## v0.3.54 — 2026-10-25

[v0.3.54](https://github.com/liveweird/grenadiers/releases/tag/v0.3.54) extends the source-size
guard to every current and future item test tree and aggregates simulation test declarations.
All 926 core test names and all fifteen production module declarations retain their identity.
Application inputs match `c72da79b41b8b8ca83b0396b9cd9f18ab882f77c`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:fd3d8bc815b6169208640b33df4690cb75b7ba98dbcd0d861d087f268760a829`.

Independent review is clear. The repository gate passes eight new filesystem fixtures plus
its existing workflow/dependency checks. Fresh ordinary debug and release runs each pass 926
core and 287 server tests plus two doctests; four existing performance tests remain ignored
and the exhaustive replay is excluded from these ordinary runs. Clippy, rustfmt and the
[distinct Linux unit job](https://github.com/liveweird/grenadiers/actions/runs/40218140208) pass.
The declaration-only change reuses exhaustive debug/release/coverage acceptance measured at
`055d1d2` (all 4,194,304 masks, 93.15% line coverage at the 90% floor). Unchanged frontend,
API, assets and server logic reuse v0.3.53 web, security and renderer evidence; these are not
claimed as freshly rerun v0.3.54 suites.

The exact new image passes eight Chromium cases through Compose and eight through Kubernetes,
with no retries. The pinned image scan reports zero fixable HIGH/CRITICAL findings. Promotion
checks pass health/readiness, image/security identity, CSP/cache headers, exact served
HTML/JavaScript/atlas equality and one fresh real-browser diagnostic/gameplay smoke. The
frontend bytes, including Claude's redraw, match v0.3.53. Temporary acceptance services are
removed; `.local-checkpoints/runtime/compose-v0.3.54.yaml` controls the live image and
`rollback-v0.3.53.yaml` retains the previous image. Matches remain process-local; no migration
is needed.

## v0.3.53 — 2026-10-25

[v0.3.53](https://github.com/liveweird/grenadiers/releases/tag/v0.3.53) adds bounded peer
admission, strict opt-in immediate-proxy identity, outstanding-session quotas and readable,
credential-free denial summaries, alongside the merged arena redraw. Application inputs match
`e1e10b0d889aa110c268bc17167b694423f6c8f2`; the source tag includes release documentation.
Localhost:8085 serves accepted image
`sha256:6035b215bd8ab428a749b9effa63261cc99b2bd4eee3d8bc69cb6c0dfe83ea7f`.

Full source-bound Rust acceptance at `055d1d2` passes every 4,194,304-mask case in debug,
release and official coverage at the 90% floor (7m45s, no retries). Measured application line
coverage is 17,197/18,461 (93.15%). A subsequent logging-target-only correction makes denial
summaries visible under the documented filter: final-source ordinary debug/release each pass
926 core and 287 server tests plus two doctests; clippy and rustfmt pass. The unchanged heavy
replay and coverage results remain attributed to their measured source, rather than claimed
as rerun after that literal change. Distinct Linux debug/release units pass at final source.

After artist PR #123 merges, Rust/Cargo/API inputs remain unchanged and the packaged server
binary is byte-identical to accepted `79a6527` (SHA-256 `13837b25c859c22a7d358c500e86cb304c94732c048823acd11385ae6438b97d`).
The affected full web gate passes 6,791 tests / 88 files at `e1e10b0`, with
90.95/89.49/89.80/93.32% coverage and unchanged floors. Compressed JavaScript/CSS are
336,437/36,292 bytes. Fresh Cargo-deny, both zero-finding npm audits, strict changed Kubernetes
checks and the final image scan pass; the image has zero fixable HIGH/CRITICAL findings under
the existing policy. Independent admission, log-filter and graphics integration reviews are clear.

The final image passes 26 Chromium cases without retries: twelve gameplay cases through
Compose (default eight, controls, audio, two reconnect), six HTTP/WS + HTTPS/WSS CSP cases and
eight through Kubernetes. Three relay/config regressions and fourteen separate enforced-policy renderer/artist cases pass,
including actual exports and an isolated active lifecycle cell. Separate real container TCP
checks prove untrusted forwarding is ignored, trusted malformed forwarding returns 400, sixteen
issued sessions exhaust only their minting peer with 429, and another peer remains eligible.
Actual default-filter summaries are visible and exclude the sentinel request value; unchanged server bytes preserve
that proof across the frontend-only merge.

Promotion health/readiness, exact runtime/image/security identity, real CSP/cache headers,
served HTML/JavaScript/atlas and a fresh browser diagnostic/gameplay flow pass. The served redraw
matches canonical SHA-256 `b057bafe7b32b76dd187fa44bf4bd6ebbb686b1aa0e8bbf8c5f633aea7ac7a6e`. Proxy trust is
empty for the direct local deployment. Temporary services, keys, worktree and branch are removed.
Durable control is `.local-checkpoints/runtime/compose-v0.3.53.yaml`; v0.3.52 remains for rollback.
Reload browser tabs; matches and quotas remain process-local. No database migration is needed.

These quotas are engineering defaults, not durable identity or a public deployment design.
Actual proxy addresses, overwrite behavior, restricted backend access, TLS and ingress limits
remain operator requirements. Claude's redraw is included; its manifest/frame geometry is unchanged,
and parity explicitly uses the packed procedural baseline while lifecycle tests load shipped art.

## v0.3.52 — 2026-10-25

[v0.3.52](https://github.com/liveweird/grenadiers/releases/tag/v0.3.52) enforces a compatible
browser content policy using static Pixi implementations and precompiled validators.
Application inputs match `578f581be1593b56ab589c6a8da2dce8a8e4fc8f`; the source tag includes
release documentation. Localhost:8085 serves accepted image
`sha256:9b5918458c30fd8dc8e877c8430607f09dd773abdec52f407bc139e5b58ddb85`.

Full source-bound Rust acceptance passes all 4,194,304 masks in debug, release and official
coverage at the 90% floor, with no retries. Application line coverage is 16,394/17,634 (92.97%).
The parallel gate phase completes in 7m23s while the web gate and image build also run. Web
passes 6,790 tests / 88 files with 90.95/89.49/89.78/93.32% coverage and unchanged floors;
compressed JavaScript/CSS are 336,413/36,292 bytes, within budgets. E2E static checks, three wired
relay/config regressions, the pinned image scan and the distinct Linux Rust unit job pass.
The image has zero fixable HIGH/CRITICAL findings under the existing scanner policy.

Before server delivery, enforced policy trials pass fourteen renderer/artist cases and controls,
audio and both reconnect cases with zero observed violations. Final acceptance uses the actual
server image without relay policy injection: eight native cases through Docker Compose, eight
through Kubernetes and six HTTP/WS + HTTPS/WSS CSP cases pass, with no retries. Real page scripts
prove execution/connection/worker blocking; the frame test isolates CSP from XFO and includes
an attachment control. An earlier relay reset crash and no-test selector are retained as failed
attempts, not passes. Independent review is clear.

Promotion health/readiness/index and a fresh native browser diagnostic/gameplay flow pass.
Served HTML/JavaScript/atlas bytes and runtime security identity match the accepted artifact;
CSP is enforced, including on API responses. Temporary acceptance services are removed.
Durable control is `.local-checkpoints/runtime/compose-v0.3.52.yaml`; v0.3.51 is retained for
rollback. Reload existing tabs; matches remain process-local. No migration is needed.

The style policy explicitly permits Mantine inline styles; scripts do not allow inline
execution or string compilation. Browser acceptance covers Chromium, not an untested wider
matrix. Peer-keyed hosting admission and the separate artist redraw remain future releases.

## v0.3.51 — 2026-10-25

[v0.3.51](https://github.com/liveweird/grenadiers/releases/tag/v0.3.51) precompiles the live
browser validators, preserving schemas, complete errors and semantic/privacy checks.
Application inputs match `a7ac65b36592ad8871a1a78f64f4e9adcc24eb36`; the source tag includes
release documentation. Localhost:8085 serves the accepted image
`sha256:cb7be95e9b5c6c291d8c828579b3de5facff9efb466d5a61e4dcdcadd2c7c76f`.

The full web gate passes 6,789 tests / 88 files at `31219c0`, with coverage
90.95/89.49/89.78/93.32% (statements/branches/functions/lines). Later portability corrections
change generated comments only; 13 focused tests, lint, Knip, drift, TypeScript and fresh
production builds pass on the final generator. Exact artifacts match across checkout depths,
invocation directories and pinned Linux. Compressed JavaScript falls from 357,324 to 333,442
bytes; CSS remains 36,292 bytes, within unchanged budgets. Independent review is clear.

Actual Chromium development and production flows validate live diagnostic welcome/pong and
gameplay admission. Sixteen native cases pass without retries: the default eight each in
Docker and Kubernetes on the same image. Kubernetes retains base security, probes and resource
limits. Strict schema checks and the pinned image scan pass, with zero fixable HIGH/CRITICAL
findings under the existing scanner policy. Isolated acceptance services are removed.

Rust source, wire contracts and external dependencies are unchanged from v0.3.50. Reuse its
full exhaustive/coverage acceptance; Linux debug/release units additionally pass at `b366958`,
whose Rust inputs equal the final candidate. Do not treat these reused results as a new full
Rust run. Docker's clean dependency install reports zero npm vulnerabilities.

Promotion health/readiness/index are 200. Served HTML/JavaScript/atlas match the image; the
atlas still matches canonical source. Runtime hardening remains intact. Durable deployment
control is `.local-checkpoints/runtime/compose-v0.3.51.yaml`; v0.3.50 is retained for rollback.
Reload browser tabs after upgrading; matches remain process-local. No database migration is
needed. CSP enforcement and the separate artist redraw remain pending.

Runtime attribution correction recorded with v0.3.52: the eight Docker cases used direct
`docker run`; the separately verified localhost promotion used Docker Compose. The original
source tag remains unchanged.

## v0.3.50 — 2026-10-25

[v0.3.50](https://github.com/liveweird/grenadiers/releases/tag/v0.3.50) hardens transport,
room recovery, browser responses and contributor checks. Application inputs match tested source
`9122e330379f4ac1bb3e0edd236dcdc5538655f0`; the source tag identifies the complete release.
Localhost:8085 serves the exact accepted image
`sha256:76980168a517474e680d8e346b6b8c38240a1b534eb403ad919b347168d0f5c6`.

Full Rust acceptance covers all 4,194,304 masks in debug, release and official coverage with no
retries (6m40s). Application line coverage is 92.97%; that fresh 80%-gated measurement supports
raising the Rust floor to 90%, verified by reporter contract/real-LLVM tests. The patched web gate
passes 6,780 tests in 86 files at the raised 90% line / 88% other floors. API/E2E static checks,
Cargo-deny, zero-finding full/production npm audits and the pinned image scan pass; the image has
zero fixable HIGH/CRITICAL findings under the existing scanner policy.

Eighteen native cases pass on this image: sixteen Compose cases covering the default suite,
controls, audio, match progression, reconnect/deadline, Cloak, Stopwatch and Oil; two Kubernetes
reconnect cases retain the base pod security/probes/resources. Failed sandbox launches, the
socket timing regression and the stale Equipment-error assertion were corrected and are not
carried as passes. Fresh accepted suites use no retries.

Promotion health/readiness/index are 200; served HTML/JavaScript/atlas bytes match the image,
and atlas pixels match canonical source. UID65532, read-only filesystem, dropped capabilities
and no new privileges remain. v0.3.49 is retained for rollback. Runtime control files now live
in `.local-checkpoints/runtime/`, rather than `/tmp`. Reload existing tabs; active matches and
capabilities remain process-local. No database migration is required.

The artist redraw remains a separate pending change. Further CSP, hosting and structural work
is tracked in the [quality-review response](../quality-review.md).

## v0.3.49 published prerelease — 2026-10-25

[v0.3.49](https://github.com/liveweird/grenadiers/releases/tag/v0.3.49) is published from source
`4157d534553330b457871554d84e973823068798`, tree `dd6a844d841e76e254ee09b339a4e4c6577f982f`,
with remotely verified annotated tag object `1a8f141adbf6c18e372a1b6dd344475727664233`.
Localhost:8085 serves exact accepted image
`sha256:10aa8554f8f22f968a6bd1852a4c0d6fe43e63b365eafc8220e7fbd826dc4d21`.

Promotion exits zero; health/readiness/index return 200. Served HTML, JavaScript and the hashed
atlas PNG match container bytes; the PNG also matches canonical source SHA256
`f2392294ba67207cdaf264ec3343cf096de0281dc281aa4c8c3339ff312d407f`. UID65532, read-only
filesystem, dropped capabilities and no new privileges are retained. v0.3.48 remains the rollback
image. An initial local helper path typo stopped before any service change; the corrected helper
completed promotion and all post-deployment checks. Application inputs match accepted source
`ee664e5`; publication documentation does not require rebuilding the accepted image.

The artist handoff/export/preview and gameplay atlas integration are now complete. New artist
redraws remain separate work. The documented completed-icon pulse compositing is intentional.

See the [renderer guide](../game/art-atlas.md) and [acceptance record](verification.md#v0349-gameplay-atlas-acceptance--2026-10-25).

## v0.3.48 published prerelease — 2026-10-24

[v0.3.48](https://github.com/liveweird/grenadiers/releases/tag/v0.3.48) is published from source
`f6336810f4a539e5e001aa6b1733f445015ae76f`, tree `87a488cc69b45e61b40b8be340f7b09ea4347930`,
with remotely verified annotated tag object `372e84642c4e582c4a632508acb2deeaa9a89aba`.
The uploaded 229,324-byte artist handoff ZIP has verified SHA256
`1f374dc6a76a05b536decc9620bf43edd448d9cc242bf0b15d4b1e28eed4b135`.

Localhost:8085 serves exact accepted image
`sha256:751383fe17e23a6ae46e58006017a15862c6b0cf2fd993645bb2c78187b00cb8`.
Promotion exits zero; health/readiness/index return 200 and served HTML/JavaScript match container
bytes. UID65532, read-only filesystem, dropped capabilities and no new privileges are retained.
Oil v0.3.47 is preserved as the rollback image. Application inputs match tested source `cd369f4`;
subsequent gate/documentation changes do not require rebuilding that image. Gameplay adoption
remains v0.3.49; current gameplay continues using the existing code-generated drawings.

See the [artist guide](../../assets/art/README.md) and [acceptance results](verification.md#v0348-artist-handoff-acceptance--2026-10-24).

## v0.3.47 published prerelease — 2026-10-24

[v0.3.47](https://github.com/liveweird/grenadiers/releases/tag/v0.3.47) completes all 22 requested item types. Release source
`494f620cf5eb13f0f18d0bf13d91bc74291bb51b`, tree `613e1ccd86fa14e3a94af50707a23be4044d70d2`, is tagged by verified annotated
object `565165ae9b5ce2c968fbb83b3ce413b5f5a562e8`. Localhost:8085 serves the exact accepted image
`sha256:969ff363f77ad6dfc6e62fa5f98e4a704d017ad1186d43fe774fedc50fd7561e`. Promotion exits zero; health, readiness and index
return 200, and served HTML/JavaScript match container bytes. The application retains UID65532,
a read-only filesystem, dropped capabilities and no new privileges. Bracelet v0.3.46 is retained
as the rollback image. Reload older browser clients for gameplay v11.

## v0.3.46 published prerelease — 2026-10-24

[v0.3.46](https://github.com/liveweird/grenadiers/releases/tag/v0.3.46) publishes Bracelet as the twenty-first requested item from source
`7b4b47cfdd7dea9efec9d3c2f2fd61f2c232ce54`, tree `7816fe3d0a373523ffe5b4a8b54873dfce3e3b7a`, through verified annotated tag
`57b9b226a2d1cc07ebe1b8487348e89858678132`. Localhost:8085 serves the exact accepted image
`sha256:1460278198b6d604595bfbe4fb8278a85ab39725c43516bb8b55ab2e6a47f5bf`. Promotion exits zero; health, readiness and index
return 200, and served HTML/JavaScript match container bytes. The application retains UID65532,
a read-only filesystem, dropped capabilities and no new privileges. Punch v0.3.45 is retained
as the rollback image. No rebuild or repeated full CI was needed.

## v0.3.45 published prerelease — 2026-10-24

Source `5be42456cee80e06e2f138bdfd458ef821bb34c2` is published as
[v0.3.45](https://github.com/liveweird/grenadiers/releases/tag/v0.3.45) through annotated tag object
`0e6224643ce85bf8f0a77894dcb929ab2655e397`. It publishes Punch as the twentieth requested item.
Localhost serves the accepted image
`sha256:3f13b26f7a4a41ba2e90b8cc9261c9e95a60eceeddf49504341e65258eb7d37d`.
Bracelet v0.3.46 is now published and promoted, as recorded above.

## v0.3.46 Bracelet acceptance checkpoint — 2026-10-24

Scoped native acceptance passes all 29 cases (28 Compose plus one Kubernetes), with zero retries.
The 22 isolated latency cases pass with maxima 62.93 ms local and 160.14 ms with shaped 50 ms
delay, below the unchanged 250 ms budget. Accepted image:
`sha256:1460278198b6d604595bfbe4fb8278a85ab39725c43516bb8b55ab2e6a47f5bf`. Publication and
localhost promotion subsequently completed; see [verification](verification.md).

## v0.3.44 published prerelease — 2026-10-24

Source `f32fd86d8f0c7c4bebba8fa60d1c66c65d1a174a`, tree
`a0f2addb9204c40b4b30be2830fde4df368f019c`, is published as
[v0.3.44](https://github.com/liveweird/grenadiers/releases/tag/v0.3.44) through annotated tag object
`c5b1ab6d4cad94ba2f16614103903067745c5179`. It publishes Kick as the nineteenth requested item.
Localhost serves the accepted image
`sha256:8ff3005861127f8ac57cc32f7e6c72ce5fa996fe628ad48426dbb8cf133e1af9`.
This is the published parent recorded for the historical Punch candidate below.

## Historical v0.3.44 Kick native-acceptance checkpoint — 2026-10-24

Kick candidate source `cb70cba891ed972380f30f52133622ce36fd9ee8`, tree
`5be27219ba749d2811396d6f84af7d31f528a692`, passes fresh source-bound Rust, web, E2E and security
gates. Every Rust profile passes 763 core and 210 server tests with four documented ignores and
enumerates all 524,288 settings masks; debug and release each pass two doctests. Rust line coverage
is 15,233/16,429 (92.720190%). The web gate passes 1,059 tests across 66 files with
95.26/90.9/96/97.67% coverage and 57 static scenarios. Configured security retains three affected
moderate packages represented by six GHSA advisories, with no high or critical findings; E2E audit
reports zero vulnerabilities. Gate receipt
`903736dda1001d60f83204fa10f58940796c87716a5d9eea151b58e62fd97153` binds actual-zero roots
`31818` and `5290`.

Fresh image `sha256:8ff3005861127f8ac57cc32f7e6c72ce5fa996fe628ad48426dbb8cf133e1af9`
passes all 56 Compose plus 56 Kubernetes cases with zero retries or carried results. Root `97948`
finishes with consumed actual exit zero; the matrix retains 54 reports, 126 PNGs, 44 runtime
records, 22 distinct Kubernetes identities, 21 fixtures and 42 seed records. Each target passes 22
latency cases and retains 80 game observations under the unchanged 250 ms budget. The earlier
failed attempt's 55 cases are not carried.

Facts receipt `00542824d775035473b8a2c6bc15ea54d176dbf990c8a2e5004dd28643746915`
records this completed execution, and cleanup receipt
`c69c1d93de5be0c4e4c4b9dfe6214175fa5b63041aa7e393ce5b5d2a479cff5a`
records owned-resource removal. Audit `58e03d36c7dc967e71342e0e70e1fb65ac045e0f5a2883800d984d83bfe4654c`,
result review `8f9f1e7d520a3da7baf52344c551545f6903310af458fdfe6b90465c6d406594`
and proof-binding review `24edf45899b2402c86204eec7516e3a31b39bba8b2704df0a6d2d3b303466dea`
are clear. Final manifest `a5d0c54f699629ea732c3f641bad02bd322ebb8c0f1a4a0efd1df293c4126620`
is durably bound by seal `94c60dac44c639f7e9a214061e9daf9475628126d15526346ed7be2ce4c9a947`;
independent post-seal review `97c1b19ff6694746b73ac9d427f30f4840f14300967e4333fed9d634fb05fd51`
is clear.

At this dated acceptance checkpoint, candidate/master/tag CI, publication and local promotion have
not run. v0.3.43 remains the verified published/deployed baseline for this checkpoint; consult
[GitHub Releases](https://github.com/liveweird/grenadiers/releases) for authoritative publication
status after it.

## v0.3.43 published prerelease — 2026-10-24

Published source `c086f719ea470b9fe24963adef84b5e3bdf3e17a`, tree
`d74fad8a1c870f2834fca48f80ea4e38f0c512d9`, is available as
[v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43) through annotated tag object
`c8395fd7dcc5701842644967802e61c76211931c`. Candidate push/PR runs `40051959641` and
`40051963999`, master run `40053637247`, and tag run `40055400664` each pass all ten jobs, 55
browser cases and all 262,144 masks in debug, release and coverage on their first attempts.

Localhost serves accepted image
`sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01` from container
`d7f68cc89bfc87d06c202fbedae78c1892f54ea3ccdc6571fee0d1c5ea2f4240`; health, readiness and
index return 200 and the served script SHA-256 is
`61d6722543d93559ad980b2f6577897ddbdfcc80b4137d598d7aecf75c6ea637`. The v0.3.42 image is retained
as the rollback parent. Delivery contract
`f5696ccb5aba2633f3ecbf7a12353aa5073d61218cf0f717807e516bd69bbeb4`, independent release review
`da546c5038b3dc514b03a1b3fef658f828b27557c629082d95f2ccd368ba9889`, delivery binding
`40578e13d8a44a3cdb22bc222ac71eedd0005b08f3fb5f0748d07f7e333cf5d1`, and binding review
`21f6b544ee9d8e906c056671242c6be73ba742d1bcecb55f39c804ae78d9ec54` are clear.

Native execution remains attributed to source `11d0dc7e39e8e2cafcf70c1548be339f52c740ab`, seal
`31fe5eef2900a0229d002e45b5a310888f3915829db0aaf23bc3535a8eea5e2c`, and final manifest
`3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c`. Artifact-equivalence receipt
`81bd78943fa8e80bc839b06bcd68c7bb26872f4cf600c2fcc80e2269c7c94209` and review
`42bbbcb68e8a306d9ed611513e186189ab3430f17bf545191edace5bdb6e3b75` connect corrected source
`5dbcd58` to the published source through eight documentation-only changes. They do not claim
unchanged Docker inputs, a new Rust compilation, or new native execution. Eighteen of 22 requested
items are published at this checkpoint. Kick is the v0.3.44 native-accepted candidate; hosted CI
and publication follow.

## Historical v0.3.43 Wall Pass native checkpoint — 2026-10-24 01:00 UTC

Wall Pass is the eighteenth requested item and is native verified at source
`11d0dc7e39e8e2cafcf70c1548be339f52c740ab`, tree
`655c7d8a80b4dd3fdd3b232534c2cbaa45b054cc`, on exact image
`sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`.
Fresh gates pass 711 core and 209 server tests with four historical ignores in each Rust profile,
797 web tests across 64 files, 56 scenarios and all 262,144 masks in debug, release and coverage.
Rust line coverage is 92.358954%; web statement/branch/function/line coverage is
95.24/90.88/96/97.66%. The configured security commands pass; the web inventory retains two
moderate development advisories and no high or critical advisories.

Native root `45615` completes with consumed exit zero: all 55 Compose plus 55 Kubernetes cases,
52 reports, 122 PNGs, 42 runtime records, 21 Kubernetes identities, 20 fixtures, 40 seed records
and 80 latency observations below 250 ms pass with zero retries or carried results. Independent
audit `25e48638872f7b61fd64c28af30abf5e44910cf29e3e8356246236991819380e` and result review
`c8652f4e67852002db9e49e2766481e6c58b071d3e93bbb7866bfd6c5194848d` are clear. Binding review
`7c6eda746a703f599ced928798750f05478b78dbf5fc387d03fbe5adac551d46` is clear. Sealer root
`96118` completes with consumed exit zero; final manifest
`3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c` authenticates 2,665
payloads, 2,666 files and 284 directories. Seal
`31fe5eef2900a0229d002e45b5a310888f3915829db0aaf23bc3535a8eea5e2c` is durable; independent
post-seal review `5d253559ba95a9d30fc83f16fd0786a6d283356d892f26abed3611771588d33f`
is clear.

The source includes the reviewed Zod 4.6.6 lockfile correction and parallel independent English
match-result inspections without changing the 43 assertions, three screenshots, fixtures or
budgets. The descendant release candidate adds CI-only procfs short-read and external-target
coverage-parent fixes in commit `dee44b5ca81d2556d775a54e47948da77b9dbaa3`. Its production and
Docker build inputs remain byte-identical to the sealed native source. Those fixes improve
diagnostics and gate setup without claiming that a particular Chrome I/O cause was fixed.

Latest corrected candidate `5dbcd58e920f852a8036c4bf741e26517793b072` pins Happy DOM 21.0.4 to
resolve GHSA-834v-8h66-3h79. Fresh local verification passes the full 797-test, 64-file web gate
with unchanged 95.24/90.88/96/97.66% coverage and the complete configured security gate: cargo
deny plus both npm audits exit zero, with three moderate development advisories
(`@vitest/mocker`, `vitest` and `fast-uri`) and zero high or critical findings. Receipt
`0a8c605c65bdceff9d04d74752aaafc3ee5e9ee2378e60805a7fecb8cc216ce0` binds the web check, and
`e0c7363348d4f4092bc0afd3037068e183b78bc99c0969a6a2ef7b6451fdceb6` binds the full security
result. The two web manifests are the only changed Docker inputs: 554 of 556 entries remain equal.
The complete host distribution still matches the accepted image byte-for-byte across 17 entries
(15 files and two directories), using retained server binary
`26bccf1696b75b33b741b43305429df92cd09870d02d1080b725e05665e3966d`; a fresh scan of that exact
image passes current policy. Equivalence receipt
`81bd78943fa8e80bc839b06bcd68c7bb26872f4cf600c2fcc80e2269c7c94209` and independent review
`42bbbcb68e8a306d9ed611513e186189ab3430f17bf545191edace5bdb6e3b75` are clear. This is retained
artifact equality, not unchanged build inputs, a new Rust compilation, or Docker-build
reproducibility; the 110 native cases remain attributed to source `11d0dc7` and accepted image
`577abb3`.

At this checkpoint, the first PR #104 head `98dffe5` failed security in push run `40050321682` and PR run
`40050338206`; other jobs were still finishing at that checkpoint. Corrected head `5dbcd58` has
not yet been pushed. Those then-pending steps were later completed by the published delivery above.

This section is the dated native checkpoint, not the current release status. The published record
above supersedes its pending statements. Kick, Punch, Bracelet and Oil Barrel remain before
ART-ATLAS.

The reviewed Wall Pass captures are [English desktop](../screenshots/wall-pass-desktop.png) and
[Polish 320px reduced motion](../screenshots/wall-pass-mobile-pl.png), with SHA-256 values
`d46367b7eeee7c55e94dbcfba6de7de50b27b96c6761ad6a9ff69db2b9332c08` and
`84843190d2be59a89f5c614c3d17bd78e969dcb7cf692d0e06243b2d3978d72f`.

## v0.3.42 published prerelease — 2026-10-23

Merge `0ff1512c5a8fdbd1e1b16583ae7791e668bf070b`, tree
`b4201abe9546d22b26b3231506d1e2ab6b1d3613`, is published as
[v0.3.42](https://github.com/liveweird/grenadiers/releases/tag/v0.3.42) through annotated tag object
`ea3930a3490fb483de46692e1009743e20c61648`. Master run `39982027955` and distinct tag run
`39984896509` each pass all nine jobs, 54 browser cases and 131,072 settings masks on the first
attempt, with independent reviews clear. Promotion root `84311` completes with consumed exit zero;
container `0a3bcbac08bc2c2cf46365917a3719d3c5e522c4555234da8e1f661f6edcf214` serves exact native image
`sha256:8ae9818d27c4345bceec6dad77f02e21ee7148626efded48c421d86421285477`. Seventeen of 22 requested
item types are published. Wall Pass was the next source candidate at this delivery checkpoint; its
later native acceptance is recorded above without changing v0.3.42's published history.

### Historical native-accepted source checkpoint — `2026-10-23 04:37 UTC`

Source `880fff7c67269f2886746b45239cd888576832f4`, tree
`c1da58099bb00bb0c99b4bd00de38e9c0c6f87d6`, and image
`sha256:8ae9818d27c4345bceec6dad77f02e21ee7148626efded48c421d86421285477` pass 54 Compose plus 54 Kubernetes cases with zero retries or
carried passes. Independent audit `0dbb15b27d5a23eae680ca9cfa2d9766a7e64e173f5f8652fd753b4611135521` and result review
`cf3380353ca06f36cc61f67c8898ed766d6263bdc6108e003a0a9ac17da4923c` are clear; durable manifest `d81c0923f5a3d0a50208fe43a12412f60ca09e2fddfc1c6aabfb793779676b7f`
authenticates the sealed evidence. Source gates cover 689 core, 208 release server and 657 web
tests, 55 static scenarios and all 131,072 settings masks in debug, release and coverage profiles.
At this checkpoint, hosted candidate, post-merge, tag, publication and local promotion remained
pending. The delivery record above documents their later completion.

The reviewed English desktop and Polish 320px reduced-motion Teleporter captures hash to
`e262534f6c157678f762519aa03cc2ba7896139f04d9002c1347f68bc12cff63` and `30f9e9cd099a2b01ae41c9c64096cb15b50b4850b5198cea54ad8b2eaddd63ba`.

## v0.3.41 published prerelease — 2026-10-23

Commit `f5c4adf66f8af9a5ef66b48f9230d17aea14f1d5`, tree
`a274fb2fe828f3a18a624614f1addf5bf5df4d99`, is published as prerelease tag `v0.3.41`.
Post-merge run `39879618476` and distinct tag run `39881306987` each pass all nine jobs,
53 browser cases and 65,536 settings masks. The published source retains the accepted application,
E2E and native inputs, while the CI-only browser readiness and observer changes retain their own
reviewed identity. The sealed native image
`sha256:d063462c3130e3c8653734da72e8dfd6785b0c036e9579d9046cee884ccd9f82` is promoted
locally with consumed exit zero. At that release checkpoint, sixteen of 22 requested item types
were published; the v0.3.42 delivery record above documents Teleporter's later publication.

## v0.3.41 browser file readiness checkpoint — 2026-10-23

The observability-only PR 92 was closed after its required PR workflow failed
while starting Chromium; its parallel successful push run supplies no acceptance
for a later source. A bounded pre-read of the installed browser files targets the
observed kernel page waits. Additional fault, I/O and pressure counters preserve
useful evidence without asserting an exact file or storage cause.

At this historical checkpoint v0.3.41 was unpublished and localhost:8085 served v0.3.40.
The application, E2E and native inputs retain H's accepted bytes; the native
106-case execution remains labeled H. Require independently reviewed input
identity and fresh push, PR, post-merge and tag checks for the meaningful CI
revision before promoting H's sealed image. See the [verification record](verification.md).

## v0.3.41 observability checkpoint — 2026-10-23

Invisible Bombs was merged in PR 91, but post-merge workflow `39870771097` failed
while starting the headed visibility browser. At that recovery checkpoint, v0.3.41
was unpublished and the local deployment served v0.3.40. The CI-only observer revision gathers bounded
process and Xvfb diagnostics without changing gameplay or test budgets; a passing
instrumented run does not establish the cause of the intermittent failure.

The recovery rolled forward from merged commit `2fbc782` while keeping version 0.3.41.
Native proofs retain their original source identity under independently reviewed input equality.
Failed hosted runs contribute zero accepted cases. See the
[historical investigation](verification.md#v0341-post-merge-startup-investigation--2026-10-23).

