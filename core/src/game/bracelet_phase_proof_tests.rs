use std::collections::{BTreeMap, BTreeSet};

use super::bracelet_test_support::{arena, input, press, simple_state, state};
use super::{Direction, InputAction, Position, SeatId};
use crate::game::{
    Bomb, BombTrigger, CellId, DamagePreventionSource, EffectKind, EnabledItems, EntityId,
    Equipment, PickupKind, StepEventKind, Tick, World,
};

fn enabled(items: &[EnabledItems]) -> EnabledItems {
    EnabledItems::from_bits(items.iter().fold(0, |bits, item| bits | item.bits())).unwrap()
}

fn contact_world(cell: CellId) -> World {
    let mut world = World::new(arena(BTreeSet::new()));
    world.bombs.insert(
        EntityId(30),
        Bomb::new(
            EntityId(30),
            cell,
            EntityId(2),
            1,
            Tick(0),
            BombTrigger::Contact,
        )
        .unwrap(),
    );
    world
}

#[test]
fn voluntary_grace_exit_then_forced_reentry_accepts_contact_before_old_fire_death() {
    let mine = CellId::new(5, 4);
    let mut world = contact_world(mine);
    world.hazards.insert(mine, Tick(2));
    let mut game = state(
        enabled(&[EnabledItems::BRACELET, EnabledItems::LANDMINE]),
        world,
        Position { x: 4_211, y: 4_096 },
        Position { x: 4_851, y: 4_096 },
        Tick(0),
    );
    assert!(game.actors[1].bomb_grace.contains(&EntityId(30)));

    let report = game
        .step(&[
            press(SeatId::One, Direction::East),
            press(SeatId::Two, Direction::West),
        ])
        .unwrap();

    assert_eq!(report.forced_movements.into_iter().next().unwrap().segment.start().x, 4_800);
    assert!(!game.actors[1].alive);
    assert!(!game.world.bombs.contains_key(&EntityId(30)));
    let eliminated = report
        .events
        .iter()
        .position(|event| {
            matches!(
                event.kind,
                StepEventKind::ActorEliminated {
                    id: EntityId(2),
                    seat: SeatId::Two,
                }
            )
        })
        .expect("the forced path through old fire eliminates the victim");
    let detonated = report
        .events
        .iter()
        .position(|event| {
            matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
        })
        .expect("contact accepted before death remains in the later explosion plan");
    assert!(eliminated < detonated);
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30)))
            .count(),
        1
    );
}

#[test]
fn old_fire_reached_only_by_forced_segment_uses_shield_once_before_heart() {
    let fire = CellId::new(5, 4);
    let mut game = simple_state(
        enabled(&[
            EnabledItems::BRACELET,
            EnabledItems::SHIELD,
            EnabledItems::HEART,
        ]),
        Position { x: 4_096, y: 4_096 },
        Position { x: 4_736, y: 4_096 },
    );
    game.insert_test_flame(fire, Tick(2));
    game.replace_actor_equipment(SeatId::Two, Equipment::Heart);
    game.apply_test_effect(SeatId::Two, EffectKind::Shield, Tick(0));

    let report = game
        .step(&[
            press(SeatId::One, Direction::East),
            input(SeatId::Two, InputAction::PlaceBomb),
        ])
        .unwrap();

    assert!(report.traces[1].is_empty());
    assert!(report.forced_movements.into_iter().next().unwrap().segment.overlaps_cell(fire));
    assert!(game.actors[1].alive);
    assert_eq!(game.actors[1].inventory.equipment(), Some(Equipment::Heart));
    let shield_prevention = report
        .events
        .iter()
        .position(|event| {
            matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    seat: SeatId::Two,
                    source: DamagePreventionSource::Shield,
                    ..
                }
            )
        })
        .expect("the forced path reaches old fire before manual actions");
    let victim_placement = report
        .events
        .iter()
        .position(|event| {
            matches!(
                event.kind,
                StepEventKind::BombPlaced(view)
                    if view.owner == EntityId(2) && view.cell == CellId::new(5, 4)
            )
        })
        .expect("shield survival preserves the queued post-shove placement");
    assert!(shield_prevention < victim_placement);
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    seat: SeatId::Two,
                    source: DamagePreventionSource::Shield,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(!report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::DamagePrevented {
            source: DamagePreventionSource::Heart,
            ..
        }
    )));
}

#[test]
fn victim_manual_actions_resolve_once_from_post_shove_cell() {
    let mut game = simple_state(
        enabled(&[EnabledItems::BRACELET, EnabledItems::PUNCH]),
        Position { x: 4_096, y: 4_096 },
        Position { x: 4_736, y: 4_096 },
    );
    game.actors[1].heading = Direction::East;
    game.replace_actor_equipment(SeatId::Two, Equipment::Punch);
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(6, 4),
        EntityId(1),
        1,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();

    let report = game
        .step(&[
            press(SeatId::One, Direction::East),
            input(SeatId::Two, InputAction::PlaceBomb),
            input(SeatId::Two, InputAction::UseEquipment),
        ])
        .unwrap();

    assert_eq!(report.forced_movements.into_iter().next().unwrap().segment.end().x, 5_760);
    let placed: Vec<_> = report
        .events
        .iter()
        .filter_map(|event| match event.kind {
            StepEventKind::BombPlaced(view) if view.owner == EntityId(2) => Some(view),
            _ => None,
        })
        .collect();
    assert_eq!(placed.len(), 1);
    assert_eq!(placed[0].cell, CellId::new(5, 4));
    let punched = game.world.bombs.get(&EntityId(30)).unwrap();
    assert!(punched.has_been_punched());
    assert_ne!(punched.cell(), CellId::new(6, 4));
    assert_eq!(game.actors[1].last_successful_punch_tick(), Some(Tick(1)));
}

#[test]
fn endpoint_pickup_teleports_state_but_report_keeps_preteleport_forced_segment() {
    let pickup = CellId::new(5, 4);
    let world = World::with_pickups(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup, PickupKind::Teleporter)]),
    )
    .unwrap();
    let mut game = state(
        enabled(&[EnabledItems::BRACELET, EnabledItems::TELEPORTER]),
        world,
        Position { x: 4_096, y: 4_096 },
        Position { x: 4_736, y: 4_096 },
        Tick(0),
    );

    let report = game.step(&[press(SeatId::One, Direction::East)]).unwrap();
    let forced = report.forced_movements.into_iter().next().unwrap();

    assert_eq!(forced.segment.start(), Position { x: 4_736, y: 4_096 });
    assert_eq!(forced.segment.end(), Position { x: 5_760, y: 4_096 });
    assert_eq!(game.actors[0].position, forced.segment.end());
    assert_eq!(game.actors[1].position, Position { x: 4_147, y: 4_096 });
    assert_eq!(game.last_teleport_tick, Some(Tick(1)));
}
