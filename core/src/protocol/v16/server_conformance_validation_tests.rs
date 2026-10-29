use crate::game::{EnabledItems, PlayerCount, SeatId, StormCloudConfig};
use crate::protocol::{Counter, Tick};
use crate::room::BoardSize as DomainBoard;

use super::*;

fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

#[test]
fn checked_room_validation_rejects_false_cpu_readiness_before_start() {
    let mut lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    let waiting = super::server_conformance_lifecycle::room_view(
        &lifecycle,
        StormCloudConfig::Disabled,
        false,
    );
    lifecycle.set_ready(SeatId::One, true).unwrap();
    let countdown = super::server_conformance_lifecycle::room_view(
        &lifecycle,
        StormCloudConfig::Disabled,
        false,
    );
    for mut candidate in [waiting, countdown] {
        assert!(super::room_validation::validate_room_view(&candidate).is_ok());
        candidate.seats[1].ready = false;
        assert_eq!(
            super::room_validation::validate_room_view(&candidate),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

fn running_room() -> RoomView {
    let lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    super::server_conformance_lifecycle::room_view(&lifecycle, StormCloudConfig::Disabled, true)
}

fn waiting_room() -> RoomView {
    let lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    super::server_conformance_lifecycle::room_view(&lifecycle, StormCloudConfig::Disabled, false)
}

#[test]
fn checked_room_validation_rejects_reviewed_metadata_and_history_mutations() {
    let mutations: &[fn(&mut RoomView)] = &[
        |room| room.available_items = ItemSet::empty(),
        |room| room.seats[0].occupied = false,
        |room| room.seats[0].ready = true,
        |room| room.seats[1].ready = true,
        |room| room.seats[0].rematch_ready = true,
        |room| room.scores[0].wins = Wins::new(4).unwrap(),
        |room| {
            room.seats[0].connection = ConnectionStatus::Disconnected {
                expires_at: Tick::new(room.tick.get() + 601).unwrap(),
            };
        },
        |room| {
            room.phase = RoomPhase::MatchResult {
                ended_at: room.tick,
                result: MatchResult::Winner {
                    seat: Seat::Four,
                    reason: MatchWinnerReason::Forfeit,
                },
                rematch_at: Tick::new(room.tick.get() + 180).unwrap(),
            };
        },
    ];
    for mutate in mutations {
        let mut room = running_room();
        mutate(&mut room);
        assert_eq!(
            super::room_validation::validate_room_view(&room),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }

    let mut waiting = waiting_room();
    waiting.round_number = RoundCount::new(1).unwrap();
    assert!(super::room_validation::validate_room_view(&waiting).is_err());
    let mut waiting = waiting_room();
    waiting.scores[0].wins = Wins::new(1).unwrap();
    assert!(super::room_validation::validate_room_view(&waiting).is_err());
    let mut waiting = waiting_room();
    waiting.last_round = Some(CompletedRound {
        round_id: counter(1),
        round_number: counter(1),
        ended_at: waiting.tick,
        result: RoundResult::Winner { seat: Seat::One },
    });
    assert!(super::room_validation::validate_room_view(&waiting).is_err());
}

#[test]
fn checked_room_validation_rejects_terminal_deadline_mutation() {
    let mut lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        DomainBoard::ElevenByNine,
        EnabledItems::EMPTY,
        true,
    );
    for round in 0..3 {
        lifecycle
            .complete_current_tick(crate::game::RoundResult::Winner(SeatId::One))
            .unwrap();
        if round < 2 {
            for _ in 0..crate::room::COUNTDOWN_TICKS * 2 {
                lifecycle.advance_idle_tick().unwrap();
            }
        }
    }
    let mut terminal = super::server_conformance_lifecycle::room_view(
        &lifecycle,
        StormCloudConfig::Disabled,
        true,
    );
    assert!(super::room_validation::validate_room_view(&terminal).is_ok());
    let RoomPhase::MatchResult { rematch_at, .. } = &mut terminal.phase else {
        panic!("expected terminal room");
    };
    *rematch_at = Tick::new(rematch_at.get() + 1).unwrap();
    assert_eq!(
        super::room_validation::validate_room_view(&terminal),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}
