use std::collections::{BTreeMap, BTreeSet};

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::inventory::{Equipment, SpecialWeapon};
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::UNITS_PER_CELL;
use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction, Pickup,
    PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, StepError, Tick,
    World,
};
use crate::room::RoomSettings;

const CENTER: CellId = CellId::new(7, 6);
const OTHER: CellId = CellId::new(10, 6);

fn arena() -> Arena {
    arena_with_blocks(BTreeSet::new())
}

fn arena_with_blocks(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn state_at(tick: Tick, shared: bool, pickup: bool) -> State {
    let exposed = if pickup {
        BTreeMap::from([(CENTER, PickupKind::Diarrhea)])
    } else {
        BTreeMap::new()
    };
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::new(SpeedLevel::Base, 256, 3, 180)
                .unwrap()
                .with_enabled_items(EnabledItems::DIARRHEA),
        ),
        World::with_pickups(arena(), BTreeMap::new(), BTreeMap::new(), exposed).unwrap(),
        [
            ActorInit {
                id: EntityId(10),
                seat: SeatId::One,
                position: Position::center(CENTER),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(20),
                seat: SeatId::Two,
                position: Position::center(if shared { CENTER } else { OTHER }),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
        tick,
    )
    .unwrap()
}

fn place(seat: SeatId) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::PlaceBomb,
    }
}

fn activate(state: &mut State, seat: SeatId, at: Tick) {
    state.actors[seat.index()]
        .effects
        .apply(EffectKind::Diarrhea, at)
        .unwrap();
}

#[test]
fn collection_starts_auto_placement_next_tick_and_expiry_stops_at_boundary() {
    let mut game = state_at(Tick(0), false, true);
    game.step(&[]).unwrap();
    assert!(game.world.bombs.is_empty());
    assert!(game.actors[0].effects.forces_bomb_drops(Tick(1)));
    game.step(&[]).unwrap();
    assert_eq!(game.world.bombs.len(), 1);

    game.world.bombs.clear();
    game.actors[0].bomb_grace.clear();
    game.tick = Tick(600);
    game.step(&[]).unwrap();
    assert!(game.world.bombs.is_empty());
    assert!(!game.actors[0].effects.forces_bomb_drops(Tick(601)));
}

#[test]
fn refresh_extends_expiry_without_adding_a_second_placement_pass() {
    let mut game = state_at(Tick(0), false, false);
    activate(&mut game, SeatId::One, Tick(0));
    game.world
        .pickups
        .exposed
        .insert(CENTER, Pickup::new(PickupKind::Diarrhea));
    game.step(&[place(SeatId::One), place(SeatId::One)])
        .unwrap();
    assert_eq!(game.world.bombs.len(), 1);
    assert!(game.actors[0].effects.forces_bomb_drops(Tick(600)));
}

#[test]
fn manual_and_auto_intents_share_conflict_resolution_for_both_seats() {
    for (one_manual, two_manual) in [(true, true), (true, false), (false, true), (false, false)] {
        let mut game = state_at(Tick(0), true, false);
        if !one_manual {
            activate(&mut game, SeatId::One, Tick(0));
        }
        if !two_manual {
            activate(&mut game, SeatId::Two, Tick(0));
        }
        let mut inputs = Vec::new();
        if one_manual {
            inputs.push(place(SeatId::One));
        }
        if two_manual {
            inputs.push(place(SeatId::Two));
        }
        game.step(&inputs).unwrap();
        assert!(game.world.bombs.is_empty());
    }

    let mut one_full = state_at(Tick(0), true, false);
    activate(&mut one_full, SeatId::One, Tick(0));
    activate(&mut one_full, SeatId::Two, Tick(0));
    add_test_bomb_with_deadline(
        &mut one_full,
        EntityId(90),
        OTHER,
        EntityId(10),
        2,
        Tick(0),
        Tick(180),
    )
    .unwrap();
    one_full.step(&[]).unwrap();
    assert!(
        one_full
            .world
            .bombs
            .values()
            .any(|bomb| bomb.owner() == EntityId(20))
    );
}

#[test]
fn clear_removes_manual_requests_but_does_not_cancel_the_curse() {
    for actions in [
        vec![
            place(SeatId::One),
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
        ],
        vec![
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            place(SeatId::One),
        ],
    ] {
        let mut game = state_at(Tick(0), false, false);
        activate(&mut game, SeatId::One, Tick(0));
        game.step(&actions).unwrap();
        assert_eq!(game.world.bombs.len(), 1);
        assert!(game.actors[0].effects.forces_bomb_drops(Tick(1)));
    }
}

#[test]
fn auto_bombs_capture_derived_stats_and_preserve_stored_items() {
    let mut game = state_at(Tick(0), false, false);
    let actor = &mut game.actors[0];
    actor.capacity = 2;
    actor.range = 4;
    actor.inventory.replace_equipment(Equipment::Kick);
    actor.inventory.replace_weapon(SpecialWeapon::Remote);
    for effect in [
        EffectKind::Diarrhea,
        EffectKind::GoldenFlame,
        EffectKind::ShortFlame,
        EffectKind::ShortFuse,
        EffectKind::LongFuse,
    ] {
        actor.effects.apply(effect, Tick(0)).unwrap();
    }
    game.step(&[]).unwrap();
    let bomb = game.world.bombs.values().next().unwrap();
    assert_eq!(bomb.range(), 1);
    assert_eq!(bomb.detonates_at(), Some(Tick(181)));
    assert_eq!(game.actors[0].inventory.equipment(), Some(Equipment::Kick));
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Remote)
    );
}

#[test]
fn auto_bombs_capture_every_fuse_and_range_modifier_combination() {
    let fuse_cases = [
        (vec![], 180),
        (vec![EffectKind::ShortFuse], 90),
        (vec![EffectKind::LongFuse], 360),
        (vec![EffectKind::ShortFuse, EffectKind::LongFuse], 180),
    ];
    let range_cases = [
        (vec![], 2),
        (vec![EffectKind::GoldenFlame], 8),
        (vec![EffectKind::ShortFlame], 1),
        (vec![EffectKind::GoldenFlame, EffectKind::ShortFlame], 1),
    ];
    for (fuse_effects, expected_fuse) in &fuse_cases {
        for (range_effects, expected_range) in &range_cases {
            let mut game = state_at(Tick(0), false, false);
            for effect in [EffectKind::Diarrhea]
                .into_iter()
                .chain(fuse_effects.iter().copied())
                .chain(range_effects.iter().copied())
            {
                game.actors[0].effects.apply(effect, Tick(0)).unwrap();
            }
            game.step(&[]).unwrap();
            let bomb = game.world.bombs.values().next().unwrap();
            assert_eq!(bomb.detonates_at(), Some(Tick(1 + expected_fuse)));
            assert_eq!(bomb.range(), *expected_range);
        }
    }
}

#[test]
fn auto_placement_declines_exhausted_id_and_deadline_without_mutation() {
    let mut no_id = state_at(Tick(0), false, false);
    activate(&mut no_id, SeatId::One, Tick(0));
    no_id.next_entity_id = u64::from(u32::MAX) + 1;
    let before_id = no_id.next_entity_id;
    no_id.step(&[]).unwrap();
    assert!(no_id.world.bombs.is_empty());
    assert_eq!(no_id.next_entity_id, before_id);

    let mut no_deadline = state_at(Tick(u64::MAX - 180), false, false);
    activate(&mut no_deadline, SeatId::One, Tick(u64::MAX - 600));
    no_deadline.step(&[]).unwrap();
    assert!(no_deadline.world.bombs.is_empty());
}

#[test]
fn movement_selects_the_automatic_bomb_cell_before_placement() {
    let mut game = state_at(Tick(0), false, false);
    game.actors[0].position.x = 8 * UNITS_PER_CELL - 40;
    game.actors[0].held.press(Direction::East);
    activate(&mut game, SeatId::One, Tick(0));
    game.step(&[]).unwrap();
    assert_eq!(
        game.world.bombs.values().next().unwrap().cell(),
        CellId::new(8, 6)
    );
}

#[test]
fn occupied_and_full_capacity_decline_without_backlog_and_reuse_next_tick() {
    let mut game = state_at(Tick(0), false, false);
    activate(&mut game, SeatId::One, Tick(0));
    game.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(90),
        CENTER,
        EntityId(20),
        2,
        Tick(0),
        Tick(2),
    )
    .unwrap();
    game.step(&[]).unwrap();
    assert_eq!(game.world.bombs.len(), 1);
    game.step(&[]).unwrap();
    assert!(game.world.bombs.is_empty());
    let reused = game.step(&[]).unwrap();
    assert!(reused.events.iter().any(|event| matches!(
        event.kind, crate::game::StepEventKind::BombPlaced(bomb) if bomb.placed_at == Tick(3)
    )));
}

#[test]
fn existing_fire_death_precedes_auto_placement_but_shield_survival_allows_it() {
    let mut dead = state_at(Tick(0), false, false);
    activate(&mut dead, SeatId::One, Tick(0));
    dead.world.hazards.insert(CENTER, Tick(2));
    dead.step(&[]).unwrap();
    assert!(!dead.actors[0].alive);
    assert!(dead.world.bombs.is_empty());
    assert!(!dead.actors[0].effects.forces_bomb_drops(Tick(1)));

    let mut protected = state_at(Tick(0), false, false);
    activate(&mut protected, SeatId::One, Tick(0));
    protected.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    protected.world.hazards.insert(CENTER, Tick(2));
    let report = protected.step(&[]).unwrap();
    assert!(protected.actors[0].alive);
    assert!(
        report
            .events
            .iter()
            .any(|event| matches!(event.kind, crate::game::StepEventKind::BombPlaced(_)))
    );
    assert!(
        report
            .events
            .iter()
            .any(|event| matches!(event.kind, crate::game::StepEventKind::BombDetonated(_)))
    );
}

#[test]
fn collection_overflow_rolls_back_earlier_auto_placement_and_input() {
    let mut game = state_at(Tick(u64::MAX - 600), false, true);
    activate(&mut game, SeatId::Two, Tick(u64::MAX - 600));
    let before = game.clone();
    assert_eq!(
        game.step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::West)
        }]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Diarrhea))
    );
    assert_eq!(game, before);
}

#[test]
fn disabled_diarrhea_is_rejected_in_every_pickup_layer() {
    for layer in 0..3 {
        let pickup = BTreeMap::from([(CENTER, PickupKind::Diarrhea)]);
        let world = World::with_pickups(
            arena_with_blocks(if layer == 0 {
                BTreeSet::from([CENTER])
            } else {
                BTreeSet::new()
            }),
            if layer == 0 {
                pickup.clone()
            } else {
                BTreeMap::new()
            },
            if layer == 1 {
                pickup.clone()
            } else {
                BTreeMap::new()
            },
            if layer == 2 { pickup } else { BTreeMap::new() },
        );
        let world = world.unwrap();
        assert!(
            State::new(
                RulesConfig::expanded_v5(RoomSettings::DEFAULT),
                world,
                [
                    ActorInit {
                        id: EntityId(1),
                        seat: SeatId::One,
                        position: Position::center(CellId::new(1, 1)),
                        heading: Direction::East,
                        speed: SpeedLevel::Base
                    },
                    ActorInit {
                        id: EntityId(2),
                        seat: SeatId::Two,
                        position: Position::center(CellId::new(13, 11)),
                        heading: Direction::West,
                        speed: SpeedLevel::Base
                    },
                ],
            )
            .is_err()
        );
    }
}
