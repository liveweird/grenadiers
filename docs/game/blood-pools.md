# Cosmetic blood pools

Blood pools extend the existing [death presentation](arena-3d.md#death-animation).
Open work and priority live only in the [backlog](../../BACKLOG.md).

## Behavior

Player and mob knockout sprites disappear after their short animation. A small blood pool
remains at the death position. It is purely aesthetic: it never changes collision, movement,
damage, bomb or spike-pit behavior, pickups, visibility rules or AI decisions.

## Presentation

A restrained dark-red pixel-art stain appears beneath the falling character and remains after
the body disappears. It is smaller than a tile and lies flat on the floor, below hazards,
pickups and living actors. Reduced motion uses the same static stain. Artists can repaint
`effect.bloodPool` in the shared sheet; every earlier frame remains unchanged.

Stains persist until the next round, including the intervening result screen. At most one
stain is retained per tile; repeated deaths replace its position rather than accumulating
decals from lair respawns. Cue deduplication is separately bounded. There is no server
simulation or wire-contract field for stains.

## Privacy and lifecycle

Pools consume the same recipient-safe death cues as the knockout animation. The renderer never
reconstructs a hidden or stale actor position, infers a concealed player's location, or treats
disconnects as deaths. Existing guarded mob-removal inference supplies cosmetic mob deaths.

Admission/reconnect, room or round replacement, backwards timelines and suspension clear
local history. Reconnecting does not reconstruct old stains from current dead actors.
An ordinary snapshot or a change to reduced motion does not duplicate a stain. A new generation
can retain a valid death latched before React renders, even when a replacement room resets its
tick. Retained envelopes cannot replay stains after suspension.

Unit and native browser checks cover persistence after body expiry, visible actor and mob
deaths, deduplication and bounds, round/result transitions, reset privacy, floor placement and
GPU resource ownership. Server gameplay is unchanged.
