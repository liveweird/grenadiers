use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, BlastKind, Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId,
    Equipment, InputAction, OilState, PickupKind, Position, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, Tick, World, fixed_arena,
};

fn rules(extra: EnabledItems) -> RulesConfig {
    let bits = EnabledItems::OIL_BARREL.bits() | extra.bits();
    let enabled = EnabledItems::from_bits(bits).unwrap();
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn oil_world(cell: CellId, phase: OilState) -> World {
    World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cell, phase)]),
    )
    .unwrap()
}

#[test]
fn pending_oil_reveals_once_safe_and_never_becomes_a_pickup() {
    let cell = CellId::new(3, 1);
    let mut world = oil_world(cell, OilState::Pending);
    world.hazards.insert(cell, Tick(2));
    let mut safe_actors = actors();
    safe_actors[0].position = Position::center(CellId::new(1, 2));
    let mut state = State::new(rules(EnabledItems::EMPTY), world, safe_actors).unwrap();

    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(cell), Some(OilState::Pending));
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(cell), Some(OilState::Unlit));
    assert!(state.world.exposed_pickups().is_empty());
    assert!(
        state
            .public_view()
            .oil_barrels
            .iter()
            .any(|barrel| barrel.cell == cell)
    );
}

#[test]
fn living_footprint_and_any_bomb_delay_revelation() {
    let cell = CellId::new(3, 1);
    let mut world = oil_world(cell, OilState::Pending);
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            cell,
            EntityId(1),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(99),
            },
        )
        .unwrap(),
    );
    let mut state = State::new(rules(EnabledItems::EMPTY), world, actors()).unwrap();
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(cell), Some(OilState::Pending));
    state.world.bombs.clear();
    state.actors[0].position = Position::center(cell);
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(cell), Some(OilState::Pending));
    state.actors[0].position = Position::center(CellId::new(1, 1));
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(cell), Some(OilState::Unlit));
}

#[test]
fn standard_stops_at_unlit_oil_while_piercing_continues() {
    let oil = CellId::new(4, 1);
    for (blast, reaches_beyond) in [(BlastKind::Standard, false), (BlastKind::Piercing, true)] {
        let mut world = oil_world(oil, OilState::Unlit);
        world.bombs.insert(
            EntityId(9),
            Bomb::new_with_blast(
                EntityId(9),
                CellId::new(2, 1),
                EntityId(1),
                4,
                Tick(0),
                BombTrigger::Timed {
                    detonates_at: Tick(1),
                },
                blast,
            )
            .unwrap(),
        );
        let plan = crate::game::explosions::plan(&world, Tick(1), [None; 2]);
        assert!(plan.ignited_oil.contains(&oil));
        assert_eq!(plan.flames.contains(&CellId::new(5, 1)), reaches_beyond);
    }
}

#[test]
fn power_ignites_by_manhattan_distance_and_standard_order_is_immutable() {
    let oil = CellId::new(3, 3);
    let mut power = oil_world(oil, OilState::Unlit);
    power.bombs.insert(
        EntityId(9),
        Bomb::new_with_blast(
            EntityId(9),
            CellId::new(3, 1),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
            BlastKind::Power,
        )
        .unwrap(),
    );
    assert!(
        crate::game::explosions::plan(&power, Tick(1), [None; 2])
            .ignited_oil
            .contains(&oil)
    );

    let plan_for = |left_id, right_id| {
        let mut world = oil_world(oil, OilState::Unlit);
        for (id, cell) in [(left_id, CellId::new(3, 1)), (right_id, CellId::new(3, 5))] {
            world.bombs.insert(
                EntityId(id),
                Bomb::new(
                    EntityId(id),
                    cell,
                    EntityId(1),
                    4,
                    Tick(0),
                    BombTrigger::Timed {
                        detonates_at: Tick(1),
                    },
                )
                .unwrap(),
            );
        }
        crate::game::explosions::plan(&world, Tick(1), [None; 2])
    };
    let first = plan_for(10, 20);
    let second = plan_for(20, 10);
    assert_eq!(first.flames, second.flames);
    assert_eq!(first.ignited_oil, second.ignited_oil);
}

#[test]
fn existing_finite_fire_commits_an_ignition_only_plan() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::EMPTY),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.world.hazards.insert(oil, Tick(10));
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(oil), Some(OilState::Burning));
    assert!(!state.world.hazards.contains_key(&oil));
}

#[test]
fn late_effect_overflow_rolls_back_oil_reveal_with_a_positive_control() {
    let oil = CellId::new(3, 1);
    let pickup = CellId::new(1, 1);
    let make_world = || {
        World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(pickup, PickupKind::Shield)]),
            BTreeMap::from([(oil, OilState::Pending)]),
        )
        .unwrap()
    };
    let enabled =
        EnabledItems::from_bits(EnabledItems::OIL_BARREL.bits() | EnabledItems::SHIELD.bits())
            .unwrap();
    let rules = rules(enabled);
    let mut state = State::new_at_tick(rules, make_world(), actors(), Tick(u64::MAX - 1)).unwrap();
    let before = state.clone();
    assert_eq!(
        state.step(&[]),
        Err(crate::game::StepError::EffectExpiryOverflow(
            EffectKind::Shield
        ))
    );
    assert_eq!(state, before);

    let mut control = State::new(rules, make_world(), actors()).unwrap();
    control.step(&[]).unwrap();
    assert_eq!(control.world.oil.state(oil), Some(OilState::Unlit));
    assert!(control.actors[0].effects().damage_immune(Tick(1)));
}

#[test]
fn burning_oil_outlives_finite_fire_without_a_duplicate() {
    let oil = CellId::new(4, 1);
    let mut world = oil_world(oil, OilState::Unlit);
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(2, 1),
            EntityId(1),
            3,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap(),
    );
    let mut safe_actors = actors();
    safe_actors[0].position = Position::center(CellId::new(1, 2));
    let mut state = State::new(rules(EnabledItems::EMPTY), world, safe_actors).unwrap();
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(oil), Some(OilState::Burning));
    assert!(!state.world.hazards.contains_key(&oil));
    for _ in 0..40 {
        state.step(&[]).unwrap();
    }
    assert!(state.world.has_fire(oil, state.tick));
    assert!(state.world.hazards.is_empty());
}

#[test]
fn concealed_oil_becomes_private_pending_after_block_destruction() {
    let oil = CellId::new(5, 1);
    let world = World::with_pickups(
        fixed_arena(),
        BTreeMap::from([(oil, PickupKind::OilBarrel)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let mut state = State::new(rules(EnabledItems::EMPTY), world, actors()).unwrap();
    state.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(3, 1),
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap(),
    );
    state.step(&[]).unwrap();
    assert_eq!(state.world.oil.state(oil), Some(OilState::Pending));
    assert!(state.public_view().oil_barrels.is_empty());
}

#[test]
fn wall_pass_loss_grants_typed_exit_only_grace_and_placement_stays_blocked() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::WALL_PASS),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);
    assert_eq!(state.actors[0].unlit_oil_grace, [oil].into());
    assert!(state.actors[0].soft_wall_grace.is_empty());

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    assert!(state.world.bombs.is_empty());
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::West),
        }])
        .unwrap();
    for _ in 0..19 {
        state.step(&[]).unwrap();
    }
    assert!(state.actors[0].unlit_oil_grace.is_empty());
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
        ])
        .unwrap();
    for _ in 0..30 {
        state.step(&[]).unwrap();
    }
    assert_eq!(state.actors[0].position.x, 2_752);
}

#[test]
fn teleport_rejects_burning_oil_at_either_landing() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::TELEPORTER),
        oil_world(oil, OilState::Burning),
        actors(),
    )
    .unwrap();
    let original = state.actors.each_ref().map(|actor| actor.position);
    state.actors[1].position = Position::center(oil);
    let before = state.actors.each_ref().map(|actor| actor.position);
    state.commit_teleport([true, false], Tick(1));
    assert_eq!(state.actors.each_ref().map(|actor| actor.position), before);
    state.actors[0].position = Position::center(oil);
    state.actors[1].position = original[1];
    let before = state.actors.each_ref().map(|actor| actor.position);
    state.commit_teleport([true, false], Tick(1));
    assert_eq!(state.actors.each_ref().map(|actor| actor.position), before);
}
