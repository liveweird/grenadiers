# Gameplay v9: voluntary-action freeze foundation

The coordinated server/browser contract is `api/gameplay-v9.schema.json`, Rust
`core::protocol::v9`, and generated `web/src/game/v9/schema.ts`. Version 8 is archived and rejected
before credential lookup, so an old client cannot claim or revoke a credential for a v9 connection.
Deploy the paired server and browser assets together. After deployment, reload clients so every
connection uses the server's current protocol version; do not mix v8 browser assets with a v9
server.

Gameplay v9 retains the same Expanded-v5 rules identity, generated-v2 maps, journal format 2 and
exact fourteen-item availability catalog as v8. It does not enable Stopwatch or change simulation
behavior. All 16,384 existing enabled-item masks and their generator paths remain unchanged.
Archived v8 settings and projection rules are pinned to that same Cloak-era catalog; the current
rules-v5 domain and journal remain capable of adding later operations without relabeling old wire
messages.

Visible actors add one required boolean, `voluntaryActionsFrozen`. Version 0.3.39 always publishes
`false`; `true` is rejected by the server-side foundation policy and browser semantic validation.
Hidden actors remain exactly `{type,id,seat,alive}`, so recipient filtering does not reveal another
actor's position, inventory, effects, timers or freeze cause. Dead visible actors must also remain
unfrozen. Paused Timed bombs remain unavailable. Remote and Contact bombs are untimed and continue
to use their existing enabled-item guards.

The existing capture and binding path still validates the complete world and report window before
recipient filtering. V9 converts the validated visible actor into its version-specific shape, then
prepares both recipient encodings before publication state can advance. Admission likewise prepares
its recipient-specific, event-free body before ownership or replacement is committed. The 4 KiB
client-frame, 64 KiB server-frame, 16-bomb and 96-event bounds are unchanged.

Stopwatch collection, voluntary-action freezing, timed-fuse pausing and presentation cues are
reserved for a later activation. This foundation intentionally adds no Stopwatch setting, pickup,
effect, paused trigger, renderer cue or control behavior.

At the 21 October 2026 18:44:23 UTC native acceptance checkpoint, frozen source `272b950` and
image `9a4dc108` passed all 51 Compose and 51 Kubernetes cases with zero retries or carried
passes. The matrix covered the required visible false field, the exact four-field hidden shape,
current-version connection handling, and the unchanged fourteen-item catalog. Independent raw
audit is clear and the evidence is durably sealed. At that checkpoint, v0.3.39 had not been
normalized, pushed, merged, published or promoted; candidate, post-merge and tag CI remained
pending. See [repository verification](../verification.md) for the full evidence and
[release notes](https://github.com/liveweird/grenadiers/releases) for current release status.
