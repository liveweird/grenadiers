use crate::game::{EventAudience, PickupKind as GamePickupKind, StepEvent, StepEventKind};
use crate::protocol::RulesVersion;

use super::projection_facts::ProjectionError;
use super::projection_test_support::{classic_state_at, room};
use super::*;

fn pickup_report(state: &crate::game::State, cell: crate::game::CellId) -> crate::game::StepReport {
    let tick = state.tick();
    crate::game::StepReport {
        tick,
        traces: crate::game::SeatId::DUEL
            .map(|seat| crate::game::MovementTrace::new(state.actors()[seat.index()].position()))
            .to_vec(),
        forced_movement: None,
        events: vec![StepEvent::new(
            tick,
            EventAudience::Both,
            StepEventKind::PickupCollected {
                id: state.actors()[0].id(),
                seat: crate::game::SeatId::One,
                cell,
                kind: GamePickupKind::BombCapacity,
            },
        )],
    }
}

#[test]
fn public_capture_rejects_pickup_on_surviving_block_and_binds_absent_floor_pickup() {
    let state = classic_state_at(1);
    let blocked = state.public_view().arena.blocks[0];
    assert_eq!(
        capture_classic(&state, &[pickup_report(&state, blocked)]),
        Err(ProjectionError::InvalidEvent)
    );

    let floor = state.actors()[0].position().cell().unwrap();
    assert!(state.public_view().pickups.is_empty());
    let capture = capture_classic(&state, &[pickup_report(&state, floor)]).unwrap();
    let room = room(
        1,
        GameSettings::for_rules(RulesVersion::ClassicV1),
        ItemSet::empty(),
    );
    assert!(bind_classic(&room, room.round_id, &capture).is_ok());
}

#[test]
fn classic_capture_rejects_expanded_item_only_event_records() {
    let state = classic_state_at(1);
    let floor = state.actors()[0].position().cell().unwrap();
    for pickup_kind in [GamePickupKind::Heart, GamePickupKind::Shield] {
        let mut pickup = pickup_report(&state, floor);
        let StepEventKind::PickupCollected { ref mut kind, .. } = pickup.events[0].kind else {
            unreachable!()
        };
        *kind = pickup_kind;
        assert_eq!(
            capture_classic(&state, &[pickup]),
            Err(ProjectionError::InvalidItems)
        );
    }

    let tick = state.tick();
    for source in [
        crate::game::DamagePreventionSource::Heart,
        crate::game::DamagePreventionSource::Shield,
    ] {
        let prevented = crate::game::StepReport {
            tick,
            traces: crate::game::SeatId::DUEL
                .map(|seat| {
                    crate::game::MovementTrace::new(state.actors()[seat.index()].position())
                })
                .to_vec(),
            forced_movement: None,
            events: vec![StepEvent::new(
                tick,
                EventAudience::Both,
                StepEventKind::DamagePrevented {
                    id: state.actors()[0].id(),
                    seat: crate::game::SeatId::One,
                    source,
                },
            )],
        };
        assert_eq!(
            capture_classic(&state, &[prevented]),
            Err(ProjectionError::InvalidItems)
        );
    }
}

#[test]
fn classic_capture_rejects_extra_seat_collector_audience_and_winner_facts() {
    let state = classic_state_at(1);
    let floor = state.actors()[0].position().cell().unwrap();
    let mut report = pickup_report(&state, floor);
    let StepEventKind::PickupCollected { ref mut seat, .. } = report.events[0].kind else {
        unreachable!()
    };
    *seat = crate::game::SeatId::Three;
    assert_eq!(
        capture_classic(&state, &[report]),
        Err(ProjectionError::InvalidEvent)
    );

    let mut report = pickup_report(&state, floor);
    report.events[0].audience = EventAudience::only(crate::game::SeatId::Four);
    assert_eq!(
        capture_classic(&state, &[report]),
        Err(ProjectionError::InvalidReports)
    );

    let mut report = pickup_report(&state, floor);
    report.events[0].kind =
        StepEventKind::RoundEnded(crate::game::RoundResult::Winner(crate::game::SeatId::Four));
    assert_eq!(
        capture_classic(&state, &[report]),
        Err(ProjectionError::InvalidResult)
    );

    let mut report = pickup_report(&state, floor);
    report.traces.push(report.traces[0].clone());
    assert_eq!(
        capture_classic(&state, &[report]),
        Err(ProjectionError::InvalidReports)
    );
}

#[test]
fn classic_capture_rejects_only_unsupported_forced_movement_victims() {
    let report = crate::protocol::forced_movement_test_support::bracelet_shove_report();
    let state = classic_state_at(report.tick.0);
    assert!(capture_classic(&state, std::slice::from_ref(&report)).is_ok());

    let mut malicious = report;
    malicious.forced_movement.as_mut().unwrap().seat = crate::game::SeatId::Four;
    assert_eq!(
        capture_classic(&state, &[malicious]),
        Err(ProjectionError::InvalidReports)
    );
}
