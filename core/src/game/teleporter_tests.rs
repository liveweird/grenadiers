use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, State, TurnAssist};
use crate::game::{
    BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Pickup, PickupKind,
    Position, RulesConfig, SeatId, SpeedLevel, StepError, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn actor(id: u32, seat: SeatId, position: Position) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn state_at(
    one: Position,
    two: Position,
    pickups: BTreeMap<CellId, PickupKind>,
    tick: Tick,
) -> State {
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(
                EnabledItems::from_bits(
                    EnabledItems::TELEPORTER.bits()
                        | EnabledItems::SHIELD.bits()
                        | EnabledItems::LANDMINE.bits(),
                )
                .unwrap(),
            ),
        ),
        World::with_pickups(fixed_arena(), BTreeMap::new(), BTreeMap::new(), pickups).unwrap(),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, two)],
        tick,
    )
    .unwrap()
}

fn centered(cell: CellId) -> Position {
    Position::center(cell)
}

#[test]
fn successful_swap_is_exact_atomic_and_clears_only_motion_state() {
    let one = Position { x: 1_600, y: 1_570 };
    let two_center = centered(CellId::new(12, 11));
    let two = Position {
        x: two_center.x - 20,
        y: two_center.y - 18,
    };
    let mut game = state_at(
        one,
        two,
        BTreeMap::from([(one.cell().unwrap(), PickupKind::Teleporter)]),
        Tick(0),
    );
    game.actors[0].residual = 37;
    game.actors[1].residual = 42;
    game.actors[0].pending_turn = Some(TurnAssist {
        direction: Direction::South,
        target: 1_536,
    });
    game.actors[1].pending_turn = Some(TurnAssist {
        direction: Direction::North,
        target: 10_752,
    });

    game.step(&[]).unwrap();

    assert_eq!(
        (game.actors[0].position, game.actors[1].position),
        (two, one)
    );
    assert_eq!(
        (
            game.actors[0].residual,
            game.actors[1].residual,
            game.actors[0].pending_turn,
            game.actors[1].pending_turn,
        ),
        (0, 0, None, None)
    );
    assert_eq!(game.last_teleport_tick, Some(Tick(1)));
    assert!(game.world.pickups.exposed.is_empty());
}

#[test]
fn unsafe_contact_solid_and_fire_destinations_consume_without_partial_swap() {
    for hazard in ["contact", "solid", "fire"] {
        let one = centered(CellId::new(1, 1));
        let two = centered(CellId::new(13, 11));
        let mut game = state_at(
            one,
            two,
            BTreeMap::from([(CellId::new(1, 1), PickupKind::Teleporter)]),
            Tick(0),
        );
        match hazard {
            "contact" => game
                .insert_test_bomb(
                    EntityId(30),
                    CellId::new(13, 11),
                    EntityId(10),
                    2,
                    Tick(0),
                    BombTrigger::Contact,
                )
                .unwrap(),
            "solid" => game
                .insert_test_bomb(
                    EntityId(30),
                    CellId::new(13, 11),
                    EntityId(10),
                    2,
                    Tick(0),
                    BombTrigger::Timed {
                        detonates_at: Tick(100),
                    },
                )
                .unwrap(),
            "fire" => {
                game.apply_test_effect(SeatId::Two, EffectKind::Shield, Tick(0));
                game.insert_test_flame(CellId::new(13, 11), Tick(100));
            }
            _ => unreachable!(),
        }

        game.step(&[]).unwrap();

        assert_eq!(
            (game.actors[0].position, game.actors[1].position),
            (one, two)
        );
        assert_eq!(game.last_teleport_tick, None);
        assert!(game.world.pickups.exposed.is_empty());
    }
}

#[test]
fn stale_or_other_actor_grace_cannot_authorize_a_swap() {
    let one = centered(CellId::new(1, 1));
    let two = centered(CellId::new(13, 11));
    let mut game = state_at(
        one,
        two,
        BTreeMap::from([(CellId::new(1, 1), PickupKind::Teleporter)]),
        Tick(0),
    );
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(13, 11),
        EntityId(10),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    game.actors[0].bomb_grace.insert(EntityId(30));
    game.actors[0].bomb_grace.insert(EntityId(999));
    let before = game
        .actors
        .iter()
        .map(|actor| actor.position)
        .collect::<Vec<_>>();

    game.step(&[]).unwrap();

    assert_eq!(
        game.actors
            .iter()
            .map(|actor| actor.position)
            .collect::<Vec<_>>(),
        before
    );
    assert_eq!(game.last_teleport_tick, None);
    assert!(game.world.pickups.exposed.is_empty());
}

#[test]
fn valid_source_and_destination_grace_is_preserved_per_actor() {
    let center = centered(CellId::new(5, 5));
    let one = Position {
        x: center.x - 80,
        y: center.y,
    };
    let two = Position {
        x: center.x + 80,
        y: center.y,
    };
    for trigger in [
        BombTrigger::Contact,
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    ] {
        let mut game = state_at(
            one,
            two,
            BTreeMap::from([(CellId::new(5, 5), PickupKind::Teleporter)]),
            Tick(0),
        );
        game.insert_test_bomb(
            EntityId(30),
            CellId::new(5, 5),
            EntityId(10),
            2,
            Tick(0),
            trigger,
        )
        .unwrap();
        assert_eq!(
            game.actors
                .iter()
                .map(|actor| actor.bomb_grace.clone())
                .collect::<Vec<_>>(),
            [
                BTreeSet::from([EntityId(30)]),
                BTreeSet::from([EntityId(30)])
            ]
        );

        game.step(&[]).unwrap();

        assert_eq!(
            game.actors
                .iter()
                .map(|actor| actor.position)
                .collect::<Vec<_>>(),
            [two, one]
        );
        assert_eq!(
            game.actors
                .iter()
                .map(|actor| actor.bomb_grace.clone())
                .collect::<Vec<_>>(),
            [
                BTreeSet::from([EntityId(30)]),
                BTreeSet::from([EntityId(30)])
            ]
        );
    }
}

#[test]
fn later_timed_pickup_overflow_rolls_back_consumption_and_marker() {
    let one_cell = CellId::new(1, 1);
    let two_cell = CellId::new(13, 11);
    let mut game = state_at(
        centered(one_cell),
        centered(two_cell),
        BTreeMap::from([
            (one_cell, PickupKind::Teleporter),
            (two_cell, PickupKind::Shield),
        ]),
        Tick(u64::MAX - EffectKind::Shield.duration_ticks()),
    );
    let before = game.clone();

    assert_eq!(
        game.step(&[]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(game, before);
}

#[test]
fn simultaneous_requests_swap_once_while_dead_and_equal_targets_are_noops() {
    let one_cell = CellId::new(1, 1);
    let two_cell = CellId::new(13, 11);
    let one = centered(one_cell);
    let two = centered(two_cell);
    let mut simultaneous = state_at(
        one,
        two,
        BTreeMap::from([
            (one_cell, PickupKind::Teleporter),
            (two_cell, PickupKind::Teleporter),
        ]),
        Tick(0),
    );
    simultaneous.step(&[]).unwrap();
    assert_eq!(
        (
            simultaneous.actors[0].position,
            simultaneous.actors[1].position
        ),
        (two, one)
    );
    assert_eq!(simultaneous.last_teleport_tick, Some(Tick(1)));
    assert!(simultaneous.world.pickups.exposed.is_empty());

    let mut dead_target = state_at(
        one,
        two,
        BTreeMap::from([(one_cell, PickupKind::Teleporter)]),
        Tick(0),
    );
    dead_target.actors[1].alive = false;
    dead_target.step(&[]).unwrap();
    assert_eq!(dead_target.actors[0].position, one);
    assert_eq!(dead_target.last_teleport_tick, None);

    let mut equal = state_at(
        one,
        two,
        BTreeMap::from([(one_cell, PickupKind::Teleporter)]),
        Tick(0),
    );
    equal.actors[1].position = one;
    equal.step(&[]).unwrap();
    assert_eq!(equal.actors[0].position, one);
    assert_eq!(equal.last_teleport_tick, None);
}

#[test]
fn arena_obstacles_reject_atomically_and_arrival_pickups_are_not_rescanned() {
    let one = centered(CellId::new(1, 1));
    let safe = centered(CellId::new(13, 11));
    let mut blocked = state_at(one, safe, BTreeMap::new(), Tick(0));
    for destination in [centered(CellId::new(0, 0)), centered(CellId::new(2, 2))] {
        blocked.actors[1].position = destination;
        blocked.commit_teleport([true, false], Tick(1));
        assert_eq!(blocked.actors[0].position, one);
        assert_eq!(blocked.actors[1].position, destination);
        assert_eq!(blocked.last_teleport_tick, None);
    }

    let mut no_rescan = state_at(one, safe, BTreeMap::new(), Tick(0));
    no_rescan
        .world
        .pickups
        .exposed
        .insert(CellId::new(13, 11), Pickup::new(PickupKind::BombCapacity));
    no_rescan.commit_teleport([true, false], Tick(1));
    assert_eq!(no_rescan.actors[0].position, safe);
    assert!(
        no_rescan
            .world
            .pickups
            .exposed
            .contains_key(&CellId::new(13, 11))
    );
}
