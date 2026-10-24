use std::collections::BTreeSet;

use super::{RulesConfig, World};
use crate::game::{BlastKind, BombTrigger, EntityId};

pub(super) fn unsupported_trigger(rules: RulesConfig, world: &World) -> Option<EntityId> {
    world.bombs.values().find_map(|bomb| match bomb.trigger() {
        BombTrigger::Timed { .. } => None,
        BombTrigger::Remote if rules.enabled_items().contains_remote_bomb() => None,
        BombTrigger::Contact if rules.enabled_items().contains_landmine() => None,
        BombTrigger::Remote | BombTrigger::Contact => Some(bomb.id()),
    })
}

pub(super) fn unsupported_blast(rules: RulesConfig, world: &World) -> Option<EntityId> {
    world.bombs.values().find_map(|bomb| {
        let supported = (!matches!(bomb.trigger(), BombTrigger::Remote | BombTrigger::Contact)
            || bomb.blast() == BlastKind::Standard)
            && match bomb.blast() {
                BlastKind::Standard => true,
                BlastKind::Piercing => rules.enabled_items().contains_piercing_bomb(),
                BlastKind::Power => rules.enabled_items().contains_power_bomb(),
            };
        (!supported).then_some(bomb.id())
    })
}

pub(super) fn duplicate_remote_owner(world: &World) -> Option<EntityId> {
    let mut owners = BTreeSet::new();
    world
        .bombs
        .values()
        .filter(|bomb| matches!(bomb.trigger(), BombTrigger::Remote))
        .find_map(|bomb| (!owners.insert(bomb.owner())).then_some(bomb.owner()))
}

pub(super) fn unsupported_kick_history(rules: RulesConfig, world: &World) -> Option<EntityId> {
    world.bombs.values().find_map(|bomb| {
        let invalid = (bomb.has_been_kicked() && !rules.enabled_items().contains_kick())
            || (bomb.motion().is_some() && !bomb.has_been_kicked())
            || bomb.motion().is_some_and(|motion| motion.countdown > 9)
            || (matches!(bomb.trigger(), BombTrigger::Contact)
                && (bomb.motion().is_some() || bomb.has_been_kicked()));
        invalid.then_some(bomb.id())
    })
}

pub(super) fn unsupported_punch_history(rules: RulesConfig, world: &World) -> Option<EntityId> {
    world.bombs.values().find_map(|bomb| {
        let invalid = bomb.has_been_punched()
            && (!rules.enabled_items().contains_punch()
                || matches!(bomb.trigger(), BombTrigger::Contact));
        invalid.then_some(bomb.id())
    })
}
