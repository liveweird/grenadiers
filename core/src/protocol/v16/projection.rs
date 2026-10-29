use crate::game::{GenerationRecord, RulesConfig};
use crate::protocol::{Cell, Counter, Direction, EntityId, Position};

use super::{
    ActorView, GameState, LairView, MobView, ProjectionError, RoomPhase, RoomView, Seat,
    StormCloudView,
};

#[derive(Clone)]
pub struct ExpandedCapture {
    pub(crate) base: crate::protocol::v14::ExpandedCapture,
    pub(crate) lairs: Vec<LairView>,
    pub(crate) mobs: Vec<MobView>,
}

pub struct BoundPublication {
    pub(crate) room: RoomView,
    pub(crate) capture: ExpandedCapture,
    physical_round: u64,
}

impl BoundPublication {
    pub(crate) const fn physical_round(&self) -> u64 {
        self.physical_round
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipientProjection {
    pub state: GameState,
    pub events: super::SnapshotEvents,
}

pub fn capture_expanded(
    state: &crate::game::State,
    reports: &[crate::game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    if state.rules().version() != 8 {
        return Err(ProjectionError::InvalidRules);
    }
    let base = crate::protocol::v14::projection_capture::capture_with_profile(
        state,
        reports,
        crate::protocol::v14::projection_facts::ProjectionProfile {
            rules: *state.rules(),
            available_items: crate::protocol::v14::projection_supported_items::available_items_for(
                crate::protocol::v14::RulesVersion::MULTIPLAYER_V6,
            ),
            storm_cloud: state.rules().storm_cloud_config(),
        },
    )?;
    let tick = state.public_view().tick.0;
    let lairs = match state.lairs() {
        None => Vec::new(),
        Some(values) => values
            .into_iter()
            .enumerate()
            .map(|(id, value)| {
                Ok(LairView {
                    id: id as u8,
                    cell: cell(value.cell)?,
                    spawn_remaining_ticks: u16::try_from(
                        value.next_spawn_at.0.saturating_sub(tick),
                    )
                    .map_err(|_| ProjectionError::InvalidClock)?,
                    living_mobs: value.living_mobs,
                })
            })
            .collect::<Result<Vec<_>, ProjectionError>>()?,
    };
    let mobs = state
        .mobs()
        .into_iter()
        .map(|value| {
            Ok(MobView {
                id: EntityId::new(value.id.0),
                lair_id: value.lair,
                position: position(value.position)?,
                heading: direction(value.heading),
                frozen: value.frozen,
            })
        })
        .collect::<Result<Vec<_>, ProjectionError>>()?;
    Ok(ExpandedCapture { base, lairs, mobs })
}

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: GenerationRecord,
) -> Result<BoundPublication, ProjectionError> {
    super::room_validation::validate_room_view(room)?;
    let facts = &capture.base;
    let base = room.configuration.domain().base();
    let (width, height) = base.board().dimensions();
    let expected_generator = if base.board() == crate::room::BoardSize::FifteenByThirteen {
        crate::game::MULTIPLAYER_GENERATOR_VERSION
    } else {
        crate::game::VARIABLE_GENERATOR_VERSION
    };
    let state_bound = match room.phase {
        RoomPhase::Running => {
            state_epoch == room.round_id
                && facts.world.physical_tick == room.tick
                && facts.world.result.is_none()
        }
        RoomPhase::Result { .. } => room.last_round.is_some_and(|last| {
            state_epoch == last.round_id
                && facts.world.physical_tick == last.ended_at
                && facts.world.result == Some(last.result)
        }),
        RoomPhase::MatchResult { ended_at, .. } if facts.world.result.is_some() => {
            room.last_round.is_some_and(|last| {
                state_epoch == last.round_id
                    && facts.world.physical_tick == last.ended_at
                    && facts.world.result == Some(last.result)
                    && last.ended_at <= ended_at
            })
        }
        RoomPhase::MatchResult { ended_at, .. } => {
            state_epoch == room.round_id
                && facts.world.physical_tick == ended_at
                && facts.world.result.is_none()
        }
        _ => false,
    };
    if facts.rules != expected_rules(room)?
        || facts.world.rules_version != 8
        || facts.world.actors.len() != base.controllers().len()
        || u16::from(facts.world.arena.width) != width
        || u16::from(facts.world.arena.height) != height
        || facts.world.enabled_items != room.settings.base.enabled_items()
        || facts.world.available_items != room.available_items
        || generation.version() != expected_generator
        || !state_bound
        || !valid_lairs(room, capture)
        || crate::protocol::v14::projection_validation::validate(
            &facts.world,
            &facts.reports,
            &facts.rules,
        )
        .is_err()
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    Ok(BoundPublication {
        room: room.clone(),
        capture: ExpandedCapture {
            base: facts.clone(),
            lairs: capture.lairs.clone(),
            mobs: capture.mobs.clone(),
        },
        physical_round: state_epoch.get() as u64,
    })
}

pub fn project_recipient(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<RecipientProjection, ProjectionError> {
    let facts = &publication.capture.base;
    let projected = crate::protocol::v14::projection_recipient::project_facts(
        &facts.world,
        &facts.reports,
        recipient,
    )?;
    let actors = projected
        .state
        .actors
        .into_iter()
        .enumerate()
        .map(|(index, actor)| match actor {
            crate::protocol::v14::ActorView::Visible {
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
            } => ActorView::Visible {
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
                storm_slow_remaining_ticks: facts.world.actors[index].storm_slow_remaining_ticks,
            },
            crate::protocol::v14::ActorView::Hidden { id, seat, alive } => {
                ActorView::Hidden { id, seat, alive }
            }
        })
        .collect();
    let storm_cloud = facts.world.storm_cloud.map(|value| StormCloudView {
        position: value.position,
        direction: value.direction,
        leg_remaining_ticks: value.leg_remaining_ticks,
    });
    Ok(RecipientProjection {
        state: GameState {
            running_updates: projected.state.running_updates,
            remaining_ticks: projected.state.remaining_ticks,
            physical_tick: projected.state.physical_tick,
            arena: projected.state.arena,
            actors,
            bombs: projected.state.bombs,
            flames: projected.state.flames,
            oil_barrels: projected.state.oil_barrels,
            pickups: projected.state.pickups,
            storm_cloud,
            lairs: publication.capture.lairs.clone(),
            mobs: publication.capture.mobs.clone(),
        },
        events: projected.events,
    })
}

pub fn prepare_snapshots(
    publication: &BoundPublication,
) -> Result<Vec<(Seat, String)>, ProjectionError> {
    let count = publication.capture.base.world.actors.len();
    [Seat::One, Seat::Two, Seat::Three, Seat::Four]
        .into_iter()
        .take(count)
        .map(|recipient| {
            let projected = project_recipient(publication, recipient)?;
            let encoded = super::encode_server_message(&super::GameServerMessage::Snapshot {
                room: publication.room.clone(),
                state: projected.state,
                events: projected.events,
            })
            .map_err(|_| ProjectionError::EncodingFailed)?;
            Ok((recipient, encoded))
        })
        .collect()
}

fn expected_rules(room: &RoomView) -> Result<RulesConfig, ProjectionError> {
    let settings = &room.settings.base;
    let speed = [
        crate::game::SpeedLevel::Base,
        crate::game::SpeedLevel::One,
        crate::game::SpeedLevel::Two,
        crate::game::SpeedLevel::Three,
        crate::game::SpeedLevel::Four,
    ]
    .get(settings.starting_speed_level() as usize)
    .copied()
    .ok_or(ProjectionError::InvalidExpandedBinding)?;
    let domain = crate::room::RoomSettings::new(
        speed,
        settings.turn_assist_units(),
        settings.wins_to_finish(),
        u64::from(settings.bomb_fuse_ticks()),
    )
    .map_err(|_| ProjectionError::InvalidExpandedBinding)?
    .with_enabled_items(
        crate::game::EnabledItems::from_bits(settings.enabled_items().mask().bits())
            .ok_or(ProjectionError::InvalidItems)?,
    );
    let (cloud, lairs) = room
        .settings
        .environment
        .domain()
        .map_err(|_| ProjectionError::InvalidExpandedBinding)?;
    Ok(RulesConfig::environment_v8(domain, cloud, lairs.config()))
}

fn valid_lairs(room: &RoomView, capture: &ExpandedCapture) -> bool {
    let tuning = room.configuration.domain().lairs();
    if !tuning.enabled() {
        return capture.lairs.is_empty() && capture.mobs.is_empty();
    }
    let expected = crate::game::lair_layout_v1(room.configuration.domain().base().board());
    if capture.lairs.len() != 2 || capture.mobs.len() > crate::game::MAX_MOBS {
        return false;
    }
    if capture.lairs[0].spawn_remaining_ticks != capture.lairs[1].spawn_remaining_ticks {
        return false;
    }
    if capture
        .mobs
        .iter()
        .any(|mob| mob.frozen != capture.mobs[0].frozen)
    {
        return false;
    }
    let active_stopwatch = capture.base.world.actors.iter().any(|actor| {
        actor.view.alive
            && actor
                .view
                .effects
                .as_slice()
                .iter()
                .any(|effect| effect.kind() == super::EffectKind::Stopwatch)
    });
    if capture
        .mobs
        .first()
        .is_some_and(|mob| mob.frozen != active_stopwatch)
    {
        return false;
    }
    let (width, height) = room.configuration.domain().base().board().dimensions();
    if capture.mobs.iter().any(|mob| {
        let half = crate::game::types::HALF_FOOTPRINT as u32;
        u32::from(mob.position.x()) < half
            || u32::from(mob.position.x()) + half > u32::from(width) * 1_024
            || u32::from(mob.position.y()) < half
            || u32::from(mob.position.y()) + half > u32::from(height) * 1_024
    }) {
        return false;
    }
    let mut ids = capture
        .base
        .world
        .actors
        .iter()
        .map(|actor| actor.view.id.get())
        .chain(capture.base.world.bombs.iter().map(|bomb| bomb.id.get()))
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != capture.base.world.actors.len() + capture.base.world.bombs.len() {
        return false;
    }
    capture.lairs.iter().enumerate().all(|(index, lair)| {
        let terrain_index =
            usize::from(lair.cell.row()) * usize::from(width) + usize::from(lair.cell.column());
        lair.id as usize == index
            && lair.cell == cell(expected[index]).ok().unwrap()
            && capture.base.world.arena.terrain.get(terrain_index)
                == Some(&crate::protocol::v14::Terrain::Wall)
            && lair.spawn_remaining_ticks <= u16::from(tuning.settings().spawn_seconds()) * 60
            && lair.living_mobs <= tuning.settings().mobs_per_lair()
            && usize::from(lair.living_mobs)
                == capture
                    .mobs
                    .iter()
                    .filter(|mob| mob.lair_id as usize == index)
                    .count()
    }) && capture
        .mobs
        .iter()
        .all(|mob| mob.lair_id < 2 && ids.insert(mob.id.get()))
}

fn cell(value: crate::game::CellId) -> Result<Cell, ProjectionError> {
    Cell::new(value.column as u8, value.row as u8).map_err(|_| ProjectionError::InvalidArena)
}
fn position(value: crate::game::Position) -> Result<Position, ProjectionError> {
    Position::new(
        u16::try_from(value.x).map_err(|_| ProjectionError::InvalidActors)?,
        u16::try_from(value.y).map_err(|_| ProjectionError::InvalidActors)?,
    )
    .map_err(|_| ProjectionError::InvalidActors)
}
const fn direction(value: crate::game::Direction) -> Direction {
    match value {
        crate::game::Direction::North => Direction::North,
        crate::game::Direction::East => Direction::East,
        crate::game::Direction::South => Direction::South,
        crate::game::Direction::West => Direction::West,
    }
}
