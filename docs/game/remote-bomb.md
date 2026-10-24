# Remote Bomb

Remote Bomb is an opt-in Expanded v5 pickup. It occupies the one-charge weapon slot and replaces
another stored weapon. A successful manual placement consumes the charge and creates one armed
Standard bomb for that owner. Failed placements retain it. Placement precedes pickup collection,
so an old charge is used before a new weapon at the same cell is stored.

## Trigger behavior

An armed Remote Bomb has no fuse. The owner's next retained manual bomb action, classified after
movement and existing-fire damage, detonates that exact bomb instead of placing another. The
action allocates no entity, consumes no stored replacement weapon, and ignores placement capacity.
A dead actor cannot trigger it. Fire and bomb chains can still detonate it, while death,
disconnect and reconnect never invent a deadline or detonation.

Each owner may have at most one live Remote Bomb. An event window may contain an older Remote
detonation and a later placement for the same owner. The bomb captures its owner, range, Standard
blast, cell and placement tick once. Automatic Diarrhea actions always place ordinary Timed bombs
and neither consume a Remote charge nor trigger an armed bomb.

The HUD shows the stored weapon and armed Remote state independently. A placed Remote has an
antenna and no burning-fuse spark; the bomb-key hint changes to the localized detonation action
while the owner has an armed bomb.

## Native acceptance

Fresh same-image acceptance passes all 49 Compose and 49 Kubernetes cases with zero retries.
The dedicated paired-browser scenario proves the planted Remote remains live for 183 ticks,
the next bomb action activates that exact ID without placing another bomb, and the later manual
placement is a fresh Standard bomb. The reviewed Compose [English desktop](../screenshots/remote-bomb-desktop.png)
and [Polish 320px](../screenshots/remote-bomb-polish-320.png) captures show the planted Remote at
(3,1), consumed charge, owner availability zero, both players alive and no flames. The English
Gear label fits normally; the Polish view uses vertical scrolling and shows the unfocused-arena
prompt. No unintended overlap was observed. Kubernetes screenshots were not visually reviewed.
The raw evidence audit is clear and its final manifest is sealed. Native evidence and documentation review are clear; publication requires exact-head candidate and post-merge CI.

## Compatibility

Remote Bomb uses catalog bit 5 and generator weight 15 after Power Bomb. Its initial release
supported all 4096 subsets and preserved every earlier 2048-mask generator path; those paths
remain stable as later items join the [current catalog](settings.md). Rules version 5, generator
version 2 and journal format 2 remain unchanged. Current deployments use [gameplay-v11](protocol-v11.md)
and require the paired server/web assets plus a browser reload.
