use super::*;
use crate::game::SpeedLevel;
use crate::room::series::RESULT_HOLD_TICKS;
use crate::room::{BoardSize, COUNTDOWN_TICKS, CpuDifficulty};

fn configuration(controls: &[SeatControl]) -> RoomConfiguration {
    RoomConfiguration::new(
        controls,
        controls
            .contains(&SeatControl::Cpu)
            .then_some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap()
}

fn settings(wins: u8) -> RoomSettings {
    RoomSettings::new(SpeedLevel::Base, 256, wins, 180).unwrap()
}

fn advance(room: &mut RosterMatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        room.advance_idle_tick().unwrap();
    }
}

fn start(room: &mut RosterMatchLifecycle, humans: &[SeatId]) {
    for seat in humans {
        room.occupy(*seat).unwrap();
    }
    for seat in humans {
        room.set_ready(*seat, true).unwrap();
    }
    advance(room, COUNTDOWN_TICKS);
}

fn final_three_seat_room() -> RosterMatchLifecycle {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Cpu]),
        settings(1),
    );
    start(&mut room, &[SeatId::One, SeatId::Two]);
    room.complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    advance(&mut room, RESULT_HOLD_TICKS);
    room
}

#[test]
fn departed_winner_is_rejected_atomically_but_prior_physics_win_remains_final() {
    let controls = [SeatControl::Human, SeatControl::Human, SeatControl::Human];
    let mut rejected = RosterMatchLifecycle::new(configuration(&controls), settings(1));
    start(&mut rejected, &[SeatId::One, SeatId::Two, SeatId::Three]);
    rejected
        .depart_with_grace([None, None, Some(DepartureReason::Explicit), None])
        .unwrap();
    let before = rejected.clone();
    assert_eq!(
        rejected.complete_running_tick(Some(RoundResult::Winner(SeatId::Three))),
        Err(RosterError::IneligibleWinner(SeatId::Three))
    );
    assert_eq!(rejected, before);

    let mut accepted = RosterMatchLifecycle::new(configuration(&controls), settings(1));
    start(&mut accepted, &[SeatId::One, SeatId::Two, SeatId::Three]);
    accepted
        .complete_running_tick(Some(RoundResult::Winner(SeatId::Three)))
        .unwrap();
    let outcome = accepted.outcome().unwrap();
    accepted
        .depart_with_grace([None, None, Some(DepartureReason::Explicit), None])
        .unwrap();
    assert_eq!(accepted.outcome(), Some(outcome));
}

#[test]
fn departure_and_idempotent_ready_reevaluate_remaining_consensus() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    for seat in SeatId::ALL[..3].iter().copied() {
        room.occupy(seat).unwrap();
    }
    room.set_ready(SeatId::One, true).unwrap();
    room.set_ready(SeatId::Two, true).unwrap();
    assert_eq!(
        room.depart_with_grace([None, None, Some(DepartureReason::Explicit), None,])
            .unwrap()
            .change(),
        MatchChange::CountdownStarted
    );

    room.phase = MatchPhase::Waiting;
    assert_eq!(
        room.set_ready(SeatId::One, true).unwrap().change(),
        MatchChange::CountdownStarted
    );
}

#[test]
fn explicit_and_expired_departures_reevaluate_rematch_consensus() {
    let mut explicit = final_three_seat_room();
    explicit.set_rematch_vote(SeatId::One, true).unwrap();
    assert_eq!(
        explicit
            .depart_with_grace([None, Some(DepartureReason::Explicit), None, None])
            .unwrap()
            .change(),
        MatchChange::RematchStarted
    );

    let mut expired = final_three_seat_room();
    expired.set_rematch_vote(SeatId::One, true).unwrap();
    expired
        .depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    advance(&mut expired, crate::room::reconnect::RECONNECT_GRACE_TICKS);
    assert_eq!(
        expired.expire_reconnect_grace().unwrap().change(),
        MatchChange::RematchStarted
    );
    assert_eq!(
        expired.participant(SeatId::Two),
        Ok(RosterParticipant::Departed)
    );
}

#[test]
fn cancellation_and_rematch_epoch_overflow_are_atomic() {
    let mut cancellation = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    cancellation.occupy(SeatId::One).unwrap();
    cancellation.set_ready(SeatId::One, true).unwrap();
    cancellation.round_attempt = u64::MAX;
    let before = cancellation.clone();
    assert_eq!(
        cancellation.set_ready(SeatId::One, false),
        Err(RosterError::RoundAttemptOverflow)
    );
    assert_eq!(cancellation, before);

    let mut rematch = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human]),
        settings(1),
    );
    start(&mut rematch, &[SeatId::One, SeatId::Two]);
    rematch
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    advance(&mut rematch, RESULT_HOLD_TICKS);
    rematch.set_rematch_vote(SeatId::One, true).unwrap();
    rematch.round_attempt = u64::MAX;
    let before = rematch.clone();
    assert_eq!(
        rematch.set_rematch_vote(SeatId::Two, true),
        Err(RosterError::RoundAttemptOverflow)
    );
    assert_eq!(rematch, before);
}
