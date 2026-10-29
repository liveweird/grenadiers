use crate::game::{GenerationRecord, RulesConfig};
use crate::protocol::Counter;

pub(crate) use super::room_validation::validate_room;
use super::{ActorView, GameState, ProjectionError, RoomPhase, RoomView, Seat, StormCloudView};

pub struct ExpandedCapture(pub(crate) crate::protocol::v14::ExpandedCapture);

pub struct BoundPublication {
    room: RoomView,
    capture: crate::protocol::v14::ExpandedCapture,
    physical_round: u64,
}

impl BoundPublication {
    pub(crate) const fn cpu_context(
        &self,
    ) -> (super::RoomSettings, crate::game::StormCloudConfig, u16) {
        (
            self.room.settings,
            self.room.settings.environment.domain(),
            self.room.rules_version.get() as u16,
        )
    }

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
    if state.rules().version() != 7 {
        return Err(ProjectionError::InvalidRules);
    }
    let available_items = crate::protocol::v14::projection_supported_items::available_items_for(
        crate::protocol::v14::RulesVersion::MULTIPLAYER_V6,
    );
    crate::protocol::v14::projection_capture::capture_with_profile(
        state,
        reports,
        crate::protocol::v14::projection_facts::ProjectionProfile {
            rules: *state.rules(),
            available_items,
            storm_cloud: state.rules().storm_cloud_config(),
        },
    )
    .map(ExpandedCapture)
}

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: GenerationRecord,
) -> Result<BoundPublication, ProjectionError> {
    validate_room(room)?;
    let facts = &capture.0;
    let base = room.configuration.domain().base();
    let (width, height) = base.board().dimensions();
    let expected = expected_rules(room)?;
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
    if facts.rules != expected
        || facts.world.rules_version != 7
        || facts.world.actors.len() != base.controllers().len()
        || u16::from(facts.world.arena.width) != width
        || u16::from(facts.world.arena.height) != height
        || facts.world.enabled_items != room.settings.base.enabled_items()
        || facts.world.available_items != room.available_items
        || generation.version() != expected_generator
        || !state_bound
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
        capture: facts.clone(),
        physical_round: state_epoch.get() as u64,
    })
}

pub fn project_recipient(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<RecipientProjection, ProjectionError> {
    let projected = crate::protocol::v14::projection_recipient::project_facts(
        &publication.capture.world,
        &publication.capture.reports,
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
                storm_slow_remaining_ticks: publication.capture.world.actors[index]
                    .storm_slow_remaining_ticks,
            },
            crate::protocol::v14::ActorView::Hidden { id, seat, alive } => {
                ActorView::Hidden { id, seat, alive }
            }
        })
        .collect();
    let cloud = publication
        .capture
        .world
        .storm_cloud
        .map(|value| StormCloudView {
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
            storm_cloud: cloud,
        },
        events: projected.events,
    })
}

pub fn prepare_snapshots(
    publication: &BoundPublication,
) -> Result<Vec<(Seat, String)>, ProjectionError> {
    let count = publication.capture.world.actors.len();
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
    let speed = match settings.starting_speed_level() {
        0 => crate::game::SpeedLevel::Base,
        1 => crate::game::SpeedLevel::One,
        2 => crate::game::SpeedLevel::Two,
        3 => crate::game::SpeedLevel::Three,
        4 => crate::game::SpeedLevel::Four,
        _ => return Err(ProjectionError::InvalidExpandedBinding),
    };
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
    Ok(RulesConfig::environment_v7(
        domain,
        room.settings.environment.domain(),
    ))
}
