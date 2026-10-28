use std::collections::BTreeMap;

use crate::room::RoomSettings;

use super::super::observations::ExitGraceObservation;
use crate::game::{
    ActorInit, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, OilState,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, Tick, World, fixed_arena,
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

fn oil_world(cell: CellId, state: OilState) -> World {
    World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cell, state)]),
    )
    .unwrap()
}

#[test]
fn typed_oil_grace_is_cleared_by_reacquisition_and_death() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::WALL_PASS),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position::center(oil);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    assert_eq!(
        state.actors[0]
            .exit_grace_observations()
            .collect::<Vec<_>>(),
        [ExitGraceObservation::UnlitOil(oil)]
    );

    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    assert!(state.actors[0].unlit_oil_grace.is_empty());
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state.world.oil.set_state(oil, OilState::Burning);
    state.step(&[]).unwrap();
    assert!(!state.actors[0].alive);
    assert!(state.actors[0].unlit_oil_grace.is_empty());
}

#[test]
fn teleport_uses_each_actors_own_wall_pass_capability() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::WALL_PASS),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.replace_actor_equipment(SeatId::Two, Equipment::WallPass);
    state.actors[1].position = Position::center(oil);
    let before = state
        .actors
        .iter()
        .map(|actor| actor.position)
        .collect::<Vec<_>>();

    state.commit_teleport([true, false], Tick(1));

    assert_eq!(state.actors[0].position, before[1]);
    assert_eq!(state.actors[1].position, before[0]);
    assert_eq!(state.last_teleport_tick, Some(Tick(1)));
}

#[test]
fn teleport_retains_only_the_owners_oil_grace_across_a_successful_swap() {
    let oil = CellId::new(3, 1);
    let mut state = State::new(
        rules(EnabledItems::WALL_PASS),
        oil_world(oil, OilState::Unlit),
        actors(),
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position {
        x: Position::center(oil).x - 100,
        ..Position::center(oil)
    };
    state.replace_actor_equipment(SeatId::One, Equipment::Heart);
    state.replace_actor_equipment(SeatId::Two, Equipment::WallPass);
    state.actors[1].position = Position {
        x: Position::center(oil).x + 100,
        ..Position::center(oil)
    };
    let before = state
        .actors
        .iter()
        .map(|actor| actor.position)
        .collect::<Vec<_>>();

    state.commit_teleport([true, false], Tick(1));

    assert_eq!(state.actors[0].position, before[1]);
    assert_eq!(state.actors[1].position, before[0]);
    assert_eq!(state.actors[0].unlit_oil_grace, [oil].into());
    assert!(state.actors[1].unlit_oil_grace.is_empty());
}

#[test]
fn forced_exit_prunes_real_unlit_grace_and_cannot_be_used_for_reentry() {
    let oil = CellId::new(7, 5);
    let enabled =
        EnabledItems::from_bits(EnabledItems::BRACELET.bits() | EnabledItems::WALL_PASS.bits())
            .unwrap();
    let mut state = State::new(rules(enabled), oil_world(oil, OilState::Unlit), actors()).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.actors[0].position = Position {
        x: 7_871,
        y: Position::center(oil).y,
    };
    state.replace_actor_equipment(SeatId::Two, Equipment::WallPass);
    state.actors[1].position = Position {
        x: 8_511,
        y: Position::center(oil).y,
    };
    state.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    state.replace_actor_equipment(SeatId::Two, Equipment::Heart);
    assert_eq!(state.actors[1].unlit_oil_grace, [oil].into());

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    assert!(report.forced_movements.len() == 1);
    assert!(state.actors[1].unlit_oil_grace.is_empty());
    assert!(!crate::game::movement::position_overlaps_cell(
        state.actors[1].position,
        oil
    ));

    state.actors[0].position = Position::center(CellId::new(1, 1));
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::West),
            },
        ])
        .unwrap();
    for _ in 0..30 {
        state.step(&[]).unwrap();
    }
    assert_eq!(state.actors[1].position.x, 8_512);
}
