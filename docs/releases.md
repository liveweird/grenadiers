# Releases

Planned release scope and work status live in the [backlog](../BACKLOG.md).
The [implementation plan](game/implementation-plan.md) defines the release checklist and future tracks.
A completed milestone is verification evidence; a release exists only after its tested commit
is tagged and its artifacts are published intentionally.

## Release verification policy

For this single-maintainer project, local checks are as authoritative as equivalent remote
checks. Ship the tested source and reuse its verified artifacts. Remote checks are required only
when they add coverage not exercised locally; a tag pointing at an already verified commit does
not require another complete suite.

Record the version, tested commit, applicable checks and outcomes, runtime environment and image
digest in the release notes. Confirm the tag targets the intended commit and check health,
readiness and the served artifact after deployment. Repeat affected checks when inputs change;
do not require separate full push, PR, master and tag runs or elaborate new proof packages for
each feature. Historical release records retain their original evidence and requirements.

## Version ownership

Use Git tags `vMAJOR.MINOR.PATCH` for the combined server/web product. Cargo workspace package
versions track the next product candidate; keep the local
`grenadiers-core` dependency version aligned when changing them. Private npm package versions
remain `0.0.0`: they are build inputs and are not separately published or product release IDs.
The WebSocket `protocolVersion` is an independent compatibility contract, not a release number.

Each shipped feature PR bumps the product version and records its scope in `CHANGELOG.md`.
Use patch releases for bounded engineering increments; historical milestone labels are preserved
in the [plan archive](history/implementation-plan.md). Keep PRs focused on one behavior and its tests. Merge passing,
independently reviewed candidates, then publish the corresponding prerelease tag.
The repository gate checks that both local Cargo packages and their dependency agree.

### Release preparation helper

Stage the candidate notes under `CHANGELOG.md`'s Unreleased heading. Set `NEXT_VERSION` to the
chosen newer product version and `RELEASE_DATE` to its `YYYY-MM-DD` date, then preview:

```sh
node scripts/prepare-release.mjs --version "$NEXT_VERSION" --date "$RELEASE_DATE"
```

The default is a dry run. Add `--write` only after reviewing the plan; it updates the Cargo
workspace and local-package lock versions, then moves the Unreleased notes into the requested
dated release while retaining an empty Unreleased section. It does not run checks or perform
Git, tag, publish, or network operations. Run the applicable release checks separately.

## Candidate checklist

1. Update package metadata, lockfiles, contracts, and affected documentation intentionally.
2. Run the complete [release checklist](game/implementation-plan.md#release-checklist), including
   coverage, dependency audits, the image scan, real browser scenarios, and Compose/Kubernetes
   runtime checks. Record commands, results, image identity, and limitations in
   [verification.md](verification.md).
3. Obtain independent review of substantive changes. Commit a focused candidate and rerun
   affected checks if the reviewed source changes.
4. When publishing is authorized, tag that tested commit and promote the exact tested image;
   avoid rebuilding an unverified replacement under the release tag. Include supported rules,
   preview limits, and upgrade notes in the release notes.

## Current prerelease distribution

Current releases publish source tags and GitHub prerelease notes. Accepted image IDs in the
verification record identify locally tested artifacts; no production registry or hosted service
has been configured. A source prerelease does not claim a production deployment.

Evidence may name a clean candidate commit from before rebase or squash. Before merging,
compare its Docker build inputs with the final PR head, require the applicable local-first checks to pass,
and verify that the merged tree equals that head. Use the corresponding release tag for the
published source. Retain the accepted image when exact artifact reuse is required; a later
rebuild can receive different operating-system security updates and needs its own acceptance.

## Upgrade and rollback

The initial deployment uses one replica and `Recreate`. In-memory matches cannot survive
restart or rollback. Plan upgrades around active matches; a graceful stop publishes the final
interrupted state but cannot preserve process-local matches or credentials.
Keep the previous immutable image reference for rollback; database migration compatibility
must be designed when durable application data is introduced.

## v0.3.71 — 2026-10-26

[v0.3.71](https://github.com/liveweird/grenadiers/releases/tag/v0.3.71) adds isolated owner,
registry and entropy measurements plus real non-reading TCP regressions. The regressions verify
write recovery, bounded shutdown, healthy-peer delivery and cleanup. Production registry locking,
entropy boundaries and timeout values remain. The [measured reference](history/registry-admission.md)
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

<a id="v0370--2026-10-26"></a>
The [v0.3.70 record](history/releases.md#v0370--2026-10-26) is preserved in release history.

<a id="v0369--2026-10-26"></a>
The [v0.3.69 record](history/releases.md#v0369--2026-10-26) is preserved in release history.

<a id="v0368--2026-10-26"></a>
The [v0.3.68 record](history/releases.md#v0368--2026-10-26) is preserved in release history.

<a id="v0367--2026-10-26"></a>
The [v0.3.67 record](history/releases.md#v0367--2026-10-26) is preserved in release history.

<a id="v0366--2026-10-26"></a>
The [v0.3.66 record](history/releases.md#v0366--2026-10-26) is preserved in release history.

<a id="v0365--2026-10-26"></a>
The [v0.3.65 record](history/releases.md#v0365--2026-10-26) is preserved in release history.

<a id="v0364--2026-10-26"></a>
The [v0.3.64 record](history/releases.md#v0364--2026-10-26) is preserved in release history.

<a id="v0363--2026-10-25"></a>
The [v0.3.63 record](history/releases.md#v0363--2026-10-25) is preserved in release history.

<a id="v0362--2026-10-25"></a>
The [v0.3.62 record](history/releases.md#v0362--2026-10-25) is preserved in release history.

<a id="v0361--2026-10-25"></a>
The [v0.3.61 record](history/releases.md#v0361--2026-10-25) is preserved in release history.

<a id="v0360--2026-10-25"></a>
The [v0.3.60 record](history/releases.md#v0360--2026-10-25) is preserved in release history.

<a id="v0359--2026-10-25"></a>
The [v0.3.59 record](history/releases.md#v0359--2026-10-25) is preserved in release history.

<a id="v0358--2026-10-25"></a>
The [v0.3.58 record](history/releases.md#v0358--2026-10-25) is preserved in release history.

<a id="v0357--2026-10-25"></a>
The [v0.3.57 record](history/releases.md#v0357--2026-10-25) is preserved in release history.

<a id="v0356--2026-10-25"></a>
The [v0.3.56 record](history/releases.md#v0356--2026-10-25) is preserved in release history.

<a id="v0355--2026-10-25"></a>
The [v0.3.55 record](history/releases.md#v0355--2026-10-25) is preserved in release history.

<a id="v0354--2026-10-25"></a>
The [v0.3.54 record](history/releases.md#v0354--2026-10-25) is preserved in release history.

<a id="v0353--2026-10-25"></a>
The [v0.3.53 record](history/releases.md#v0353--2026-10-25) is preserved in release history.

<a id="v0352--2026-10-25"></a>
The [v0.3.52 record](history/releases.md#v0352--2026-10-25) is preserved in release history.

<a id="v0351--2026-10-25"></a>
The [v0.3.51 record](history/releases.md#v0351--2026-10-25), including corrected Docker/Compose
runtime attribution, is preserved in release history.

<a id="v0350--2026-10-25"></a>
The [v0.3.50 record](history/releases.md#v0350--2026-10-25) is preserved in release history.

<a id="v0349-published-prerelease--2026-10-25"></a>
[v0.3.49 published prerelease](history/releases.md#v0349-published-prerelease--2026-10-25)
is preserved in the release history.

<a id="v0342-published-prerelease--2026-10-23"></a>
Older release records, including [v0.3.42 published prerelease — 2026-10-23](history/releases.md#v0342-published-prerelease--2026-10-23),
are preserved in the [release history](history/releases.md).
