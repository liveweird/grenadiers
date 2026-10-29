use super::*;
use crate::game::SpeedLevel;
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
fn duplicate_ready_during_countdown_preserves_original_deadline_and_state() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    room.set_ready(SeatId::One, true).unwrap();
    let MatchPhase::Countdown { ends_at, .. } = room.phase() else {
        panic!("ready consensus must start countdown");
    };
    advance(&mut room, 30);
    let before = room.clone();
    assert_eq!(
        room.set_ready(SeatId::One, true).unwrap().change(),
        MatchChange::Unchanged
    );
    assert_eq!(room, before);
    assert!(
        matches!(room.phase(), MatchPhase::Countdown { ends_at: actual, .. } if actual == ends_at)
    );
    advance(&mut room, COUNTDOWN_TICKS - 30);
    assert_eq!(room.phase(), MatchPhase::Running);
}

#[test]
fn terminal_departures_retire_immediately_and_reevaluate_rematch_consensus() {
    let mut explicit = final_three_seat_room();
    explicit.set_rematch_vote(SeatId::One, true).unwrap();
    assert_eq!(
        explicit
            .depart_with_grace([None, Some(DepartureReason::Explicit), None, None])
            .unwrap()
            .change(),
        MatchChange::RematchStarted
    );

    let mut disconnected = final_three_seat_room();
    let outcome = disconnected.outcome().unwrap();
    let update = disconnected
        .depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    assert_eq!(update.change(), MatchChange::Changed);
    assert_eq!(
        update.newly_ineligible_seats().collect::<Vec<_>>(),
        vec![SeatId::Two]
    );
    assert_eq!(disconnected.outcome(), Some(outcome));
    assert_eq!(
        disconnected.participant(SeatId::Two),
        Ok(RosterParticipant::Departed)
    );
    assert_eq!(
        disconnected
            .set_rematch_vote(SeatId::One, true)
            .unwrap()
            .change(),
        MatchChange::RematchStarted
    );
}

#[test]
fn terminal_transport_loss_at_end_tick_or_later_never_starts_fresh_grace() {
    for delay in [0, 1, 37] {
        let mut room = RosterMatchLifecycle::new(
            configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Cpu]),
            settings(1),
        );
        start(&mut room, &[SeatId::One, SeatId::Two]);
        room.complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        let outcome = room.outcome().unwrap();
        advance(&mut room, delay);

        let update = room
            .depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
            .unwrap();

        assert_eq!(room.outcome(), Some(outcome));
        assert_eq!(
            update.newly_ineligible_seats().collect::<Vec<_>>(),
            vec![SeatId::Two]
        );
        assert_eq!(
            room.participant(SeatId::Two),
            Ok(RosterParticipant::Departed)
        );
        assert_eq!(
            room.resume_connection(SeatId::Two),
            Err(RosterError::SeatNotReserved(SeatId::Two))
        );
    }
}

#[test]
fn reservation_created_before_terminal_keeps_its_deadline_and_resume_policy() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Cpu]),
        settings(1),
    );
    start(&mut room, &[SeatId::One, SeatId::Two]);
    room.depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    let RosterParticipant::Reserved { expires_at } = room.participant(SeatId::Two).unwrap() else {
        panic!("transport loss before the final outcome must reserve the seat");
    };
    room.complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let outcome = room.outcome().unwrap();

    let mut resumed = room.clone();
    resumed.resume_connection(SeatId::Two).unwrap();
    assert_eq!(
        resumed.participant(SeatId::Two),
        Ok(RosterParticipant::Connected)
    );
    assert_eq!(resumed.outcome(), Some(outcome));
    let second_loss = resumed
        .depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    assert_eq!(resumed.outcome(), Some(outcome));
    assert_eq!(
        second_loss.newly_ineligible_seats().collect::<Vec<_>>(),
        vec![SeatId::Two]
    );
    assert_eq!(
        resumed.participant(SeatId::Two),
        Ok(RosterParticipant::Departed)
    );

    advance(&mut room, RESULT_HOLD_TICKS);
    room.set_rematch_vote(SeatId::One, true).unwrap();
    let ticks_until_expiry = expires_at.0 - room.tick().0;
    advance(&mut room, ticks_until_expiry);
    assert!(matches!(
        room.resume_connection(SeatId::Two),
        Err(RosterError::GraceExpired {
            seat: SeatId::Two,
            expires_at: actual,
        }) if actual == expires_at
    ));
    let update = room.expire_reconnect_grace().unwrap();
    assert_eq!(update.change(), MatchChange::RematchStarted);
    assert_eq!(room.outcome(), None);
    assert_eq!(
        update.newly_ineligible_seats().collect::<Vec<_>>(),
        vec![SeatId::Two]
    );
    assert_eq!(
        room.participant(SeatId::Two),
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

#[test]
fn inter_round_disconnect_preserves_completed_round_hold_and_delays_restart() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Cpu]),
        settings(3),
    );
    start(&mut room, &[SeatId::One, SeatId::Two]);
    room.complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let completed = room.latest_completed().unwrap();
    let original_hold = Tick(completed.ended_at().0 + RESULT_HOLD_TICKS);
    advance(&mut room, RESULT_HOLD_TICKS);
    assert!(matches!(
        room.phase(),
        MatchPhase::Countdown {
            kind: CountdownKind::InterRound,
            ..
        }
    ));
    advance(&mut room, 30);
    room.depart_with_grace([None, Some(DepartureReason::TransportLoss), None, None])
        .unwrap();
    assert_eq!(
        room.phase(),
        MatchPhase::Result {
            next_countdown_at: original_hold
        }
    );
    assert_eq!(room.latest_completed(), Some(completed));
    advance(&mut room, 20);
    assert_eq!(
        room.phase(),
        MatchPhase::Result {
            next_countdown_at: original_hold
        }
    );
    room.resume_connection(SeatId::Two).unwrap();
    room.advance_idle_tick().unwrap();
    assert!(matches!(
        room.phase(),
        MatchPhase::Countdown {
            kind: CountdownKind::InterRound,
            ..
        }
    ));
}

#[test]
fn reserved_creator_keeps_authority_but_cannot_change_settings_until_connected() {
    let mut room = RosterMatchLifecycle::new(
        configuration(&[SeatControl::Human, SeatControl::Human, SeatControl::Human]),
        settings(3),
    );
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Two).unwrap();
    room.depart_with_grace([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap();
    assert_eq!(room.settings_editor(), Some(SeatId::One));
    let before = room.clone();
    assert_eq!(
        room.set_settings(SeatId::One, settings(2)),
        Err(RosterError::SeatNotConnected(SeatId::One))
    );
    assert_eq!(
        room.set_settings(SeatId::Two, settings(2)),
        Err(RosterError::UnauthorizedSettingsEditor(SeatId::Two))
    );
    assert_eq!(room, before);
    room.resume_connection(SeatId::One).unwrap();
    room.set_settings(SeatId::One, settings(2)).unwrap();
    assert_eq!(room.settings(), settings(2));
    room.depart_with_grace([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();
    assert_eq!(room.settings_editor(), None);
}
