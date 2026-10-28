use super::*;
use crate::game::SeatId;
use crate::protocol::Tick;

#[test]
fn bind_rejects_cloaked_actor_private_grace_before_recipient_filtering() {
    let mut state = tests::multiplayer_state_with_items(
        crate::game::PlayerCount::Four,
        crate::game::EnabledItems::INVISIBILITY_CLOAK,
    );
    state.apply_test_effect(
        SeatId::Two,
        crate::game::EffectKind::InvisibilityCloak,
        crate::game::Tick(0),
    );
    let capture = capture_expanded(&state, &[]).unwrap();
    let publication = projection_certification_tests::bind_result(&capture).unwrap();
    assert!(matches!(
        project_recipient(&publication, Seat::One)
            .unwrap()
            .state
            .actors[1],
        ActorView::Hidden {
            seat: Seat::Two,
            ..
        }
    ));

    let mut corrupt = capture.clone();
    corrupt.world.actors[1]
        .soft_wall_grace
        .push(crate::protocol::Cell::new(7, 5).unwrap());
    assert_eq!(
        projection_certification_tests::bind_result(&corrupt),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn damage_grace_receipt_and_remaining_ticks_are_equivalent() {
    let mut state = tests::multiplayer_state_with_items(
        crate::game::PlayerCount::Four,
        crate::game::EnabledItems::HEART,
    );
    state.set_test_damage_grace(SeatId::One, crate::game::Tick(0));
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    projection_validation_tests::assert_valid(&capture);
    assert_eq!(
        capture.world.actors[0].damage_grace_received_at,
        Some(Tick::new(0).unwrap())
    );
    assert_eq!(
        capture.world.actors[0]
            .view
            .damage_grace_remaining_ticks
            .get(),
        59
    );

    let mut future = capture.clone();
    future.world.actors[0].damage_grace_received_at = Some(Tick::new(2).unwrap());
    assert_eq!(
        projection_validation::validate(&future.world, &future.reports, &future.rules),
        Err(ProjectionError::InvalidActors)
    );

    let mut mismatched = capture.clone();
    mismatched.world.actors[0].view.damage_grace_remaining_ticks =
        DamageGraceTicks::new(58).unwrap();
    assert_eq!(
        projection_validation::validate(&mismatched.world, &mismatched.reports, &mismatched.rules),
        Err(ProjectionError::InvalidActors)
    );
}

#[test]
fn private_pickup_kind_phase_and_compatibility_layers_are_certified() {
    let mut state = tests::multiplayer_state_with_items(
        crate::game::PlayerCount::Four,
        crate::game::EnabledItems::SHIELD,
    );
    state.insert_test_pickup(
        crate::game::CellId::new(7, 5),
        crate::game::PickupKind::Shield,
    );
    let capture = capture_expanded(&state, &[]).unwrap();
    projection_validation_tests::assert_valid(&capture);

    let mut kind = capture.clone();
    kind.world.private_pickups[0].kind = PickupKind::Heart;
    kind.world.pickups[0].kind = PickupKind::Heart;
    assert_eq!(
        projection_validation::validate(&kind.world, &kind.reports, &kind.rules),
        Err(ProjectionError::InvalidItems)
    );

    let mut phase = capture.clone();
    phase.world.private_pickups[0].phase = projection_facts::PrivatePickupPhase::Pending;
    assert_eq!(
        projection_validation::validate(&phase.world, &phase.reports, &phase.rules),
        Err(ProjectionError::InvalidPickups)
    );

    let mut compatibility = capture.clone();
    compatibility.world.private_pickup_cells.clear();
    assert_eq!(
        projection_validation::validate(
            &compatibility.world,
            &compatibility.reports,
            &compatibility.rules
        ),
        Err(ProjectionError::InvalidPickups)
    );
}
