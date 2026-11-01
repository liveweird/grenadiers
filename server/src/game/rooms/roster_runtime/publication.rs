use grenadiers_core::{
    game::{DrawReason, GenerationRecord, SeatId, State, StepReport},
    protocol::{Counter, Tick as WireTick, v14},
    room::{
        CountdownKind, InterruptReason, MatchPhase, RoomSettings, RosterMatchLifecycle,
        RosterParticipant, SeatControl,
        series::{CompletedRound, MatchOutcome, MatchResult},
    },
};

use super::{
    error::RosterRuntimeError,
    types::{RosterIdentity, RosterNotice, RosterPublication, RosterSeatFrames, wire_seat},
};

pub(super) fn room_view(
    identity: &RosterIdentity,
    lifecycle: &RosterMatchLifecycle,
    revision: u32,
    settings_revision: u32,
) -> Result<v14::RoomView, RosterRuntimeError> {
    let room = v14::RoomView {
        settings: settings(lifecycle.settings())?,
        settings_revision: counter(settings_revision)?,
        settings_editor: lifecycle.settings_editor().map(wire_seat),
        available_items: available_items()?,
        configuration: v14::RoomConfiguration::from_domain(identity.configuration)
            .map_err(|_| RosterRuntimeError::UnsupportedConfiguration)?,
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: counter64(lifecycle.round_attempt())?,
        revision: counter(revision)?,
        tick: tick(lifecycle.tick())?,
        rules_version: v14::RulesVersion::MULTIPLAYER_V6,
        map_id: v14::MapId::GeneratedV2,
        round_number: v14::RoundCount::new(lifecycle.actual_round_number())
            .map_err(|_| RosterRuntimeError::CapacityReached)?,
        scores: scores(lifecycle)?,
        last_round: lifecycle.latest_completed().map(completed).transpose()?,
        seats: seats(lifecycle)?,
        phase: phase(lifecycle.phase())?,
    };
    v14::validate_room_view(&room).map_err(|_| RosterRuntimeError::Publication)?;
    Ok(room)
}

pub(super) fn bound(
    room: &v14::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<v14::BoundPublication, RosterRuntimeError> {
    let capture =
        v14::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    v14::bind_expanded(room, counter64(epoch)?, &capture, Some(generation))
        .map_err(|_| RosterRuntimeError::Publication)
}

pub(super) fn snapshot(
    room: v14::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let publication = bound(&room, epoch, state, reports, generation)?;
    let encoded =
        v14::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::Legacy(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v14::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::Legacy(room, None),
        frames: None,
    }
}

pub(super) fn counter(value: u32) -> Result<Counter, RosterRuntimeError> {
    Counter::new(value).map_err(|_| RosterRuntimeError::CapacityReached)
}

pub(super) fn counter64(value: u64) -> Result<Counter, RosterRuntimeError> {
    u32::try_from(value)
        .ok()
        .and_then(|value| Counter::new(value).ok())
        .ok_or(RosterRuntimeError::CapacityReached)
}

pub(super) fn tick(value: grenadiers_core::game::Tick) -> Result<WireTick, RosterRuntimeError> {
    WireTick::new(value.0).map_err(|_| RosterRuntimeError::CapacityReached)
}

pub(super) fn settings(value: RoomSettings) -> Result<v14::GameSettings, RosterRuntimeError> {
    v14::GameSettings::new(
        value.starting_speed() as u8,
        value.turn_assist_units(),
        value.wins_to_finish(),
        u16::try_from(value.bomb_fuse_ticks()).map_err(|_| RosterRuntimeError::Publication)?,
        v14::ItemSet::from_mask(
            v14::ItemMask::new(value.enabled_items().bits())
                .map_err(|_| RosterRuntimeError::Publication)?,
        ),
    )
    .map_err(|_| RosterRuntimeError::Publication)
}

pub(super) fn available_items() -> Result<v14::ItemSet, RosterRuntimeError> {
    use v14::ItemId::*;
    v14::ItemSet::new([
        Kick,
        Punch,
        Bracelet,
        WallPass,
        Heart,
        RemoteBomb,
        PiercingBomb,
        PowerBomb,
        Landmine,
        Shield,
        Stopwatch,
        InvisibilityCloak,
        InvisibleBombs,
        GoldenFlame,
        ShortFuse,
        Diarrhea,
        SlowFoot,
        ShortFlame,
        LongFuse,
        ReverseControls,
        OilBarrel,
        Teleporter,
    ])
    .map_err(|_| RosterRuntimeError::Publication)
}

pub(super) fn scores(lifecycle: &RosterMatchLifecycle) -> Result<v14::Scores, RosterRuntimeError> {
    lifecycle
        .scores()
        .iter()
        .enumerate()
        .map(|(index, &wins)| {
            Ok(v14::Score {
                seat: wire_seat(SeatId::ALL[index]),
                wins: v14::Wins::new(wins).map_err(|_| RosterRuntimeError::Publication)?,
            })
        })
        .collect()
}

pub(super) fn seats(
    lifecycle: &RosterMatchLifecycle,
) -> Result<Vec<v14::SeatStatus>, RosterRuntimeError> {
    lifecycle
        .participants()
        .iter()
        .enumerate()
        .map(|(index, &participant)| {
            let seat = SeatId::ALL[index];
            let control = lifecycle.configuration().controller(seat).unwrap();
            let controller = match control {
                SeatControl::Human => v14::Controller::Human,
                SeatControl::Cpu => v14::Controller::Cpu {
                    difficulty: match lifecycle.configuration().cpu_difficulty().unwrap() {
                        grenadiers_core::room::CpuDifficulty::Easy => v14::Difficulty::Easy,
                        grenadiers_core::room::CpuDifficulty::Standard => v14::Difficulty::Standard,
                        grenadiers_core::room::CpuDifficulty::Hard => {
                            return Err(RosterRuntimeError::UnsupportedConfiguration);
                        }
                    },
                },
            };
            let connection = match participant {
                RosterParticipant::NeverAdmitted => v14::ConnectionStatus::Vacant,
                RosterParticipant::Connected => v14::ConnectionStatus::Connected,
                RosterParticipant::Reserved { expires_at } => v14::ConnectionStatus::Disconnected {
                    expires_at: tick(expires_at)?,
                },
                RosterParticipant::Departed => v14::ConnectionStatus::Expired,
                RosterParticipant::Cpu => v14::ConnectionStatus::ServerControlled,
            };
            Ok(v14::SeatStatus {
                seat: wire_seat(seat),
                occupied: !matches!(participant, RosterParticipant::NeverAdmitted),
                ready: lifecycle.ready(seat).map_err(super::error::lifecycle)?,
                rematch_ready: lifecycle
                    .rematch_vote(seat)
                    .map_err(super::error::lifecycle)?,
                controller,
                connection,
            })
        })
        .collect()
}

pub(super) fn completed(value: CompletedRound) -> Result<v14::CompletedRound, RosterRuntimeError> {
    Ok(v14::CompletedRound {
        round_id: counter64(value.epoch())?,
        round_number: counter(value.assignment().round_number().get())?,
        ended_at: tick(value.ended_at())?,
        result: round_result(value.result()),
    })
}

fn round_result(value: grenadiers_core::game::RoundResult) -> v14::RoundResult {
    match value {
        grenadiers_core::game::RoundResult::Winner(seat) => v14::RoundResult::Winner {
            seat: wire_seat(seat),
        },
        grenadiers_core::game::RoundResult::Draw(reason) => v14::RoundResult::Draw {
            reason: match reason {
                DrawReason::MutualElimination => v14::DrawReason::MutualElimination,
                DrawReason::Timeout => v14::DrawReason::Timeout,
            },
        },
    }
}

pub(super) fn phase(value: MatchPhase) -> Result<v14::RoomPhase, RosterRuntimeError> {
    Ok(match value {
        MatchPhase::Waiting => v14::RoomPhase::Waiting,
        MatchPhase::Countdown { kind, ends_at } => v14::RoomPhase::Countdown {
            kind: match kind {
                CountdownKind::Initial => v14::CountdownKind::Initial,
                CountdownKind::InterRound => v14::CountdownKind::InterRound,
            },
            ends_at: tick(ends_at)?,
        },
        MatchPhase::Running => v14::RoomPhase::Running,
        MatchPhase::Result { next_countdown_at } => v14::RoomPhase::Result {
            hold_until: tick(next_countdown_at)?,
        },
        MatchPhase::MatchResult {
            outcome,
            rematch_at,
        } => v14::RoomPhase::MatchResult {
            ended_at: tick(outcome.ended_at())?,
            result: match_result(outcome),
            rematch_at: tick(rematch_at)?,
        },
        MatchPhase::Interrupted { ended_at, reason } => v14::RoomPhase::Interrupted {
            ended_at: tick(ended_at)?,
            reason: interrupt(reason),
        },
    })
}

fn match_result(value: MatchOutcome) -> v14::MatchResult {
    match value.result() {
        MatchResult::ScoreWinner(seat) => v14::MatchResult::Winner {
            seat: wire_seat(seat),
            reason: v14::MatchWinnerReason::RoundWins,
        },
        MatchResult::ForfeitWinner(seat) => v14::MatchResult::Winner {
            seat: wire_seat(seat),
            reason: v14::MatchWinnerReason::Forfeit,
        },
        MatchResult::Abandoned => v14::MatchResult::Abandoned,
        MatchResult::Interrupted(reason) => v14::MatchResult::Interrupted {
            reason: interrupt(reason),
        },
    }
}

const fn interrupt(value: InterruptReason) -> v14::MatchInterruptReason {
    match value {
        InterruptReason::ParticipantLeft | InterruptReason::ConnectionLost => {
            v14::MatchInterruptReason::ConnectionLost
        }
        InterruptReason::ServerOverloaded => v14::MatchInterruptReason::ServerOverloaded,
        InterruptReason::ServerStopped => v14::MatchInterruptReason::ServerStopped,
        InterruptReason::InternalError => v14::MatchInterruptReason::InternalError,
    }
}
