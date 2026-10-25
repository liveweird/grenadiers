# Long Fuse

Long Fuse is a configurable timed curse introduced in v0.3.30 and enabled in new rooms. Its shared lobby switch controls whether generated arenas include it.

Collection activates or refreshes Long Fuse for exactly 600 physical simulation ticks. While active, a newly placed timed bomb captures twice the configured base fuse. The captured detonation deadline remains unchanged if the effect expires, the owner dies, or the round ends.

Short Fuse and Long Fuse retain independent timers. When both are active, new bombs use the configured base fuse. If one expires first, later bombs use the remaining effect. Expiry runs before placement and collection runs after placement in a simulation step, matching other timed pickups.

The curse changes a timed bomb's captured fuse independently of its blast shape; untimed triggers
have no fuse to modify. The v0.3.30 increment exposes only ordinary timed bombs. Classic and
explicit empty expanded settings remain supported.

Screenshots: [desktop pickup](../screenshots/long-fuse-desktop.png) and
[Polish 320px active curse](../screenshots/long-fuse-polish-320.png).
See [verification](../verification.md) for acceptance and publication status.
