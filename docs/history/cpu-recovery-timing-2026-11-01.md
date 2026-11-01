# Recovery checkpoint timing experiment — 2026-11-01

This experiment establishes scheduling facts for the
[continuation design](../game/single-player-cpu.md#interrupted-escape-continuation-boundary).
It does not implement CPU recovery or certify movement safety. Delivery status remains in
[BACKLOG.md](../../BACKLOG.md#single-player).

## Scope and results

The [test-only patch](fixtures/recovery-checkpoint-timing-v136.patch) applies to
[v1.3.6](https://github.com/liveweird/grenadiers/releases/tag/v1.3.6). It registers two `cfg(test)`
modules beside the existing physical owner tests and changes no production behavior.
The fixture uses the real `Driver::tick`, runtime publication, input handling and physical
movement. Its grant tracker and admission decisions are synthetic test models, not existing
CPU planner/controller authority.

Five focused tests pass:

| Case | Evidence |
| --- | --- |
| Exact admission | Synthetic admission waits before `basis+6`, accepts that exact publication and refuses a later one. |
| First grant | Real physical movement occurs on `basis+7…basis+9`. |
| Renewal order | The `basis+9` publication is observed before servicing `basis+10`; a refreshed grant moves on that next update and matches uninterrupted position/heading. |
| Missing or invalid renewal | Stop takes effect on the first unauthorized update. |
| Early publication | After one/two moves, the real pending-report count is one/two; `current_notice()` clears it to zero. The synthetic tracker resets phase, revokes, and the next physical update is stationary. |

The early-publication case exercises the actual runtime reset, deliberately forced by the test;
it is not an observed reconnect incident. Publication cadence follows accumulated reports,
not `tick % 3`.

Independent review caught two weak assertions before acceptance: comparing two potentially
stationary outcomes did not prove renewed movement, and observing a publication tick did not
prove report accumulation reset. The final tests explicitly assert both movement and pending
report counts. Focused tests, strict server Clippy with tests, formatting, source-size and diff
checks pass on the final experimental source.

## Limits

The experiment does not prove real CPU admission, a complete conditional trajectory, any safe
Stop suffix, Teleporter/contact checks, pending-turn assistance, residual-motion carry or
production recovery. Matching position and heading for this straight movement is narrower than
matching every opaque motion field. Synthetic grants must never be used as safety certificates.

The separate Core proof, bounded renewal work and normal controller integration remain required
before activating new policy/replay identities. In particular, early revocation requires checked
Stop branches after the first and second moving updates as well as the usual third-update
checkpoint.

## Reproduce

Extract a clean `git archive v1.3.6` into a temporary directory. Apply the linked patch there,
then run:

```sh
CARGO_BUILD_JOBS=1 cargo test -p grenadiers-server --locked recovery_checkpoint_timing --lib
cargo clippy -p grenadiers-server --locked --tests -- -D warnings
```

The tested experiment was committed as `17ff756f` against application source
`efec286f0ad3046c78a2d288d41967af2b946c00`. The published v1.3.6 differs from that application
source only in documentation; the patch is checked against the published source archive.
Keeping this fixture in history leaves current production controllers and archived contracts
unchanged.
