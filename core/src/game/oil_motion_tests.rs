use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, Bomb, BombMotion, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment,
    InputAction, OilState, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    StepEventKind, Tick, World, fixed_arena,
};

fn actors(one: CellId) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(one),
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

fn state(oil: CellId, enabled: EnabledItems, one: CellId) -> State {
    let world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Burning)]),
    )
    .unwrap();
    let enabled =
        EnabledItems::from_bits(EnabledItems::OIL_BARREL.bits() | enabled.bits()).unwrap();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        world,
        actors(one),
    )
    .unwrap()
}

fn moving_bomb(cell: CellId) -> Bomb {
    let mut bomb = Bomb::new(
        EntityId(9),
        cell,
        EntityId(1),
        1,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    bomb.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 0,
        }),
        true,
    );
    bomb
}

fn assert_detonated(report: &crate::game::StepReport) {
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(9))
    ));
}

#[test]
fn kicked_bomb_detonates_from_burning_source_or_after_entering_burning_target() {
    let source = CellId::new(7, 5);
    let target = CellId::new(8, 5);
    for burning in [source, target] {
        let mut state = state(burning, EnabledItems::KICK, CellId::new(1, 1));
        state.world.bombs.insert(EntityId(9), moving_bomb(source));

        let report = state.step(&[]).unwrap();

        assert!(!state.world.bombs.contains_key(&EntityId(9)));
        assert_detonated(&report);
    }
}

#[test]
fn punch_rejects_a_burning_source_but_a_burning_landing_detonates_after_punch() {
    let actor = CellId::new(3, 5);
    let source = CellId::new(4, 5);
    let landing = CellId::new(7, 5);
    for (burning, punched) in [(source, false), (landing, true)] {
        let mut state = state(burning, EnabledItems::PUNCH, actor);
        state.replace_actor_equipment(SeatId::One, Equipment::Punch);
        state.world.bombs.insert(
            EntityId(9),
            Bomb::new(
                EntityId(9),
                source,
                EntityId(2),
                1,
                Tick(0),
                BombTrigger::Timed {
                    detonates_at: Tick(100),
                },
            )
            .unwrap(),
        );

        let report = state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::UseEquipment,
            }])
            .unwrap();

        assert_eq!(
            state.actors[0].last_successful_punch_tick().is_some(),
            punched
        );
        assert!(!state.world.bombs.contains_key(&EntityId(9)));
        assert_detonated(&report);
    }
}

#[test]
fn burning_oil_detonates_remote_bombs_but_only_destroys_unaccepted_contact_mines() {
    let fire = CellId::new(7, 5);
    for (trigger, detonates) in [(BombTrigger::Remote, true), (BombTrigger::Contact, false)] {
        let item = if detonates {
            EnabledItems::REMOTE_BOMB
        } else {
            EnabledItems::LANDMINE
        };
        let mut state = state(fire, item, CellId::new(1, 1));
        state.world.bombs.insert(
            EntityId(9),
            Bomb::new(EntityId(9), fire, EntityId(1), 1, Tick(0), trigger).unwrap(),
        );

        let report = state.step(&[]).unwrap();

        assert!(state.world.bombs.is_empty());
        assert_eq!(
            report.events.iter().any(|event| matches!(
                event.kind,
                StepEventKind::BombDetonated(view) if view.id == EntityId(9)
            )),
            detonates
        );
    }
}

#[test]
fn contact_accepted_before_burning_oil_damage_still_detonates_once() {
    let fire = CellId::new(3, 1);
    let mut state = state(
        fire,
        EnabledItems::from_bits(EnabledItems::LANDMINE.bits() | EnabledItems::SHIELD.bits())
            .unwrap(),
        CellId::new(2, 1),
    );
    state.actors[0].position.x = 2_752;
    state.apply_test_effect(SeatId::One, crate::game::EffectKind::Shield, Tick(0));
    state.world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            fire,
            EntityId(2),
            1,
            Tick(0),
            BombTrigger::Contact,
        )
        .unwrap(),
    );

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert!(state.actors[0].alive);
    assert_eq!(
        report
            .events
            .iter()
            .filter(|event| matches!(
                event.kind,
                StepEventKind::BombDetonated(view) if view.id == EntityId(9)
            ))
            .count(),
        1
    );
}
