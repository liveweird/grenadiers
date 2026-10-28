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
    super::projection_bomb_validation::validate(world)?;
    super::projection_item_validation::validate(world, rules)?;
    super::projection_oil::validate(world)?;
    super::projection_history::validate(world, reports, rules)
}
