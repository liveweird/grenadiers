use super::*;
use crate::game::DrawReason;

fn occupied_room() -> RoomLifecycle {
    let mut room = RoomLifecycle::new();
    room.occupy(SeatId::One).unwrap();
    room.occupy(SeatId::Two).unwrap();
    room
}

fn countdown(last_ready: SeatId) -> RoomLifecycle {
    let mut room = occupied_room();
    let first = match last_ready {
        SeatId::One => SeatId::Two,
        SeatId::Two => SeatId::One,
    };
    room.set_ready(first, true).unwrap();
    room.set_ready(last_ready, true).unwrap();
    room
}

fn running_room() -> RoomLifecycle {
    let mut room = countdown(SeatId::Two);
    for _ in 0..COUNTDOWN_TICKS {
        room.advance_idle_tick().unwrap();
    }
    room
}

#[test]
fn defaults_to_canonical_vacant_waiting_room() {
    let room = RoomLifecycle::new();
    assert_eq!(room.tick(), Tick(0));
    assert_eq!(room.round_attempt(), 1);
    assert_eq!(room.phase(), Phase::Waiting);
    assert!(
        room.seats()
            .iter()
            .all(|seat| !seat.occupied() && !seat.ready())
    );
    assert_eq!(room, RoomLifecycle::default());
}

#[test]
fn either_seat_can_finish_countdown_at_the_exact_boundary() {
    for last_ready in SeatId::ALL {
        let mut room = countdown(last_ready);
        assert_eq!(room.phase(), Phase::Countdown { ends_at: Tick(180) });
        for _ in 0..179 {
            assert_eq!(room.advance_idle_tick(), Ok(Change::Changed));
        }
        assert_eq!(room.tick(), Tick(179));
        assert!(matches!(room.phase(), Phase::Countdown { .. }));
        assert_eq!(room.advance_idle_tick(), Ok(Change::Started));
        assert_eq!(room.tick(), Tick(180));
        assert_eq!(room.phase(), Phase::Running);
        assert!(room.seats().iter().all(|seat| !seat.ready()));
        assert_eq!(room.complete_running_tick(None), Ok(Change::Changed));
        assert_eq!(room.tick(), Tick(181));
    }
}

#[test]
fn readiness_repetition_and_countdown_cancellation_are_deterministic() {
    let mut room = occupied_room();
    assert_eq!(room.set_ready(SeatId::One, false), Ok(Change::Unchanged));
    assert_eq!(room.set_ready(SeatId::One, true), Ok(Change::Changed));
    assert_eq!(room.set_ready(SeatId::One, true), Ok(Change::Unchanged));
    room.set_ready(SeatId::Two, true).unwrap();

    assert_eq!(room.set_ready(SeatId::One, false), Ok(Change::Changed));
    assert_eq!(room.phase(), Phase::Waiting);
    assert_eq!(room.round_attempt(), 2);
    assert!(room.seats().iter().all(|seat| !seat.ready()));
    assert_eq!(room.set_ready(SeatId::One, false), Ok(Change::Unchanged));

    room.set_ready(SeatId::One, true).unwrap();
    room.set_ready(SeatId::Two, true).unwrap();
    assert_eq!(
        room.depart(SeatId::Two, DepartureReason::Explicit),
        Ok(Change::Changed)
    );
    assert_eq!(room.phase(), Phase::Waiting);
    assert_eq!(room.round_attempt(), 3);
    assert!(room.seats().iter().all(|seat| !seat.ready()));
    assert_eq!(
        room.depart(SeatId::Two, DepartureReason::Explicit),
        Ok(Change::Unchanged)
    );
    assert_eq!(room.round_attempt(), 3);
}

#[test]
fn waiting_departure_clears_readiness_without_advancing_attempt() {
    let mut room = occupied_room();
    room.set_ready(SeatId::One, true).unwrap();

    assert_eq!(
        room.depart(SeatId::Two, DepartureReason::TransportLoss),
        Ok(Change::Changed)
    );

    assert_eq!(room.phase(), Phase::Waiting);
    assert_eq!(room.round_attempt(), 1);
    assert!(room.seats().iter().all(|seat| !seat.ready()));
    assert!(room.seats()[0].occupied());
    assert!(!room.seats()[1].occupied());
}

#[test]
fn admission_and_readiness_obey_phase_and_occupancy() {
    let mut room = RoomLifecycle::new();
    assert_eq!(
        room.set_ready(SeatId::One, true),
        Err(LifecycleError::SeatVacant(SeatId::One))
    );
    room.occupy(SeatId::One).unwrap();
    assert_eq!(
        room.occupy(SeatId::One),
        Err(LifecycleError::SeatOccupied(SeatId::One))
    );

    let mut running = running_room();
    let before = running.clone();
    assert_eq!(
        running.occupy(SeatId::One),
        Err(LifecycleError::InvalidPhase)
    );
    assert_eq!(
        running.set_ready(SeatId::One, true),
        Err(LifecycleError::InvalidPhase)
    );
    assert_eq!(running, before);
    assert_eq!(
        running.advance_idle_tick(),
        Err(LifecycleError::InvalidPhase)
    );
}

#[test]
fn running_departures_interrupt_at_the_last_completed_tick() {
    for (departure, expected) in [
        (DepartureReason::Explicit, InterruptReason::ParticipantLeft),
        (
            DepartureReason::TransportLoss,
            InterruptReason::ConnectionLost,
        ),
    ] {
        let mut room = running_room();
        assert_eq!(room.depart(SeatId::One, departure), Ok(Change::Finished));
        assert_eq!(
            room.phase(),
            Phase::Interrupted {
                ended_at: Tick(180),
                reason: expected,
            }
        );
        assert!(!room.seats()[0].occupied());
        assert!(room.seats()[1].occupied());
    }
}

#[test]
fn completed_result_wins_over_later_departures_and_interruptions() {
    for departure in [DepartureReason::Explicit, DepartureReason::TransportLoss] {
        let mut room = running_room();
        let result = RoundResult::Winner(SeatId::One);
        assert_eq!(
            room.complete_running_tick(Some(result)),
            Ok(Change::Finished)
        );
        let terminal = room.phase();
        assert_eq!(
            terminal,
            Phase::Result {
                ended_at: Tick(181),
                result,
            }
        );
        assert_eq!(room.depart(SeatId::Two, departure), Ok(Change::Changed));
        assert_eq!(room.phase(), terminal);
        assert_eq!(
            room.interrupt(InterruptReason::ServerStopped),
            Change::Unchanged
        );
        assert_eq!(room.phase(), terminal);
        assert_eq!(
            room.set_ready(SeatId::One, true),
            Err(LifecycleError::InvalidPhase)
        );
    }
}

#[test]
fn interruption_is_immutable_and_freezes_its_clock() {
    let mut room = occupied_room();
    room.advance_idle_tick().unwrap();
    assert_eq!(
        room.interrupt(InterruptReason::ServerOverloaded),
        Change::Finished
    );
    let terminal = room.clone();
    assert_eq!(room.advance_idle_tick(), Ok(Change::Unchanged));
    assert_eq!(
        room.interrupt(InterruptReason::InternalError),
        Change::Unchanged
    );
    assert_eq!(
        room.complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout))),
        Err(LifecycleError::InvalidPhase)
    );
    assert_eq!(room, terminal);

    let mut result = running_room();
    result
        .complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout)))
        .unwrap();
    assert_eq!(result.advance_idle_tick(), Ok(Change::Changed));
    assert_eq!(result.tick(), Tick(182));
    assert!(matches!(
        result.phase(),
        Phase::Result {
            ended_at: Tick(181),
            ..
        }
    ));
}

#[test]
fn arithmetic_failures_are_atomic() {
    let mut deadline = occupied_room();
    deadline.tick = Tick(u64::MAX - COUNTDOWN_TICKS + 1);
    deadline.set_ready(SeatId::One, true).unwrap();
    let before = deadline.clone();
    assert_eq!(
        deadline.set_ready(SeatId::Two, true),
        Err(LifecycleError::TickOverflow)
    );
    assert_eq!(deadline, before);

    for phase in [
        Phase::Waiting,
        Phase::Result {
            ended_at: Tick(4),
            result: RoundResult::Draw(DrawReason::Timeout),
        },
    ] {
        let mut room = RoomLifecycle {
            tick: Tick(u64::MAX),
            round_attempt: 1,
            phase,
            seats: [SeatState::default(); 2],
        };
        let before = room.clone();
        assert_eq!(room.advance_idle_tick(), Err(LifecycleError::TickOverflow));
        assert_eq!(room, before);
    }

    let mut running = running_room();
    running.tick = Tick(u64::MAX);
    let before = running.clone();
    assert_eq!(
        running.complete_running_tick(Some(RoundResult::Winner(SeatId::One))),
        Err(LifecycleError::TickOverflow)
    );
    assert_eq!(running, before);

    for cancel_by_departure in [false, true] {
        let mut room = countdown(SeatId::Two);
        room.round_attempt = u64::MAX;
        let before = room.clone();
        let change = if cancel_by_departure {
            room.depart(SeatId::One, DepartureReason::Explicit)
        } else {
            room.set_ready(SeatId::One, false)
        };
        assert_eq!(change, Err(LifecycleError::RoundAttemptOverflow));
        assert_eq!(room, before);
    }
}

#[test]
fn identical_action_logs_replay_to_equal_state() {
    fn replay() -> RoomLifecycle {
        let mut room = RoomLifecycle::new();
        room.advance_idle_tick().unwrap();
        room.occupy(SeatId::Two).unwrap();
        room.occupy(SeatId::One).unwrap();
        room.set_ready(SeatId::Two, true).unwrap();
        room.set_ready(SeatId::One, true).unwrap();
        for _ in 0..COUNTDOWN_TICKS {
            room.advance_idle_tick().unwrap();
        }
        room.complete_running_tick(None).unwrap();
        room.complete_running_tick(Some(RoundResult::Winner(SeatId::Two)))
            .unwrap();
        room
    }

    assert_eq!(replay(), replay());
}
