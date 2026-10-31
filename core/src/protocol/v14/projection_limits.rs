use super::projection_facts::WorldFacts;

pub(crate) fn bombs(world: &WorldFacts) -> usize {
    if matches!(world.rules_version, 9 | 10) {
        usize::from(world.arena.width) * usize::from(world.arena.height)
    } else {
        super::MAX_BOMBS
    }
}

pub(crate) const fn report_events(rules_version: u8) -> usize {
    if matches!(rules_version, 9 | 10) {
        216
    } else {
        49
    }
}

pub(crate) const fn snapshot_events(rules_version: u8) -> usize {
    if matches!(rules_version, 9 | 10) {
        256
    } else {
        super::MAX_SNAPSHOT_EVENTS
    }
}
