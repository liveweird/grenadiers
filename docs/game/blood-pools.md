# Cosmetic blood pool design

This design extends the existing [death presentation](arena-3d.md#death-animation).
Delivery status and priority live only in the [backlog](../../BACKLOG.md).

## Requested behavior

Player and mob knockout sprites disappear after their short animation. A small blood pool
remains at the death position. It is purely aesthetic: it never changes collision, movement,
damage, bomb behavior, pickups, visibility rules or AI decisions.

## Proposed presentation

Use a restrained dark-red pixel-art stain, smaller than a tile and lying flat on the floor.
The stain can appear beneath the falling character and remain after the body disappears.
Keep hazards, pickups and living actors legible above it. Reduced motion needs no animation.
Add a named frame to the shared artist sheet without moving or changing existing artwork.

Keep stains until the next round, including the intervening result screen. Bound retained
stains, combining repeated deaths in the same tile rather than accumulating unlimited decals
from lair respawns. They need no server simulation or wire-contract field.

## Privacy and lifecycle

Consume the same recipient-safe death cues as the knockout animation. Never reconstruct a
hidden or stale actor position, infer a concealed player's location, or treat disconnects as
deaths. Existing guarded mob-removal inference remains the boundary for cosmetic mob deaths.

Admission/reconnect, room or round replacement, backwards timelines and suspension clear
local history. Reconnecting does not reconstruct old stains from current dead actors.
An ordinary snapshot or a change to reduced motion must not duplicate a stain.

Verify persistence after body expiry, visible actor and mob deaths, deduplication and bounds,
round/result transitions, reset privacy, floor placement and GPU resource ownership. Keep all
existing server gameplay checks unchanged.
