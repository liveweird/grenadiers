use crate::cpu::{BombKnowledge, WorldId};
use crate::game::{CellId, EnabledItems, PlayerCount, SeatId};

use super::*;

fn publication(count: PlayerCount) -> BoundPublication {
    let state = tests::multiplayer_state(count);
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = tests::running_room(count, capture.world.physical_tick);
    let generation = projection_validation_tests::generation(count, EnabledItems::EMPTY);
    bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap()
}

#[test]
fn cpu_adapter_exactly_matches_human_privacy_effects_and_per_actor_teleport_markers() {
    use crate::game::{
        BombTrigger, EffectKind as DomainEffect, EntityId, PickupKind as DomainPickup, Tick,
    };

    let enabled = EnabledItems::from_bits(
        EnabledItems::INVISIBILITY_CLOAK.bits()
            | EnabledItems::INVISIBLE_BOMBS.bits()
            | EnabledItems::STOPWATCH.bits()
            | EnabledItems::TELEPORTER.bits(),
    )
    .unwrap();
    let mut state =
        tests::multiplayer_state_with_items_and_seed(PlayerCount::Four, enabled, [17; 32]);
    state.apply_test_effect(SeatId::Two, DomainEffect::InvisibilityCloak, Tick(0));
    state.apply_test_effect(SeatId::One, DomainEffect::InvisibleBombs, Tick(0));
    state.apply_test_effect(SeatId::Three, DomainEffect::Stopwatch, Tick(0));
    state
        .insert_test_bomb(
            EntityId(90),
            CellId::new(7, 7),
            EntityId(11),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(180),
            },
        )
        .unwrap();
    let collector = state.actors()[3].position().cell().unwrap();
    state.insert_test_pickup(collector, DomainPickup::Teleporter);
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    assert_eq!(
        capture
            .world
            .actors
            .iter()
            .filter(|actor| actor.view.last_teleport_tick.is_some())
            .count(),
        2
    );
    let mut room = tests::running_room(PlayerCount::Four, capture.world.physical_tick);
    room.settings = GameSettings::new(
        0,
        256,
        3,
        180,
        ItemSet::new([
            ItemId::InvisibilityCloak,
            ItemId::InvisibleBombs,
            ItemId::Stopwatch,
            ItemId::Teleporter,
        ])
        .unwrap(),
    )
    .unwrap();
    let generation = projection_validation_tests::generation(PlayerCount::Four, enabled);
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();

    for recipient in [Seat::One, Seat::Two, Seat::Three, Seat::Four] {
        let human = project_recipient(&publication, recipient).unwrap().state;
        let cpu = observe_cpu_recipient(
            &publication,
            recipient,
            WorldId {
                room_instance: 9,
                physical_round: 2,
            },
        )
        .unwrap();
        assert_eq!(human.actors.len(), cpu.parts().actors.len());
        for (wire, observed) in human.actors.iter().zip(&cpu.parts().actors) {
            match (wire, observed) {
                (
                    ActorView::Hidden { id, seat, alive },
                    crate::cpu::ObservedActor::Hidden {
                        id: cpu_id,
                        seat: cpu_seat,
                        alive: cpu_alive,
                    },
                ) => {
                    assert_eq!(id.get(), cpu_id.0);
                    assert_eq!(domain_seat(*seat), *cpu_seat);
                    assert_eq!(alive, cpu_alive);
                }
                (
                    ActorView::Visible {
                        seat,
                        effects,
                        voluntary_actions_frozen,
                        last_teleport_tick,
                        ..
                    },
                    crate::cpu::ObservedActor::Visible(actor),
                ) => {
                    assert_eq!(domain_seat(*seat), actor.seat);
                    assert_eq!(*voluntary_actions_frozen, actor.voluntary_actions_frozen);
                    assert_eq!(
                        last_teleport_tick.map(|tick| tick.get()),
                        actor.last_teleport_tick.map(|tick| tick.0)
                    );
                    assert_eq!(effects.as_slice().len(), actor.effects.len());
                    for (wire_effect, cpu_effect) in effects.as_slice().iter().zip(&actor.effects) {
                        assert_eq!(
                            cpu_observation_types::domain_effect(wire_effect.kind()),
                            cpu_effect.kind
                        );
                        assert_eq!(wire_effect.remaining_ticks(), cpu_effect.remaining_ticks);
                    }
                }
                pair => panic!("CPU privacy diverged from human projection: {pair:?}"),
            }
        }
        let human_hides_bombs = matches!(
            &human.actors[recipient.index()],
            ActorView::Visible { effects, .. }
                if effects.as_slice().iter().any(|effect|
                    effect.kind() == EffectKind::InvisibleBombs)
        );
        assert_eq!(
            matches!(cpu.parts().bombs, BombKnowledge::Unavailable),
            human_hides_bombs
        );
        if let BombKnowledge::Visible(bombs) = &cpu.parts().bombs {
            assert_eq!(bombs.len(), human.bombs.len());
        }
    }
}

#[test]
fn every_multiplayer_recipient_keeps_the_canonical_roster_and_own_visible_actor() {
    for count in [PlayerCount::Three, PlayerCount::Four] {
        let publication = publication(count);
        for recipient in [Seat::One, Seat::Two, Seat::Three, Seat::Four]
            .into_iter()
            .take(count.get())
        {
            let observed = observe_cpu_recipient(
                &publication,
                recipient,
                WorldId {
                    room_instance: 9,
                    physical_round: 2,
                },
            )
            .unwrap();
            assert_eq!(observed.parts().actors.len(), count.get());
            assert!(
                observed
                    .parts()
                    .actors
                    .iter()
                    .enumerate()
                    .all(|(index, actor)| actor.seat().index() == index)
            );
            assert!(matches!(
                &observed.parts().actors[recipient.index()],
                crate::cpu::ObservedActor::Visible(actor)
                    if actor.seat == domain_seat(recipient)
            ));
            assert!(matches!(observed.parts().bombs, BombKnowledge::Visible(_)));
        }
    }
}

const fn domain_seat(seat: Seat) -> SeatId {
    match seat {
        Seat::One => SeatId::One,
        Seat::Two => SeatId::Two,
        Seat::Three => SeatId::Three,
        Seat::Four => SeatId::Four,
    }
}
