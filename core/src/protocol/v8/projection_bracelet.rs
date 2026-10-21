use crate::protocol::Seat;

use super::projection_facts::{ProjectionError, ReportFacts, WorldFacts};
use super::{ItemId, ItemSet};

pub(super) fn validate_world(
    world: &WorldFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    let round_start = world
        .physical_tick
        .get()
        .checked_sub(u64::from(world.running_updates))
        .ok_or(ProjectionError::InvalidClock)?;
    let allowed = bracelet_allowed(world, supported_items);
    if world
        .last_successful_shove_ticks
        .iter()
        .flatten()
        .any(|tick| !allowed || tick.get() <= round_start || *tick > world.physical_tick)
    {
        return Err(ProjectionError::InvalidActors);
    }
    Ok(())
}

pub(super) fn validate_report(
    world: &WorldFacts,
    report: &ReportFacts,
) -> Result<(), ProjectionError> {
    validate_report_for_items(world, report, world.available_items)
}

fn validate_report_for_items(
    world: &WorldFacts,
    report: &ReportFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    let Some(forced) = report.forced_movement else {
        return Ok(());
    };
    if !bracelet_allowed(world, supported_items) {
        return Err(ProjectionError::InvalidReports);
    }
    let round_start = world
        .physical_tick
        .get()
        .checked_sub(u64::from(world.running_updates))
        .ok_or(ProjectionError::InvalidClock)?;
    if report.tick.get() <= round_start {
        return Err(ProjectionError::InvalidReports);
    }

    let horizontal = i32::from(forced.end.x()) - i32::from(forced.start.x());
    let vertical = i32::from(forced.end.y()) - i32::from(forced.start.y());
    let distance = horizontal.unsigned_abs().max(vertical.unsigned_abs());
    if distance == 0 || distance > 1_024 || (horizontal != 0) == (vertical != 0) {
        return Err(ProjectionError::InvalidReports);
    }

    let (victim_index, pusher_index) = match forced.victim {
        Seat::One => (0, 1),
        Seat::Two => (1, 0),
    };
    if world.last_successful_shove_ticks[pusher_index].is_none_or(|marker| marker < report.tick) {
        return Err(ProjectionError::InvalidReports);
    }
    if report.tick == world.physical_tick
        && world.last_teleport_tick != Some(report.tick)
        && world.actors[victim_index].position != forced.end
    {
        return Err(ProjectionError::InvalidReports);
    }
    Ok(())
}

fn bracelet_allowed(world: &WorldFacts, supported_items: ItemSet) -> bool {
    supported_items.contains(ItemId::Bracelet)
        && world.available_items.contains(ItemId::Bracelet)
        && world.enabled_items.contains(ItemId::Bracelet)
}

#[cfg(test)]
pub(crate) fn validate_report_for_test(
    world: &WorldFacts,
    report: &ReportFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    validate_report_for_items(world, report, supported_items)
}
