use super::*;

#[test]
fn cloaked_opponents_teleport_without_resetting_or_leaking_to_observer() {
    use crate::game::{EffectKind as DomainEffect, EnabledItems, PickupKind as DomainPickup};

    let enabled = EnabledItems::from_bits(
        EnabledItems::TELEPORTER.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap();
    for seed in 0..=u8::MAX {
        let mut state = super::tests::multiplayer_state_with_items_and_seed(
            crate::game::PlayerCount::Four,
            enabled,
            [seed; 32],
        );
        state.apply_test_effect(
            crate::game::SeatId::Two,
            DomainEffect::InvisibilityCloak,
            crate::game::Tick(0),
        );
        state.apply_test_effect(
            crate::game::SeatId::Three,
            DomainEffect::InvisibilityCloak,
            crate::game::Tick(0),
        );
        let pickup_cell = state.actors()[1].position().cell().unwrap();
        state.insert_test_pickup(pickup_cell, DomainPickup::Teleporter);
        let report = state.step(&[]).unwrap();
        if state.actors()[2].last_teleport_tick().is_none() {
            continue;
        }

        let capture = capture_expanded(&state, &[report]).unwrap();
        assert_eq!(capture.world.actors[0].last_teleport_tick, None);
        assert_eq!(
            capture.world.actors[1].last_teleport_tick,
            capture.world.last_teleport_tick
        );
        assert_eq!(
            capture.world.actors[2].last_teleport_tick,
            capture.world.last_teleport_tick
        );
        assert_eq!(capture.world.actors[3].last_teleport_tick, None);

        let mut room =
            super::tests::running_room(crate::game::PlayerCount::Four, capture.world.physical_tick);
        room.settings = GameSettings::new(
            0,
            256,
            3,
            180,
            ItemSet::new([ItemId::Teleporter, ItemId::InvisibilityCloak]).unwrap(),
        )
        .unwrap();
        let generation =
            super::projection_validation_tests::generation(crate::game::PlayerCount::Four, enabled);
        let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
        let observer = project_recipient(&publication, Seat::One).unwrap();
        assert!(matches!(
            observer.state.actors[1],
            ActorView::Hidden {
                seat: Seat::Two,
                ..
            }
        ));
        assert!(matches!(
            observer.state.actors[2],
            ActorView::Hidden {
                seat: Seat::Three,
                ..
            }
        ));
        assert!(matches!(
            observer.state.actors[0],
            ActorView::Visible {
                last_teleport_tick: None,
                ..
            }
        ));
        assert!(
            !serde_json::to_string(&observer.events)
                .unwrap()
                .contains("three")
        );
        return;
    }
    panic!("seed corpus did not select the cloaked Seat Three target");
}
