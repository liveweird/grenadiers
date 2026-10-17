use crate::game::{DrawReason, RoundResult, SeatId};
use crate::protocol::{Counter, MapId, RoomCode, RoomId, RulesVersion};
use crate::room::{DepartureReason, InterruptReason, MatchLifecycle, MatchPhase};

use super::{RoomMetadata, RoomView, project_room_view};

pub(super) fn metadata() -> RoomMetadata {
    RoomMetadata {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        revision: Counter::new(1).unwrap(),
        rules_version: RulesVersion::ClassicV2,
        map_id: MapId::GeneratedV1,
    }
}

pub(super) fn occupied() -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::new();
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle
}

pub(super) fn advance(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

pub(super) fn start_initial(lifecycle: &mut MatchLifecycle) {
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    advance(lifecycle, 180);
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
}

pub(super) fn start_next(lifecycle: &mut MatchLifecycle) {
    advance(lifecycle, 360);
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
}

pub(super) fn project(lifecycle: &MatchLifecycle) -> RoomView {
    project_room_view(metadata(), lifecycle).unwrap()
}

pub(super) fn waiting_view() -> RoomView {
    project(&occupied())
}

pub(super) fn initial_countdown_view() -> RoomView {
    let mut lifecycle = occupied();
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    project(&lifecycle)
}

pub(super) fn first_running_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    project(&lifecycle)
}

pub(super) fn result_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    project(&lifecycle)
}

pub(super) fn inter_countdown_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout)))
        .unwrap();
    advance(&mut lifecycle, 180);
    project(&lifecycle)
}

pub(super) fn later_running_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    start_next(&mut lifecycle);
    project(&lifecycle)
}

pub(super) fn score_terminal_lifecycle() -> MatchLifecycle {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    for win in 1..=3 {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        if win < 3 {
            start_next(&mut lifecycle);
        }
    }
    lifecycle
}

pub(super) fn forfeit_terminal_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap();
    project(&lifecycle)
}

pub(super) fn abandoned_terminal_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .depart_batch([
            Some(DepartureReason::Explicit),
            Some(DepartureReason::Explicit),
        ])
        .unwrap();
    project(&lifecycle)
}

pub(super) fn match_interrupted_view() -> RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .depart_batch([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    project(&lifecycle)
}

pub(super) fn prematch_interrupted_view() -> RoomView {
    let mut lifecycle = occupied();
    lifecycle.interrupt(InterruptReason::ServerStopped).unwrap();
    project(&lifecycle)
}
