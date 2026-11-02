use crate::cpu::{
    Observation, ObservedEnvironment, ObservedRespawnLairs, ObservedSpikePits, WorldId,
};
use crate::{game, protocol};

use super::{ActorView, BoundPublication, GameState, ProjectionError, Seat};

pub fn observe_cpu_recipient(
    publication: &BoundPublication,
    recipient: Seat,
    world: WorldId,
) -> Result<Observation, ProjectionError> {
    observe_cpu_recipient_with_punch_activation(
        publication,
        recipient,
        world,
        game::PunchActivationPolicy::ManualV1,
    )
}

pub(crate) fn observe_cpu_recipient_with_punch_activation(
    publication: &BoundPublication,
    recipient: Seat,
    world: WorldId,
    punch_activation: game::PunchActivationPolicy,
) -> Result<Observation, ProjectionError> {
    if world.physical_round != publication.physical_round() {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let projected = super::project_recipient(publication, recipient)?;
    let room_settings = publication.room.settings;
    let (environment_config, tuning, spike_tuning) = room_settings
        .environment
        .domain()
        .map_err(|_| ProjectionError::InvalidRules)?;
    let settings = crate::protocol::v11::expanded_settings(room_settings.base)
        .map_err(|_| ProjectionError::InvalidRules)?;
    let rules = game::RulesConfig::environment_v10(
        settings,
        environment_config,
        tuning.config(),
        spike_tuning.config(),
    )
    .with_punch_activation_policy(punch_activation);
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
    let lairs = projected
        .state
        .lairs
        .iter()
        .map(|lair| {
            Ok(game::LairStatusView {
                cell: game::CellId::new(lair.cell.column().into(), lair.cell.row().into()),
                next_spawn_at: lair
                    .spawn_remaining_ticks
                    .map(|remaining| {
                        projected
                            .state
                            .physical_tick
                            .get()
                            .checked_add(u64::from(remaining))
                            .map(game::Tick)
                            .ok_or(ProjectionError::InvalidClock)
                    })
                    .transpose()?,
                living_mobs: lair.living_mobs,
            })
        })
        .collect::<Result<Vec<_>, ProjectionError>>()?;
    let mobs = projected
        .state
        .mobs
        .iter()
        .map(|mob| game::MobView {
            id: game::EntityId(mob.id.get()),
            lair: mob.lair_id,
            position: game::Position {
                x: i32::from(mob.position.x()),
                y: i32::from(mob.position.y()),
            },
            heading: domain_direction(mob.heading),
            frozen: mob.frozen,
        })
        .collect();
    let spike_pits = projected
        .state
        .spike_pits
        .iter()
        .map(|pit| game::SpikePitView {
            cell: game::CellId::new(pit.cell.column().into(), pit.cell.row().into()),
            phase: match pit.phase {
                super::SpikePitPhase::Idle => game::SpikePitPhase::Idle,
                super::SpikePitPhase::Warning => game::SpikePitPhase::Warning,
                super::SpikePitPhase::Raised => game::SpikePitPhase::Raised,
            },
            remaining_ticks: pit.remaining_ticks.into(),
        })
        .collect();
    let parts = crate::protocol::v14::observation_parts(
        legacy_state(projected.state),
        recipient,
        world,
        u16::from(publication.room.rules_version.get()),
        rules,
        settings,
    )?;
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(environment_config, cloud, timers),
        tuning
            .enabled()
            .then(|| ObservedRespawnLairs::new(tuning.config(), lairs, mobs)),
        spike_tuning
            .enabled()
            .then(|| ObservedSpikePits::new(spike_tuning.config(), spike_pits)),
    )
    .map_err(crate::protocol::v14::observation_error)
}

fn domain_direction(value: protocol::Direction) -> game::Direction {
    match value {
        protocol::Direction::North => game::Direction::North,
        protocol::Direction::East => game::Direction::East,
        protocol::Direction::South => game::Direction::South,
        protocol::Direction::West => game::Direction::West,
    }
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
        // Rules-v14 observation conversion intentionally receives only the
        // neutral world fields; lairs are supplied through `ObservedRespawnLairs`.
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
