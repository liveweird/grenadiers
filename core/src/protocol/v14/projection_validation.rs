use crate::game::RulesConfig;

use super::ProjectionError;
use super::projection_facts::{ReportFacts, WorldFacts};

pub(crate) fn validate(
    world: &WorldFacts,
    reports: &[ReportFacts],
    rules: &RulesConfig,
) -> Result<(), ProjectionError> {
    if rules.version() < 6 && world.actors.len() != 2 {
        return Err(ProjectionError::InvalidActors);
    }
    super::projection_world_validation::validate(world)?;
    super::projection_actor_validation::validate(world)?;
    validate_environment(world, rules)?;
    super::projection_bomb_validation::validate(world)?;
    super::projection_item_validation::validate(world, rules)?;
    super::projection_oil::validate(world)?;
    super::projection_history::validate(world, reports, rules)
}

fn validate_environment(world: &WorldFacts, rules: &RulesConfig) -> Result<(), ProjectionError> {
    use crate::game::StormCloudConfig;
    if world.actors.iter().any(|actor| {
        u64::from(actor.storm_slow_remaining_ticks) > crate::game::STORM_SLOW_TICKS
            || (!actor.view.alive && actor.storm_slow_remaining_ticks != 0)
    }) {
        return Err(ProjectionError::InvalidEffects);
    }
    match (rules.storm_cloud_config(), world.storm_cloud) {
        (StormCloudConfig::Disabled, None)
            if world
                .actors
                .iter()
                .all(|actor| actor.storm_slow_remaining_ticks == 0) =>
        {
            Ok(())
        }
        (StormCloudConfig::Single, Some(cloud)) if valid_cloud(world, cloud) => Ok(()),
        _ => Err(ProjectionError::InvalidRules),
    }
}

fn valid_cloud(world: &WorldFacts, cloud: super::projection_facts::StormCloudFacts) -> bool {
    use crate::protocol::Direction;
    if cloud.leg_remaining_ticks > crate::game::STORM_CLOUD_LEG_TICKS {
        return false;
    }
    let x = i32::from(cloud.position.x());
    let y = i32::from(cloud.position.y());
    let min = 1_536;
    let max_x = (i32::from(world.arena.width) - 2) * 1_024 + 512;
    let max_y = (i32::from(world.arena.height) - 2) * 1_024 + 512;
    if x < min || x > max_x || y < min || y > max_y {
        return false;
    }
    let progress = i32::from(crate::game::STORM_CLOUD_LEG_TICKS - cloud.leg_remaining_ticks)
        * crate::game::STORM_CLOUD_SPEED_PER_TICK;
    let centered = |value: i32| value.rem_euclid(1_024) == 512;
    match cloud.direction {
        Direction::North => centered(x) && (y + progress).rem_euclid(1_024) == 512,
        Direction::East => centered(y) && (x - progress).rem_euclid(1_024) == 512,
        Direction::South => centered(x) && (y - progress).rem_euclid(1_024) == 512,
        Direction::West => centered(y) && (x + progress).rem_euclid(1_024) == 512,
    }
}
