use crate::game::{EventAudience, PickupKind as GamePickupKind, StepEvent, StepEventKind};
use crate::protocol::RulesVersion;

use super::projection_facts::ProjectionError;
use super::projection_test_support::{classic_state_at, room};
use super::*;

fn pickup_report(state: &crate::game::State, cell: crate::game::CellId) -> crate::game::StepReport {
    let tick = state.tick();
    crate::game::StepReport {
        tick,
        traces: crate::game::SeatId::ALL
            .map(|seat| crate::game::MovementTrace::new(state.actors()[seat.index()].position())),
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
fn classic_capture_rejects_heart_only_event_records() {
    let state = classic_state_at(1);
    let floor = state.actors()[0].position().cell().unwrap();
    let mut pickup = pickup_report(&state, floor);
    let StepEventKind::PickupCollected { ref mut kind, .. } = pickup.events[0].kind else {
        unreachable!()
    };
    *kind = GamePickupKind::Heart;
    assert_eq!(
        capture_classic(&state, &[pickup]),
        Err(ProjectionError::InvalidItems)
    );

    let tick = state.tick();
    let prevented = crate::game::StepReport {
        tick,
        traces: crate::game::SeatId::ALL
            .map(|seat| crate::game::MovementTrace::new(state.actors()[seat.index()].position())),
        events: vec![StepEvent::new(
            tick,
            EventAudience::Both,
            StepEventKind::DamagePrevented {
                id: state.actors()[0].id(),
                seat: crate::game::SeatId::One,
                source: crate::game::DamagePreventionSource::Heart,
            },
        )],
    };
    assert_eq!(
        capture_classic(&state, &[prevented]),
        Err(ProjectionError::InvalidItems)
    );
}
