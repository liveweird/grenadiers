use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, Bomb, BombTrigger, CellId, DamagePreventionSource, Direction, EffectKind,
    EnabledItems, EntityId, Equipment, InputAction, OilState, Position, RulesConfig, SeatId,
    SeatInput, SpeedLevel, State, StepEventKind, Tick, World, fixed_arena,
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

fn rules(extra: EnabledItems) -> RulesConfig {
    let enabled = EnabledItems::from_bits(EnabledItems::OIL_BARREL.bits() | extra.bits()).unwrap();
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

fn oil_world(sites: impl IntoIterator<Item = (CellId, OilState)>) -> World {
    World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        sites.into_iter().collect(),
    )
    .unwrap()
}

fn prevention_count(report: &crate::game::StepReport, source: DamagePreventionSource) -> usize {
    report
        .events
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                StepEventKind::DamagePrevented {
                    seat: SeatId::One,
                    source: actual,
                    ..
                } if actual == source
            )
        })
        .count()
}

#[test]
fn shield_event_is_deduplicated_across_old_and_newly_persistent_fire() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::SHIELD),
        oil_world([(oil, OilState::Unlit)]),
        actors(),
    )
    .unwrap();
    state.actors[0].position = Position::center(oil);
    state.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(0));
    state.world.hazards.insert(oil, Tick(10));

    let report = state.step(&[]).unwrap();

    assert!(state.actors[0].alive);
    assert_eq!(state.world.oil.state(oil), Some(OilState::Burning));
    assert_eq!(prevention_count(&report, DamagePreventionSource::Shield), 1);
}

#[test]
fn heart_survival_keeps_grace_for_a_distinct_still_unlit_barrel() {
    let retained = CellId::new(2, 1);
    let ignited = CellId::new(3, 1);
    let mut world = oil_world([(retained, OilState::Unlit), (ignited, OilState::Unlit)]);
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(4, 1),
            EntityId(2),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap(),
    );
    let mut state = State::new(rules(EnabledItems::WALL_PASS), world, actors()).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position {
        x: 3_072,
        y: Position::center(ignited).y,
    };
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);
    assert_eq!(state.actors[0].unlit_oil_grace.len(), 2);

    let report = state.step(&[]).unwrap();

    assert!(state.actors[0].alive);
    assert_eq!(state.actors[0].inventory.equipment(), None);
    assert_eq!(state.world.oil.state(ignited), Some(OilState::Burning));
    assert_eq!(state.world.oil.state(retained), Some(OilState::Unlit));
    assert_eq!(state.actors[0].unlit_oil_grace, [retained].into());
    assert_eq!(prevention_count(&report, DamagePreventionSource::Heart), 1);
}

#[test]
fn a_trace_that_leaves_oil_before_ignition_is_not_hit_retroactively() {
    let oil = CellId::new(3, 1);
    let mut world = oil_world([(oil, OilState::Unlit)]);
    world.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            CellId::new(4, 1),
            EntityId(2),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap(),
    );
    let mut state = State::new(rules(EnabledItems::WALL_PASS), world, actors()).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position {
        x: 2_753,
        ..Position::center(oil)
    };
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::West),
        }])
        .unwrap();

    assert!(report.traces[0].overlaps_cell(oil));
    assert!(!crate::game::movement::position_overlaps_cell(
        state.actors[0].position,
        oil
    ));
    assert!(state.actors[0].alive);
    assert_eq!(state.world.oil.state(oil), Some(OilState::Burning));
    assert!(state.actors[0].unlit_oil_grace.is_empty());
    assert!(!report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::ActorEliminated {
            seat: SeatId::One,
            ..
        } | StepEventKind::DamagePrevented {
            seat: SeatId::One,
            ..
        }
    )));
}
