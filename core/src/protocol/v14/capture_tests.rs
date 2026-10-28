use super::*;
use crate::protocol::Cell;

#[test]
fn message_collection_limits_are_independent_and_exact() {
    assert_eq!(MAX_CLIENT_MESSAGE_BYTES, 4_096);
    assert_eq!(MAX_SERVER_MESSAGE_BYTES, 98_304);
    assert_eq!(super::super::MAX_SERVER_MESSAGE_BYTES, 65_536);
    let exact = "x".repeat(MAX_SERVER_MESSAGE_BYTES);
    assert_eq!(enforce_server_message_size(exact.clone()), Ok(exact));
    assert_eq!(
        enforce_server_message_size("x".repeat(MAX_SERVER_MESSAGE_BYTES + 1)),
        Err(super::super::EncodeServerMessageError::MessageTooLarge),
    );
    assert!(parse_client_message(&"x".repeat(MAX_CLIENT_MESSAGE_BYTES + 1)).is_err());
    assert_eq!(MAX_SNAPSHOT_EVENTS, 192);
    assert_eq!(MAX_EVENT_INDEX, 191);
    assert!(EventIndex::new(191).is_ok());
    assert!(EventIndex::new(192).is_err());
    assert_eq!(crate::protocol::v11::MAX_BOMBS, 16);
    assert_eq!(MAX_BOMBS, 32);
}

#[test]
fn capture_preserves_two_three_and_four_canonical_actors() {
    for count in [
        crate::game::PlayerCount::Two,
        crate::game::PlayerCount::Three,
        crate::game::PlayerCount::Four,
    ] {
        let mut state = super::tests::multiplayer_state(count);
        let report = state.step(&[]).unwrap();
        let capture = capture_expanded(&state, &[report]).unwrap();
        assert_eq!(capture.world.actors.len(), count.get());
        for (&seat, actor) in count.seats().iter().zip(&capture.world.actors) {
            assert_eq!(actor.view.seat, super::projection_capture_wire::seat(seat));
        }
    }
}

#[test]
fn seeded_four_actor_capture_accepts_the_full_item_mask() {
    let mut state = super::tests::multiplayer_state_with_items(
        crate::game::PlayerCount::Four,
        crate::game::EnabledItems::ALL,
    );
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    assert_eq!(
        capture.world.enabled_items.mask().bits(),
        crate::game::EnabledItems::ALL.bits()
    );
}

#[test]
fn mixed_private_pickup_layers_capture_in_global_cell_order() {
    use std::collections::BTreeMap;

    use crate::game::{
        ActorInit, CellId, Direction, EnabledItems, EntityId, PickupKind, Position, RulesConfig,
        SeatId, SpeedLevel, State, World, fixed_arena,
    };

    let world = World::with_pickups(
        fixed_arena(),
        BTreeMap::from([(CellId::new(5, 1), PickupKind::Heart)]),
        BTreeMap::from([(CellId::new(3, 1), PickupKind::Heart)]),
        BTreeMap::from([(CellId::new(1, 1), PickupKind::Heart)]),
    )
    .unwrap();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::One {
            CellId::new(1, 1)
        } else {
            CellId::new(13, 11)
        }),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let mut state = State::new(
        RulesConfig::expanded_v5(
            crate::room::RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART),
        ),
        world,
        actors,
    )
    .unwrap();
    state.insert_test_flame(CellId::new(3, 1), crate::game::Tick(1));
    let capture = capture_expanded(&state, &[]).unwrap();
    let coordinates: Vec<_> = capture
        .world
        .private_pickup_cells
        .iter()
        .map(|cell| (cell.row(), cell.column()))
        .collect();
    assert_eq!(coordinates, vec![(1, 1), (1, 3), (1, 5)]);
    let phases: Vec<_> = capture
        .world
        .private_pickups
        .iter()
        .map(|pickup| (pickup.cell, pickup.kind, pickup.phase))
        .collect();
    assert_eq!(
        phases,
        vec![
            (
                Cell::new(1, 1).unwrap(),
                super::PickupKind::Heart,
                super::projection_facts::PrivatePickupPhase::Exposed,
            ),
            (
                Cell::new(3, 1).unwrap(),
                super::PickupKind::Heart,
                super::projection_facts::PrivatePickupPhase::Pending,
            ),
            (
                Cell::new(5, 1).unwrap(),
                super::PickupKind::Heart,
                super::projection_facts::PrivatePickupPhase::Concealed,
            ),
        ]
    );
}

#[test]
fn damage_grace_receipt_must_not_be_in_the_future() {
    let mut state = super::tests::multiplayer_state_with_items(
        crate::game::PlayerCount::Three,
        crate::game::EnabledItems::HEART,
    );
    state.set_test_damage_grace(crate::game::SeatId::One, crate::game::Tick(1));
    assert_eq!(
        capture_expanded(&state, &[]),
        Err(ProjectionError::InvalidEffects)
    );

    state.set_test_damage_grace(crate::game::SeatId::One, crate::game::Tick(0));
    let capture = capture_expanded(&state, &[]).unwrap();
    assert_eq!(
        capture.world.actors[0].damage_grace_received_at,
        Some(crate::protocol::Tick::new(0).unwrap())
    );
    assert_eq!(
        capture.world.actors[0]
            .view
            .damage_grace_remaining_ticks
            .get(),
        60
    );
}
