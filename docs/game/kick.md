# Kick

Kick is configurable permanent equipment enabled in new Expanded-v5 rooms. Walking voluntarily into a stationary
Timed or Remote bomb arms motion in the effective movement direction. The first one-cell hop occurs
on the next running update and later hops occur every ten updates until a hard wall, breakable block,
any bomb, or a living actor blocks the swept cells. Contact mines cannot be kicked.

Motion changes only the bomb's public cell. Its entity ID, owner, range, blast kind, placement tick
and captured fuse remain unchanged. A moved bomb remains fully solid in its current cell between
hops. Movement creates no placement, detonation or audio event. Timed bombs still detonate at their
captured deadline; Remote bombs retain their owner action trigger.

Direction, countdown and persistent `hasBeenKicked` provenance are private trusted state used for
capture, resume, replay and archive rejection. They never appear in gameplay-v11 wire state or
events. Public clients render the latest authoritative cell on the retained bomb drawing. Invisible
Bombs filtering and Invisibility Cloak attribution rules remain unchanged.

Version 0.3.44 added Kick as the first item in its nineteen-item catalog. Its capability bit
is `1`; `ALL` is `3,145,721`. Generator v2 appends weight 15 after Wall Pass, making the complete
pool 395 and Kick interval `[380,395)`. All 524,288 catalog subsets are validated. Archived gameplay
v9 and earlier reject Kick catalog entries, equipment, pickups, collection history, and trusted bomb
motion or stopped history. At the 2026-10-24 source-gates checkpoint, candidate
`cb70cba891ed972380f30f52133622ce36fd9ee8` passes all 524,288 masks plus fresh Rust, web, E2E and
security gates. Fresh image `sha256:8ff3005861127f8ac57cc32f7e6c72ce5fa996fe628ad48426dbb8cf133e1af9`
passes all 56 Compose plus 56 Kubernetes cases with zero retries or carried results. Independent
audit, result/binding review, durable seal
`94c60dac44c639f7e9a214061e9daf9475628126d15526346ed7be2ce4c9a947` and post-seal review are
clear. Published source `f32fd86d8f0c7c4bebba8fa60d1c66c65d1a174a`, tree
`a0f2addb9204c40b4b30be2830fde4df368f019c`, is available as
[v0.3.44](https://github.com/liveweird/grenadiers/releases/tag/v0.3.44), and the accepted image is
promoted locally.

Captures refreshed on 2026-10-25 for the Napoleonic arena art show the stopped kicked bomb in
[English desktop](../screenshots/kick-desktop.png) and
[Polish 320px reduced motion](../screenshots/kick-mobile-pl.png). Their SHA-256 values are
`4a50e5883b603954bd35ca82b26032482c93d58a399793917e0bb64e0c9db823` and
`4819920b713180dbedce93e32d60a600e2fb4cd6caf851c7865ffc5e5e22e7dd`.
