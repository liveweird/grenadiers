# Piercing Bomb

Piercing Bomb is an opt-in Expanded v5 pickup. It occupies the one-charge weapon slot, replacing
another stored weapon. A successful manual bomb placement consumes the charge and captures a
Piercing blast. Failed placements retain it, and collecting a replacement on the same tick follows
placement, so the old charge is used before the new one is stored.

## Blast behavior

A Piercing blast spends one range unit for every crossed cell. It burns and destroys a breakable
block, then continues if range remains. Arena walls and boundaries stop it without fire. Every bomb
also stops the incoming ray and chains normally; a chained bomb uses its own captured blast and
range. The complete explosion plan uses the immutable pre-explosion world, so another Piercing ray
marking a block for destruction does not let an ordinary ray pass through that block in the same plan.

The blast, range, fuse, owner, and placement tick are captured on placement. Later inventory
replacement, pickup expiry, death, or reconnect cannot change an existing bomb. Automatic Diarrhea
bombs are always ordinary and never inspect or consume the stored charge.

## Compatibility

Piercing Bomb uses catalog bit 6 and generator weight 15 after Slow foot. Its initial catalog
supported all 1024 subsets and preserved every earlier 512-mask path; later items join the
[current catalog](settings.md). Rules version 5, generator version 2 and journal format 2 remain
unchanged. Current [gameplay-v11](protocol-v11.md) requires paired server/web deployment and a
browser reload.

## Acceptance and presentation

Fresh same-image native acceptance passes all 47 Compose and 47 Kubernetes browser cases with
zero retries. The paired-browser scenario collects the pickup with ordinary key input, places an
armed bomb, observes its blast continue through the breakable blocks at `(4,1)` and `(5,1)`, and
proves consumption by placing a fresh Standard bomb afterward on both peers.

The accepted Compose run produced the [English desktop](../screenshots/piercing-bomb-desktop.png)
and [Polish 320px](../screenshots/piercing-bomb-polish-320.png) captures. Neither reviewed capture
has overlapping controls or game content. In the compact four-column English HUD, “Piercing bomb”
wraps inside the word; this is a nonblocking cosmetic preview limitation rather than a claim of
perfect compact layout. Kubernetes captures were retained by the run but were not separately
viewed. Independent evidence review, candidate CI and post-merge CI pass.
[v0.3.34](https://github.com/liveweird/grenadiers/releases/tag/v0.3.34) is published as a source
prerelease, and its exact accepted image serves localhost:8085 at the deployment checkpoint; see
[verification](../verification.md).
