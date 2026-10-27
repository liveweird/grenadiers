use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::{BombKnowledge, ObservedActor, ObservedBombTrigger, UnknownActorFacts, WorldId};
use crate::game::{
    self, ActorInit, BombTrigger, CellId, EffectKind, EnabledItems, EntityId, OilState, Position,
    RulesConfig, SeatId, SpeedLevel, State, Tick, World,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::*;

pub(super) const WORLD: WorldId = WorldId {
    room_instance: 77,
    physical_round: 9,
};

pub(super) fn enabled(items: &[ItemId]) -> EnabledItems {
    let bits = ItemSet::new(items.iter().copied()).unwrap().mask().bits();
    EnabledItems::from_bits(bits).unwrap()
}

fn actors(world: &World, speed: SpeedLevel) -> [ActorInit; 2] {
    let spawns = world.arena().spawns();
    SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: game::Direction::East,
        speed,
    })
}

pub(super) fn state(settings: RoomSettings, world: World) -> State {
    State::new(
        RulesConfig::expanded_v5(settings),
        world.clone(),
        actors(&world, settings.starting_speed()),
    )
    .unwrap()
}

fn publication(state: &State, settings: RoomSettings) -> BoundPublication {
    let capture = capture_expanded(state, &[]).unwrap();
    bind_capture(&capture, settings)
}

pub(super) fn bind_capture(capture: &ExpandedCapture, settings: RoomSettings) -> BoundPublication {
    let wire_items = ItemSet::from_mask(ItemMask::new(settings.enabled_items().bits()).unwrap());
    let room = RoomView {
        settings: GameSettings::new(
            speed_index(settings.starting_speed()),
            settings.turn_assist_units(),
            settings.wins_to_finish(),
            u16::try_from(settings.bomb_fuse_ticks()).unwrap(),
            wire_items,
        )
        .unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: expanded_items(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(3).unwrap(),
        revision: Counter::new(4).unwrap(),
        tick: capture.world.0.physical_tick,
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: true,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Connected,
        }),
        phase: RoomPhase::Running,
    };
    bind_expanded(
        &room,
        room.round_id,
        capture,
        Some(game::generate_expanded_arena([31; 32], settings.enabled_items()).record()),
    )
    .unwrap()
}

const fn speed_index(speed: SpeedLevel) -> u8 {
    match speed {
        SpeedLevel::Base => 0,
        SpeedLevel::One => 1,
        SpeedLevel::Two => 2,
        SpeedLevel::Three => 3,
        SpeedLevel::Four => 4,
    }
}

pub(super) fn observe(
    state: &State,
    settings: RoomSettings,
    seat: Seat,
) -> crate::cpu::Observation {
    observe_cpu_recipient(&publication(state, settings), seat, WORLD).unwrap()
}

#[test]
fn recipient_projection_preserves_privacy_and_distinguishes_unavailable_bombs() {
    let enabled = enabled(&[
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::OilBarrel,
        ItemId::Shield,
        ItemId::Heart,
        ItemId::Stopwatch,
    ]);
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let arena = game::test_support::open_border_arena(BTreeSet::from([
        CellId::new(2, 2),
        CellId::new(5, 2),
    ]));
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::from([(CellId::new(2, 2), game::PickupKind::Shield)]),
        BTreeMap::from([(CellId::new(3, 2), game::PickupKind::Stopwatch)]),
        BTreeMap::from([(CellId::new(4, 2), game::PickupKind::BlastRange)]),
        BTreeMap::from([
            (CellId::new(5, 2), OilState::Concealed),
            (CellId::new(6, 2), OilState::Pending),
            (CellId::new(7, 2), OilState::Unlit),
            (CellId::new(8, 2), OilState::Burning),
        ]),
    )
    .unwrap();
    let mut state = state(settings, world);
    state.apply_test_effect(SeatId::One, EffectKind::InvisibleBombs, Tick(0));
    state.apply_test_effect(SeatId::One, EffectKind::InvisibilityCloak, Tick(0));
    state.apply_test_effect(SeatId::Two, EffectKind::InvisibilityCloak, Tick(0));
    state
        .insert_test_bomb(
            EntityId(20),
            CellId::new(4, 3),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(180),
            },
        )
        .unwrap();

    let one = observe(&state, settings, Seat::One);
    let two = observe(&state, settings, Seat::Two);
    assert_eq!(one.parts().world, WORLD);
    assert_eq!(one.parts().bombs, BombKnowledge::Unavailable);
    assert!(matches!(two.parts().bombs, BombKnowledge::Visible(ref bombs) if bombs.len() == 1));
    assert!(matches!(
        one.parts().actors[1],
        ObservedActor::Hidden {
            seat: SeatId::Two,
            ..
        }
    ));
    assert!(matches!(
        two.parts().actors[0],
        ObservedActor::Hidden {
            seat: SeatId::One,
            ..
        }
    ));
    assert_eq!(one.parts().pickups.len(), 1);
    assert_eq!(one.parts().pickups[0].cell, CellId::new(4, 2));
    assert_eq!(one.parts().oil.len(), 2);
    assert!(one.parts().oil.iter().all(|oil| matches!(
        oil.state,
        game::OilBarrelState::Unlit | game::OilBarrelState::Burning
    )));
}

#[test]
fn visible_bomb_trigger_variants_are_exact_recipient_facts() {
    let cases = [
        (
            BombTrigger::Timed {
                detonates_at: Tick(180),
            },
            ObservedBombTrigger::Timed {
                remaining_ticks: 180,
                paused: false,
            },
            Vec::new(),
        ),
        (
            BombTrigger::Remote,
            ObservedBombTrigger::Remote,
            vec![ItemId::RemoteBomb],
        ),
        (
            BombTrigger::Contact,
            ObservedBombTrigger::Contact,
            vec![ItemId::Landmine],
        ),
    ];
    for (trigger, expected, items) in cases {
        let settings = RoomSettings::DEFAULT.with_enabled_items(enabled(&items));
        let world = World::new(game::test_support::open_border_arena(BTreeSet::new()));
        let mut state = state(settings, world);
        state
            .insert_test_bomb(
                EntityId(20),
                CellId::new(4, 3),
                EntityId(1),
                2,
                Tick(0),
                trigger,
            )
            .unwrap();
        let observation = observe(&state, settings, Seat::Two);
        let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
            panic!("bomb layer must be available")
        };
        assert_eq!(bombs[0].trigger, expected);
        assert_eq!(bombs[0].placed_at, Tick(0));
    }

    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled(&[ItemId::Stopwatch]));
    let world = World::new(game::test_support::open_border_arena(BTreeSet::new()));
    let mut state = state(settings, world);
    state.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
    state
        .insert_test_bomb(
            EntityId(20),
            CellId::new(4, 3),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(180),
            },
        )
        .unwrap();
    state.step(&[]).unwrap();
    let observation = observe(&state, settings, Seat::Two);
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        panic!("bomb layer must be available")
    };
    assert_eq!(
        bombs[0].trigger,
        ObservedBombTrigger::Timed {
            remaining_ticks: 180,
            paused: true,
        }
    );
}

#[test]
fn bomb_placement_tick_remains_exact_across_later_publications() {
    let settings = RoomSettings::DEFAULT;
    let world = World::new(game::test_support::open_border_arena(BTreeSet::new()));
    let mut state = state(settings, world);
    state.step(&[]).unwrap();
    state
        .step(&[game::SeatInput {
            seat: SeatId::Two,
            action: game::InputAction::PlaceBomb,
        }])
        .unwrap();
    let placed_at = state.tick();
    for _ in 0..3 {
        let observation = observe(&state, settings, Seat::Two);
        let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
            panic!("bomb layer must be available")
        };
        assert_eq!(bombs.len(), 1);
        assert_eq!(bombs[0].placed_at, placed_at);
        assert_eq!(bombs[0].id, state.world().bombs().values().next().unwrap().id());
        state.step(&[]).unwrap();
    }
    assert_eq!(placed_at, Tick(2));
}

#[test]
fn frozen_teleport_settings_flames_and_unknown_actor_state_are_projected() {
    let enabled = enabled(&[
        ItemId::Stopwatch,
        ItemId::Teleporter,
        ItemId::Kick,
        ItemId::RemoteBomb,
    ]);
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 9, 300)
        .unwrap()
        .with_enabled_items(enabled);
    let spawn = game::test_support::open_border_arena(BTreeSet::new()).spawns()[0];
    let world = World::with_pickups(
        game::test_support::open_border_arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawn, game::PickupKind::Teleporter)]),
    )
    .unwrap();
    let mut state = state(settings, world);
    state.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
    state.step(&[]).unwrap();
    state.insert_test_flame(CellId::new(4, 3), Tick(7));
    let mut capture = capture_expanded(&state, &[]).unwrap();
    capture.world.0.actors[0].inventory = InventoryView {
        equipment: Some(EquipmentId::Kick),
        weapon: Some(WeaponId::RemoteBomb),
    };
    let observation =
        observe_cpu_recipient(&bind_capture(&capture, settings), Seat::One, WORLD).unwrap();
    let ObservedActor::Visible(actor) = &observation.parts().actors[0] else {
        panic!("recipient must be visible")
    };
    assert!(actor.voluntary_actions_frozen);
    assert_eq!(actor.last_teleport_tick, Some(Tick(1)));
    assert_eq!(actor.unknown, UnknownActorFacts::ALL);
    assert_eq!(actor.equipment, Some(game::Equipment::Kick));
    assert_eq!(actor.weapon, Some(game::SpecialWeapon::Remote));
    let ObservedActor::Visible(other) = &observation.parts().actors[1] else {
        panic!("uncloaked rival must be visible")
    };
    assert_eq!(other.effects.len(), 1);
    assert_eq!(other.effects[0].kind, EffectKind::Stopwatch);
    assert_eq!(other.effects[0].remaining_ticks, 179);
    assert_eq!(observation.parts().rules.settings, settings);
    assert_eq!(observation.parts().rules.version, 5);
    assert_eq!(observation.parts().rules.tick_rate, 60);
    assert_eq!(observation.parts().flames[0].remaining_ticks, 6);
}
