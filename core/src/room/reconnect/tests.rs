use super::*;

fn occupied_pair(at: u64) -> ReconnectGrace {
    let mut grace = ReconnectGrace::new();
    grace.occupy(SeatId::One, Tick(at)).unwrap();
    grace.occupy(SeatId::Two, Tick(at)).unwrap();
    grace
}

#[test]
fn resume_is_allowed_at_599_and_rejected_at_600() {
    let mut before = occupied_pair(10);
    assert_eq!(
        before.disconnect(SeatId::One, Tick(10)),
        Ok(DisconnectStatus::Started {
            expires_at: Tick(610)
        })
    );
    assert_eq!(before.resume(SeatId::One, Tick(609)), Ok(()));
    assert_eq!(before.seat(SeatId::One), ReservationState::Connected);

    let mut boundary = occupied_pair(10);
    boundary.disconnect(SeatId::One, Tick(10)).unwrap();
    let unchanged = boundary.clone();
    assert_eq!(
        boundary.resume(SeatId::One, Tick(610)),
        Err(ReconnectError::GraceExpired {
            seat: SeatId::One,
            expires_at: Tick(610),
        })
    );
    assert_eq!(boundary, unchanged);
    let facts = boundary.expire_due(Tick(610)).unwrap();
    assert_eq!(facts.newly_expired(), [true, false]);
    assert_eq!(facts.states()[0], ReservationState::Expired);
}

#[test]
fn duplicate_loss_never_extends_the_original_deadline() {
    let mut grace = occupied_pair(4);
    grace.disconnect(SeatId::Two, Tick(20)).unwrap();
    assert_eq!(
        grace.disconnect(SeatId::Two, Tick(400)),
        Ok(DisconnectStatus::AlreadyDisconnected {
            expires_at: Tick(620),
        })
    );
    assert_eq!(grace.latest_tick(), Tick(400));
    assert_eq!(
        grace.seat(SeatId::Two),
        ReservationState::Disconnected {
            expires_at: Tick(620)
        }
    );
    assert_eq!(
        grace.expire_due(Tick(620)).unwrap().newly_expired(),
        [false, true]
    );
}

#[test]
fn staggered_expiry_preserves_the_other_seats_independent_eligibility() {
    let mut grace = occupied_pair(0);
    grace.disconnect(SeatId::One, Tick(10)).unwrap();
    grace.disconnect(SeatId::Two, Tick(20)).unwrap();

    let first = grace.expire_due(Tick(610)).unwrap();
    assert_eq!(first.newly_expired(), [true, false]);
    assert_eq!(
        grace.seats(),
        &[
            ReservationState::Expired,
            ReservationState::Disconnected {
                expires_at: Tick(620)
            },
        ]
    );
    assert_eq!(grace.resume(SeatId::Two, Tick(619)), Ok(()));
    assert_eq!(
        grace.resume(SeatId::One, Tick(619)),
        Err(ReconnectError::SeatExpired(SeatId::One))
    );
}

#[test]
fn both_seats_expire_as_one_bounded_fact_batch_in_either_loss_order() {
    for order in [[SeatId::One, SeatId::Two], [SeatId::Two, SeatId::One]] {
        let mut grace = occupied_pair(0);
        grace.disconnect(order[0], Tick(30)).unwrap();
        grace.disconnect(order[1], Tick(30)).unwrap();
        let facts = grace.expire_due(Tick(630)).unwrap();
        assert_eq!(facts.at(), Tick(630));
        assert_eq!(facts.newly_expired(), [true, true]);
        assert_eq!(facts.states(), [ReservationState::Expired; 2]);
        assert_eq!(
            grace.expire_due(Tick(630)).unwrap().newly_expired(),
            [false; 2]
        );
    }
}

#[test]
fn expiry_facts_distinguish_connected_disconnected_expired_and_vacant() {
    let mut grace = ReconnectGrace::new();
    grace.occupy(SeatId::One, Tick(5)).unwrap();
    let facts = grace.expire_due(Tick(5)).unwrap();
    assert_eq!(
        facts.states(),
        [ReservationState::Connected, ReservationState::Vacant]
    );
    assert_eq!(facts.newly_expired(), [false, false]);

    assert_eq!(grace.release(SeatId::One, Tick(6)), Ok(true));
    assert_eq!(grace.release(SeatId::One, Tick(6)), Ok(false));
    assert_eq!(grace.seats(), &[ReservationState::Vacant; 2]);
}

#[test]
fn overflow_and_stale_operations_are_atomic() {
    let mut grace = ReconnectGrace::new();
    grace.occupy(SeatId::One, Tick(u64::MAX)).unwrap();
    let before_overflow = grace.clone();
    assert_eq!(
        grace.disconnect(SeatId::One, Tick(u64::MAX)),
        Err(ReconnectError::TickOverflow)
    );
    assert_eq!(grace, before_overflow);

    let mut monotonic = occupied_pair(50);
    monotonic.disconnect(SeatId::One, Tick(60)).unwrap();
    let before_stale = monotonic.clone();
    assert_eq!(
        monotonic.expire_due(Tick(59)),
        Err(ReconnectError::NonMonotonicTick {
            received: Tick(59),
            earliest: Tick(60),
        })
    );
    assert_eq!(monotonic, before_stale);
}

#[test]
fn invalid_state_operations_do_not_change_clock_or_seats() {
    let mut grace = occupied_pair(7);
    let connected = grace.clone();
    assert_eq!(
        grace.resume(SeatId::One, Tick(8)),
        Err(ReconnectError::SeatNotDisconnected(SeatId::One))
    );
    assert_eq!(grace, connected);

    grace.release(SeatId::Two, Tick(9)).unwrap();
    let vacant = grace.clone();
    assert_eq!(
        grace.disconnect(SeatId::Two, Tick(10)),
        Err(ReconnectError::SeatVacant(SeatId::Two))
    );
    assert_eq!(grace, vacant);
}
