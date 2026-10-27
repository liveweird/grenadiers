use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use super::{ActorInit, InputAction, SeatInput, State, StateError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::test_support::{add_test_bomb_with_deadline, add_test_bomb_with_trigger};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{
    BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, EventAudience,
    PickupKind, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId, SpecialWeapon,
    SpeedLevel, StepEventKind, Tick, World,
};
use crate::room::RoomSettings;

const CORNERS: [CellId; 4] = [
    CellId::new(1, 1),
    CellId::new(13, 1),
    CellId::new(1, 11),
    CellId::new(13, 11),
];

fn arena(count: PlayerCount) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, BTreeSet::new(), CORNERS[..count.get()].to_vec()).unwrap()
}

fn actors(count: PlayerCount) -> Vec<ActorInit> {
    count
        .seats()
        .iter()
        .map(|&seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(CORNERS[seat.index()]),
            heading: if matches!(seat, SeatId::One | SeatId::Three) {
                Direction::East
            } else {
                Direction::West
            },
            speed: SpeedLevel::Base,
        })
        .collect()
}

fn rules(enabled: EnabledItems) -> RulesConfig {
    RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

fn state(count: PlayerCount, enabled: EnabledItems) -> State {
    State::new(rules(enabled), World::new(arena(count)), actors(count)).unwrap()
}

fn supported_items() -> [EnabledItems; 21] {
    [
        EnabledItems::KICK,
        EnabledItems::WALL_PASS,
        EnabledItems::HEART,
        EnabledItems::REMOTE_BOMB,
        EnabledItems::PIERCING_BOMB,
        EnabledItems::POWER_BOMB,
        EnabledItems::LANDMINE,
        EnabledItems::SHIELD,
        EnabledItems::STOPWATCH,
        EnabledItems::INVISIBILITY_CLOAK,
        EnabledItems::GOLDEN_FLAME,
        EnabledItems::SHORT_FUSE,
        EnabledItems::DIARRHEA,
        EnabledItems::SLOW_FOOT,
        EnabledItems::SHORT_FLAME,
        EnabledItems::LONG_FUSE,
        EnabledItems::REVERSE_CONTROLS,
        EnabledItems::INVISIBLE_BOMBS,
        EnabledItems::OIL_BARREL,
        EnabledItems::PUNCH,
        EnabledItems::BRACELET,
    ]
}

fn supported_mask() -> EnabledItems {
    EnabledItems::from_bits(
        supported_items()
            .into_iter()
            .fold(0, |bits, item| bits | item.bits()),
    )
    .unwrap()
}

#[test]
fn every_supported_item_and_the_combined_mask_construct_and_step_for_three_and_four_players() {
    let unsupported = EnabledItems::TELEPORTER.bits();
    assert_eq!(
        supported_mask().bits() | unsupported,
        EnabledItems::ALL.bits()
    );
    assert_eq!(supported_mask().bits() & unsupported, 0);
    for count in [PlayerCount::Three, PlayerCount::Four] {
        for enabled in supported_items() {
            let mut game = state(count, enabled);
            assert_eq!(game.step(&[]).unwrap().traces.len(), count.get());
        }
        let mut combined = state(count, supported_mask());
        assert_eq!(combined.step(&[]).unwrap().traces.len(), count.get());
    }
}

#[test]
fn punch_and_bracelet_are_multiplayer_enabled_while_teleporter_remains_blocked() {
    for count in [PlayerCount::Three, PlayerCount::Four] {
        for enabled in [EnabledItems::PUNCH, EnabledItems::BRACELET] {
            let mut game = state(count, enabled);
            assert_eq!(game.step(&[]).unwrap().traces.len(), count.get());
        }
        assert_eq!(
            State::new(
                rules(EnabledItems::TELEPORTER),
                World::new(arena(count)),
                actors(count)
            ),
            Err(StateError::UnsupportedEnabledItems(
                EnabledItems::TELEPORTER
            ))
        );
    }
}

#[test]
fn simultaneous_stopwatch_holders_freeze_each_other_and_pause_timed_bombs_once() {
    let mut game = state(PlayerCount::Four, EnabledItems::STOPWATCH);
    game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));
    game.apply_test_effect(SeatId::Three, EffectKind::Stopwatch, Tick(0));
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(90),
        CellId::new(7, 5),
        EntityId(11),
        2,
        Tick(0),
        Tick(10),
    )
    .unwrap();

    game.step(&[]).unwrap();

    assert_eq!(game.voluntary_actions_frozen(), [true; 4]);
    assert_eq!(
        game.world.bombs()[&EntityId(90)].detonates_at(),
        Some(Tick(11))
    );
    assert_eq!(
        game.world.bombs()[&EntityId(90)]
            .timed_history()
            .unwrap()
            .paused_ticks,
        1
    );
}

#[test]
fn three_way_pickup_and_bomb_contention_follow_multiplayer_priority_rules() {
    let contested = CellId::new(7, 5);
    let world = World::with_pickups(
        arena(PlayerCount::Four),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(contested, PickupKind::BombCapacity)]),
    )
    .unwrap();
    let mut actor_inits = actors(PlayerCount::Four);
    for seat in [SeatId::One, SeatId::Three, SeatId::Four] {
        actor_inits[seat.index()].position = Position::center(contested);
    }
    let assignment = RoundAssignment::new(NonZeroU32::new(2).unwrap());
    let mut pickup_game = State::new_at_tick_with_assignment(
        rules(EnabledItems::EMPTY),
        world,
        actor_inits,
        Tick(0),
        assignment,
    )
    .unwrap();

    pickup_game.step(&[]).unwrap();
    assert_eq!(pickup_game.actor(SeatId::Three).unwrap().bomb_capacity(), 2);
    assert_eq!(pickup_game.actor(SeatId::One).unwrap().bomb_capacity(), 1);
    assert_eq!(pickup_game.actor(SeatId::Four).unwrap().bomb_capacity(), 1);

    let mut bomb_game = state(PlayerCount::Four, EnabledItems::EMPTY);
    for seat in [SeatId::One, SeatId::Three, SeatId::Four] {
        bomb_game.actors[seat.index()].position = Position::center(contested);
    }
    bomb_game.actors[SeatId::Two.index()].position = Position::center(CellId::new(9, 5));
    bomb_game
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::Three,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::Four,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    assert_eq!(bomb_game.world.bombs().len(), 1);
    assert_eq!(
        bomb_game.world.bombs().values().next().unwrap().owner(),
        EntityId(11)
    );
}

#[test]
fn remote_bombs_are_independent_for_multiple_owners() {
    let world = World::with_pickups(
        arena(PlayerCount::Four),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (CORNERS[0], PickupKind::RemoteBomb),
            (CORNERS[2], PickupKind::RemoteBomb),
        ]),
    )
    .unwrap();
    let mut game = State::new(
        rules(EnabledItems::REMOTE_BOMB),
        world,
        actors(PlayerCount::Four),
    )
    .unwrap();
    game.step(&[]).unwrap();
    assert_eq!(
        game.actor(SeatId::One).unwrap().inventory().weapon(),
        Some(SpecialWeapon::Remote)
    );
    assert_eq!(
        game.actor(SeatId::Three).unwrap().inventory().weapon(),
        Some(SpecialWeapon::Remote)
    );
    let inputs = [SeatId::One, SeatId::Three].map(|seat| SeatInput {
        seat,
        action: InputAction::PlaceBomb,
    });
    game.step(&inputs).unwrap();
    assert_eq!(game.world.bombs().len(), 2);

    let report = game.step(&inputs).unwrap();
    assert!(game.world.bombs().is_empty());
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
            .count(),
        2
    );
}

#[test]
fn swept_contact_mine_detects_any_multiplayer_actor_trace() {
    let mut game = state(PlayerCount::Four, EnabledItems::LANDMINE);
    let mine = CellId::new(5, 5);
    game.actors[0].position = Position {
        x: i32::from(mine.column) * UNITS_PER_CELL - HALF_FOOTPRINT - 1,
        y: Position::center(mine).y,
    };
    game.actors[0].heading = Direction::East;
    add_test_bomb_with_trigger(
        &mut game,
        EntityId(90),
        mine,
        EntityId(11),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();

    let report = game
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::BombDetonated(bomb) if bomb.id == EntityId(90)
    )));
}

#[test]
fn cloak_and_bomb_blindness_keep_four_player_event_audiences_private() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::INVISIBILITY_CLOAK.bits() | EnabledItems::INVISIBLE_BOMBS.bits(),
    )
    .unwrap();
    let mut game = state(PlayerCount::Four, enabled);
    game.apply_test_effect(SeatId::One, EffectKind::InvisibilityCloak, Tick(0));
    game.apply_test_effect(SeatId::Four, EffectKind::InvisibleBombs, Tick(0));
    for (id, cell, owner) in [
        (EntityId(90), CellId::new(5, 5), EntityId(10)),
        (EntityId(91), CellId::new(9, 5), EntityId(11)),
    ] {
        add_test_bomb_with_deadline(&mut game, id, cell, owner, 1, Tick(0), Tick(1)).unwrap();
    }

    let report = game.step(&[]).unwrap();
    let audience = |id| {
        report
            .events
            .iter()
            .find_map(|event| match event.kind {
                StepEventKind::BombDetonated(bomb) if bomb.id == id => Some(event.audience),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(audience(EntityId(90)), EventAudience::only(SeatId::One));
    let visible = audience(EntityId(91));
    assert!(visible.includes(SeatId::One));
    assert!(visible.includes(SeatId::Two));
    assert!(visible.includes(SeatId::Three));
    assert!(!visible.includes(SeatId::Four));
}

#[test]
fn damage_resolution_visits_all_actors_with_independent_prevention() {
    let enabled = EnabledItems::HEART_AND_SHIELD;
    let mut game = state(PlayerCount::Four, enabled);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    game.apply_test_effect(SeatId::Three, EffectKind::Shield, Tick(0));
    let cells: Vec<_> = game
        .actors
        .iter()
        .map(|actor| actor.position.cell().unwrap())
        .collect();
    for cell in cells {
        game.insert_test_flame(cell, Tick(10));
    }

    let report = game.step(&[]).unwrap();
    assert!(game.actor(SeatId::One).unwrap().alive());
    assert!(!game.actor(SeatId::Two).unwrap().alive());
    assert!(game.actor(SeatId::Three).unwrap().alive());
    assert!(!game.actor(SeatId::Four).unwrap().alive());
    for seat in SeatId::ALL {
        assert!(report.events.iter().any(|event| match event.kind {
            StepEventKind::DamagePrevented {
                seat: event_seat, ..
            }
            | StepEventKind::ActorEliminated {
                seat: event_seat, ..
            } => event_seat == seat,
            _ => false,
        }));
    }
}
