# Releases

Release labels and scope live in the [implementation plan](game/implementation-plan.md).
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
Use patch releases within the current milestone for bounded engineering increments.
`0.2.0` is the first playable duel preview; `0.3.0` adds classic match progression. Subsequent
`0.3.x` increments stage reconnect support and activate it at `0.3.9`; `0.4.0` remains the resilient playtest milestone. Keep PRs focused on one behavior and its tests. Merge passing,
independently reviewed candidates, then publish the corresponding prerelease tag.
The repository gate checks that both local Cargo packages and their dependency agree.

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
compare its Docker build inputs with the final PR head, require every configured push/PR check to pass,
and verify that the merged tree equals that head. Use the corresponding release tag for the
published source. Retain the accepted image when exact artifact reuse is required; a later
rebuild can receive different operating-system security updates and needs its own acceptance.

## Upgrade and rollback

The initial deployment uses one replica and `Recreate`. In-memory matches cannot survive
restart or rollback. Plan upgrades around active matches; a graceful stop publishes the final
interrupted state but cannot preserve process-local matches or credentials.
Keep the previous immutable image reference for rollback; database migration compatibility
must be designed when durable application data is introduced.

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
