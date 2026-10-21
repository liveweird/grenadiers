use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, Bomb, BombTrigger, CellId, DamagePreventionSource, Direction, EffectKind,
    EnabledItems, EntityId, Equipment, InputAction, OilState, PickupKind, Position, RulesConfig,
    SeatId, SeatInput, SpecialWeapon, SpeedLevel, State, StepError, StepEventKind, Tick, World,
    fixed_arena,
};

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

fn oil_rules(extra: EnabledItems) -> RulesConfig {
    let enabled = EnabledItems::from_bits(EnabledItems::OIL_BARREL.bits() | extra.bits()).unwrap();
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

#[test]
fn ignition_removes_unlit_exit_grace_from_a_shielded_survivor() {
    let oil = CellId::new(3, 1);
    let mut world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Unlit)]),
    )
    .unwrap();
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(2, 1),
            EntityId(2),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap(),
    );
    let mut state = State::new(oil_rules(EnabledItems::WALL_PASS), world, actors()).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);
    state.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    assert_eq!(state.actors[0].unlit_oil_grace, [oil].into());

    let report = state.step(&[]).unwrap();

    assert!(state.actors[0].alive);
    assert_eq!(state.world.oil.state(oil), Some(OilState::Burning));
    assert!(state.actors[0].unlit_oil_grace.is_empty());
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    seat: SeatId::One,
                    source: DamagePreventionSource::Shield,
                    ..
                }
            ))
            .count(),
        1
    );
}

#[test]
fn perpendicular_ignition_never_changes_a_standard_ray_mid_plan() {
    let oil = CellId::new(5, 3);
    let beyond = CellId::new(6, 3);
    let plan_for = |vertical_id, horizontal_id| {
        let mut world = World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(oil, OilState::Unlit)]),
        )
        .unwrap();
        for (id, cell, range) in [
            (vertical_id, CellId::new(5, 5), 2),
            (horizontal_id, CellId::new(3, 3), 4),
        ] {
            world.bombs.insert(
                EntityId(id),
                Bomb::new(
                    EntityId(id),
                    cell,
                    EntityId(1),
                    range,
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

    let first = plan_for(9, 10);
    let swapped = plan_for(10, 9);
    assert!(first.ignited_oil.contains(&oil));
    assert!(!first.flames.contains(&beyond));
    assert_eq!(first.flames, swapped.flames);
    assert_eq!(first.ignited_oil, swapped.ignited_oil);
}

#[test]
fn ignition_only_removes_finite_fire_without_creating_adjacent_flames() {
    let oil = CellId::new(3, 1);
    let adjacent = CellId::new(4, 1);
    let world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Unlit)]),
    )
    .unwrap();
    let mut state = State::new(oil_rules(EnabledItems::EMPTY), world, actors()).unwrap();
    state.world.hazards.insert(oil, Tick(10));

    state.step(&[]).unwrap();

    assert_eq!(state.world.oil.state(oil), Some(OilState::Burning));
    assert!(state.world.hazards.is_empty());
    assert!(!state.world.has_fire(adjacent, state.tick));
}

#[test]
fn late_pickup_overflow_rolls_back_ignition_damage_grace_and_actions() {
    let oil = CellId::new(4, 1);
    let pickup = CellId::new(13, 11);
    let make_state = |tick: Tick| {
        let mut world = World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(pickup, PickupKind::Shield)]),
            BTreeMap::from([(oil, OilState::Unlit)]),
        )
        .unwrap();
        world.bombs.insert(
            EntityId(9),
            Bomb::new(
                EntityId(9),
                CellId::new(2, 1),
                EntityId(2),
                2,
                tick,
                BombTrigger::Timed {
                    detonates_at: Tick(tick.0 + 1),
                },
            )
            .unwrap(),
        );
        let enabled = EnabledItems::from_bits(
            EnabledItems::OIL_BARREL.bits()
                | EnabledItems::WALL_PASS.bits()
                | EnabledItems::SHIELD.bits(),
        )
        .unwrap();
        let mut state = State::new_at_tick(oil_rules(enabled), world, actors(), tick).unwrap();
        state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
        state.actors[0].position = Position::center(oil);
        state.replace_actor_equipment(SeatId::One, Equipment::Heart);
        state
    };
    let inputs = [SeatInput {
        seat: SeatId::Two,
        action: InputAction::Press(Direction::West),
    }];

    let mut overflow = make_state(Tick(u64::MAX - 1));
    let before = overflow.clone();
    assert_eq!(
        overflow.step(&inputs),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(overflow, before);

    let mut control = make_state(Tick(0));
    let report = control.step(&inputs).unwrap();
    assert_eq!(control.world.oil.state(oil), Some(OilState::Burning));
    assert!(control.actors[0].unlit_oil_grace.is_empty());
    assert_eq!(control.actors[0].inventory.equipment(), None);
    assert!(control.actors[1].effects.damage_immune(Tick(1)));
    assert!(!report.events.is_empty());
}

#[test]
fn existing_remote_detonation_keeps_priority_over_unlit_placement_veto() {
    let oil = CellId::new(3, 1);
    let remote = CellId::new(7, 5);
    let mut world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Unlit)]),
    )
    .unwrap();
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            remote,
            EntityId(1),
            1,
            Tick(0),
            BombTrigger::Remote,
        )
        .unwrap(),
    );
    let enabled =
        EnabledItems::from_bits(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::WALL_PASS.bits())
            .unwrap();
    let mut state = State::new(oil_rules(enabled), world, actors()).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    assert!(!state.world.bombs.contains_key(&EntityId(9)));
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(9))
    ));
    assert!(!state.world.bombs.values().any(|bomb| bomb.cell() == oil));
}

#[test]
fn newly_placed_mine_on_burning_oil_is_destroyed_without_inheriting_contact() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        oil_rules(
            EnabledItems::from_bits(EnabledItems::LANDMINE.bits() | EnabledItems::SHIELD.bits())
                .unwrap(),
        ),
        World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(oil, OilState::Burning)]),
        )
        .unwrap(),
        actors(),
    )
    .unwrap();
    state.actors[0].position = Position::center(oil);
    state.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Landmine);
    state.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(0));

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    assert!(state.actors[0].alive);
    assert!(state.world.bombs.is_empty());
    assert_eq!(state.actors[0].inventory.weapon(), None);
    assert!(
        report
            .events
            .iter()
            .any(|event| matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
    assert!(
        !report
            .events
            .iter()
            .any(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
    );
}
