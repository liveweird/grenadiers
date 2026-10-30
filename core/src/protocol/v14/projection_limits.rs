use super::projection_facts::WorldFacts;

pub(crate) fn bombs(world: &WorldFacts) -> usize {
    if world.rules_version == 9 {
        usize::from(world.arena.width) * usize::from(world.arena.height)
    } else {
        super::MAX_BOMBS
    }
}

pub(crate) const fn report_events(rules_version: u8) -> usize {
    if rules_version == 9 { 216 } else { 49 }
}

pub(crate) const fn snapshot_events(rules_version: u8) -> usize {
    if rules_version == 9 {
        256
    } else {
        super::MAX_SNAPSHOT_EVENTS
    }
}
