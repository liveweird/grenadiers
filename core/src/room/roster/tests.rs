use super::*;
use crate::game::{DrawReason, SpeedLevel};
use crate::room::reconnect::RECONNECT_GRACE_TICKS;
use crate::room::series::MatchResult;
use crate::room::series::RESULT_HOLD_TICKS;
use crate::room::{BoardSize, COUNTDOWN_TICKS, CountdownKind, CpuDifficulty};

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

#[test]
fn pending_human_blocks_start_but_one_human_and_cpu_can_start() {
    let mut pending = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    pending.occupy(SeatId::One).unwrap();
    assert_eq!(
        pending.set_ready(SeatId::One, true).unwrap().change(),
        MatchChange::Changed
    );
    assert_eq!(pending.phase(), MatchPhase::Waiting);

    let mut solo = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    solo.occupy(SeatId::One).unwrap();
    assert_eq!(
        solo.set_ready(SeatId::One, true).unwrap().change(),
        MatchChange::CountdownStarted
    );
    advance(&mut solo, COUNTDOWN_TICKS);
    assert_eq!(solo.phase(), MatchPhase::Running);
}

#[test]
fn countdown_cancel_is_transactional_and_advances_attempt() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Two).unwrap();
    room.set_ready(SeatId::One, true).unwrap();
    room.set_ready(SeatId::Two, true).unwrap();
    assert!(matches!(
        room.phase(),
        MatchPhase::Countdown {
            kind: CountdownKind::Initial,
            ..
        }
    ));
    room.set_ready(SeatId::One, false).unwrap();
    assert_eq!(room.phase(), MatchPhase::Waiting);
    assert_eq!(room.round_attempt(), 2);
}

#[test]
fn forming_departure_reopens_seat_and_cancels_countdown_without_changing_legacy_policy() {
    let mut forming = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    for seat in SeatId::DUEL {
        forming.occupy(seat).unwrap();
        forming.set_ready(seat, true).unwrap();
    }
    forming
        .depart_forming_with_grace([None, Some(DepartureReason::Explicit), None, None])
        .unwrap();
    assert_eq!(forming.phase(), MatchPhase::Waiting);
    assert_eq!(forming.ready(SeatId::One), Ok(false));
    assert_eq!(
        forming.participant(SeatId::Two),
        Ok(RosterParticipant::NeverAdmitted)
    );

    let mut legacy = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    legacy.occupy(SeatId::One).unwrap();
    legacy.occupy(SeatId::Two).unwrap();
    legacy
        .depart_with_grace([None, Some(DepartureReason::Explicit), None, None])
        .unwrap();
    assert_eq!(
        legacy.participant(SeatId::Two),
        Ok(RosterParticipant::Departed)
    );
}

#[test]
fn forming_transport_reserves_until_exact_expiry_then_reopens() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Two).unwrap();
    room.depart_forming_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    assert!(matches!(
        room.participant(SeatId::Two),
        Ok(RosterParticipant::Reserved { .. })
    ));
    advance(&mut room, RECONNECT_GRACE_TICKS);
    room.expire_forming_reconnect_grace().unwrap();
    assert_eq!(
        room.participant(SeatId::Two),
        Ok(RosterParticipant::NeverAdmitted)
    );
}

#[test]
fn permanently_departed_human_does_not_block_remaining_eligible_humans() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    for seat in [SeatId::One, SeatId::Two, SeatId::Three] {
        room.occupy(seat).unwrap();
    }
    room.depart_with_grace([None, None, Some(DepartureReason::Explicit), None])
        .unwrap();
    room.set_ready(SeatId::One, true).unwrap();
    assert_eq!(
        room.set_ready(SeatId::Two, true).unwrap().change(),
        MatchChange::CountdownStarted
    );
}

#[test]
fn grace_expiry_forfeits_without_fabricating_round() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Two).unwrap();
    room.set_ready(SeatId::One, true).unwrap();
    room.set_ready(SeatId::Two, true).unwrap();
    advance(&mut room, COUNTDOWN_TICKS);
    room.complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout)))
        .unwrap();
    let completed = room.latest_completed();
    room.depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    advance(&mut room, RECONNECT_GRACE_TICKS);
    let update = room.expire_reconnect_grace().unwrap();
    assert_eq!(
        update.newly_ineligible_seats().collect::<Vec<_>>(),
        vec![SeatId::Two]
    );
    assert_eq!(room.latest_completed(), completed);
    assert_eq!(room.scores(), &[0, 0]);
    assert!(
        matches!(room.phase(), MatchPhase::MatchResult { outcome, .. }
        if outcome.result() == MatchResult::ForfeitWinner(SeatId::One))
    );
}

#[test]
fn final_outcome_is_immutable_and_departed_seats_do_not_return_on_rematch() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Cpu]),
        settings(1),
    );
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Two).unwrap();
    room.set_ready(SeatId::One, true).unwrap();
    room.set_ready(SeatId::Two, true).unwrap();
    advance(&mut room, COUNTDOWN_TICKS);
    room.complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let outcome = room.outcome.unwrap();
    room.depart_with_grace([None, Some(DepartureReason::Explicit), None, None])
        .unwrap();
    assert_eq!(room.outcome, Some(outcome));
    advance(&mut room, RESULT_HOLD_TICKS);
    assert_eq!(
        room.set_rematch_vote(SeatId::One, true).unwrap().change(),
        MatchChange::RematchStarted
    );
    assert_eq!(
        room.participant(SeatId::Two),
        Ok(RosterParticipant::Departed)
    );
}

#[test]
fn invalid_tail_and_cpu_departures_leave_state_unchanged() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu, SeatControl::Human]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    let before = room.clone();
    assert_eq!(
        room.depart_with_grace([None, Some(DepartureReason::Explicit), None, None]),
        Err(RosterError::CpuTarget(SeatId::Two))
    );
    assert_eq!(room, before);
    assert_eq!(
        room.depart_with_grace([None, None, None, Some(DepartureReason::Explicit)]),
        Err(RosterError::InvalidDepartureTail(SeatId::Four))
    );
    assert_eq!(room, before);
}

#[test]
fn mixed_four_seat_departures_settle_once_and_current_tick_completion_does_not_advance() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[
            SeatControl::Human,
            SeatControl::Human,
            SeatControl::Human,
            SeatControl::Human,
        ]),
        settings(3),
    );
    for seat in SeatId::ALL {
        room.occupy(seat).unwrap();
        room.set_ready(seat, true).unwrap();
    }
    advance(&mut room, COUNTDOWN_TICKS);
    let at = room.tick();
    let update = room
        .depart_with_grace([
            None,
            None,
            Some(DepartureReason::Explicit),
            Some(DepartureReason::TransportLoss),
        ])
        .unwrap();
    assert_eq!(
        update.newly_ineligible_seats().collect::<Vec<_>>(),
        vec![SeatId::Three]
    );
    assert_eq!(room.phase(), MatchPhase::Running);
    room.complete_current_tick(RoundResult::Draw(DrawReason::MutualElimination))
        .unwrap();
    assert_eq!(room.tick(), at);
    assert_eq!(room.latest_completed().unwrap().ended_at(), at);
}

#[test]
fn resume_at_expiry_boundary_is_rejected_then_expiry_is_a_tombstone() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    room.set_ready(SeatId::One, true).unwrap();
    advance(&mut room, COUNTDOWN_TICKS);
    room.depart_with_grace([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap();
    for _ in 0..RECONNECT_GRACE_TICKS {
        room.complete_running_tick(None).unwrap();
    }
    let before = room.clone();
    assert!(matches!(
        room.resume_connection(SeatId::One),
        Err(RosterError::GraceExpired { .. })
    ));
    assert_eq!(room, before);
    room.expire_reconnect_grace().unwrap();
    assert_eq!(
        room.participant(SeatId::One),
        Ok(RosterParticipant::Departed)
    );
    assert!(matches!(
        room.phase(),
        MatchPhase::MatchResult { outcome, .. }
            if outcome.result() == MatchResult::Abandoned
    ));
}

#[test]
fn clock_and_deadline_overflow_leave_state_unchanged() {
    let mut clock = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    clock.tick = Tick(u64::MAX);
    let before = clock.clone();
    assert_eq!(clock.advance_idle_tick(), Err(RosterError::TickOverflow));
    assert_eq!(clock, before);

    let mut deadline = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    deadline.occupy(SeatId::One).unwrap();
    deadline.tick = Tick(u64::MAX - COUNTDOWN_TICKS + 1);
    let before = deadline.clone();
    assert_eq!(
        deadline.set_ready(SeatId::One, true),
        Err(RosterError::TickOverflow)
    );
    assert_eq!(deadline, before);
}

#[test]
fn publication_getters_check_seats_and_derive_cpu_consent_from_phase() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu, SeatControl::Human]),
        settings(1),
    );
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Three).unwrap();
    assert_eq!(room.actual_round_number(), 0);
    assert_eq!(room.outcome(), None);
    assert_eq!(room.ready(SeatId::One), Ok(false));
    assert_eq!(room.ready(SeatId::Two), Ok(true));
    assert_eq!(
        room.ready(SeatId::Four),
        Err(RosterError::UnsupportedSeat(SeatId::Four))
    );
    assert_eq!(
        room.rematch_vote(SeatId::Four),
        Err(RosterError::UnsupportedSeat(SeatId::Four))
    );

    room.set_ready(SeatId::One, true).unwrap();
    room.set_ready(SeatId::Three, true).unwrap();
    assert_eq!(room.ready(SeatId::One), Ok(true));
    assert_eq!(room.ready(SeatId::Two), Ok(true));
    advance(&mut room, COUNTDOWN_TICKS);
    assert_eq!(room.actual_round_number(), 1);
    assert_eq!(room.ready(SeatId::One), Ok(false));
    assert_eq!(room.ready(SeatId::Two), Ok(false));

    room.complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let outcome = room.outcome().unwrap();
    assert_eq!(outcome.result(), MatchResult::ScoreWinner(SeatId::One));
    assert_eq!(room.rematch_vote(SeatId::Two), Ok(false));
    advance(&mut room, RESULT_HOLD_TICKS);
    assert_eq!(room.rematch_vote(SeatId::Two), Ok(true));
    assert_eq!(room.rematch_vote(SeatId::One), Ok(false));
    room.set_rematch_vote(SeatId::One, true).unwrap();
    assert_eq!(room.rematch_vote(SeatId::One), Ok(true));
}

#[test]
fn profile_constant_does_not_change_default_activation() {
    assert_eq!(ROSTER_LIFECYCLE_PROFILE, 3);
}
