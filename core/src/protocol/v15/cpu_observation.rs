use crate::cpu::{Observation, ObservedEnvironment, WorldId};
use crate::{game, protocol};

use super::{ActorView, BoundPublication, GameState, ProjectionError, Seat};

pub fn observe_cpu_recipient(
    publication: &BoundPublication,
    recipient: Seat,
    world: WorldId,
) -> Result<Observation, ProjectionError> {
    if world.physical_round != publication.physical_round() {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let projected = super::project_recipient(publication, recipient)?;
    let (room_settings, environment_config, rules_version) = publication.cpu_context();
    let settings = crate::protocol::v11::expanded_settings(room_settings.base)
        .map_err(|_| ProjectionError::InvalidRules)?;
    let rules = game::RulesConfig::environment_v7(settings, environment_config);
    let timers = projected
        .state
        .actors
        .iter()
        .map(|actor| match actor {
            ActorView::Visible {
                storm_slow_remaining_ticks,
                ..
            } => Some(*storm_slow_remaining_ticks),
            ActorView::Hidden { .. } => None,
        })
        .collect();
    let cloud = projected.state.storm_cloud.map(domain_cloud);
    let parts = crate::protocol::v14::observation_parts(
        legacy_state(projected.state),
        recipient,
        world,
        rules_version,
        rules,
        settings,
    )?;
    Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(environment_config, cloud, timers),
    )
    .map_err(crate::protocol::v14::observation_error)
}

fn legacy_state(state: GameState) -> crate::protocol::v14::GameState {
    crate::protocol::v14::GameState {
        running_updates: state.running_updates,
        remaining_ticks: state.remaining_ticks,
        physical_tick: state.physical_tick,
        arena: state.arena,
        actors: state.actors.into_iter().map(legacy_actor).collect(),
        bombs: state.bombs,
        flames: state.flames,
        oil_barrels: state.oil_barrels,
        pickups: state.pickups,
    }
}

fn legacy_actor(actor: ActorView) -> crate::protocol::v14::ActorView {
    match actor {
        ActorView::Visible {
            id,
            seat,
            position,
            heading,
            speed_level,
            alive,
            bomb_capacity,
            bomb_range,
            live_bomb_count,
            inventory,
            effects,
            damage_grace_remaining_ticks,
            voluntary_actions_frozen,
            last_teleport_tick,
            ..
        } => crate::protocol::v14::ActorView::Visible {
            id,
            seat,
            position,
            heading,
            speed_level,
            alive,
            bomb_capacity,
            bomb_range,
            live_bomb_count,
            inventory,
            effects,
            damage_grace_remaining_ticks,
            voluntary_actions_frozen,
            last_teleport_tick,
        },
        ActorView::Hidden { id, seat, alive } => {
            crate::protocol::v14::ActorView::Hidden { id, seat, alive }
        }
    }
}

fn domain_cloud(cloud: super::StormCloudView) -> game::StormCloudView {
    game::StormCloudView {
        position: game::Position {
            x: cloud.position.x().into(),
            y: cloud.position.y().into(),
        },
        direction: match cloud.direction {
            protocol::Direction::North => game::Direction::North,
            protocol::Direction::East => game::Direction::East,
            protocol::Direction::South => game::Direction::South,
            protocol::Direction::West => game::Direction::West,
        },
        leg_remaining_ticks: cloud.leg_remaining_ticks,
    }
}
