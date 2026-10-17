use std::collections::BTreeSet;

use jsonschema::validator_for;
use serde_json::{Value, json};

use super::{DrawReason, MAX_SAFE_TICK, ProjectionError, Seat, WireRoundResult, project_state};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::simulation::{ActorInit, RoundResult, State};
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;

fn open_world() -> World {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    World::new(
        Arena::new(
            terrain,
            BTreeSet::new(),
            [CellId::new(1, 1), CellId::new(13, 11)],
        )
        .unwrap(),
    )
}

fn actor(id: u32, seat: SeatId, cell: CellId) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn state_at(one: CellId, two: CellId) -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, two)],
    )
    .unwrap()
}

fn initial_view() -> crate::game::PublicState {
    state_at(CellId::new(1, 1), CellId::new(13, 11)).public_view()
}

fn schema_for(name: &str) -> jsonschema::Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay.schema.json")).unwrap();
    schema["$ref"] = json!(format!("#/$defs/{name}"));
    validator_for(&schema).unwrap()
}

#[test]
fn initial_and_active_domain_states_project_to_schema_valid_game_state() {
    let mut state = state_at(CellId::new(1, 1), CellId::new(13, 11));
    let initial = project_state(&state.public_view()).unwrap();
    assert_eq!(initial.tick.get(), 0);
    assert_eq!(initial.result, None);
    assert_eq!(initial.state.running_updates, 0);
    assert_eq!(initial.state.remaining_ticks, 10_800);
    assert!(schema_for("GameState").is_valid(&serde_json::to_value(&initial.state).unwrap()));

    state
        .step(&[crate::game::SeatInput {
            seat: SeatId::One,
            action: crate::game::InputAction::PlaceBomb,
        }])
        .unwrap();
    let active = project_state(&state.public_view()).unwrap();
    assert_eq!(active.state.bombs.len(), 1);
    assert_eq!(active.state.actors[0].live_bomb_count, 1);
    assert_eq!(active.state.bombs[0].owner.get(), 10);
    assert!(schema_for("GameState").is_valid(&serde_json::to_value(&active.state).unwrap()));
}

#[test]
fn result_projection_accepts_frozen_stale_deadlines_and_expiries() {
    let shared = CellId::new(4, 5);
    let mut state = state_at(shared, CellId::new(12, 10));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        shared,
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(31),
        CellId::new(8, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(2),
    )
    .unwrap();
    state.step(&[]).unwrap();
    for _ in 0..40 {
        state.step(&[]).unwrap();
    }

    let projected = project_state(&state.public_view()).unwrap();

    assert_eq!(projected.tick.get(), 41);
    assert_eq!(
        projected.result,
        Some(WireRoundResult::Winner { seat: Seat::Two })
    );
    assert_eq!(projected.state.bombs[0].detonates_at.get(), 2);
    assert!(
        projected
            .state
            .flames
            .iter()
            .all(|flame| flame.expires_at.get() < projected.tick.get())
    );
}

#[test]
fn explicit_speed_and_result_mappings_cover_every_domain_variant() {
    for (speed, expected) in [
        (SpeedLevel::Base, 0),
        (SpeedLevel::One, 1),
        (SpeedLevel::Two, 2),
        (SpeedLevel::Three, 3),
        (SpeedLevel::Four, 4),
    ] {
        let mut view = initial_view();
        view.actors[0].speed = speed;
        assert_eq!(
            project_state(&view).unwrap().state.actors[0].speed_level,
            expected
        );
    }

    let mut timeout = initial_view();
    timeout.running_updates = 10_800;
    timeout.remaining_ticks = 0;
    timeout.result = Some(RoundResult::Draw(crate::game::DrawReason::Timeout));
    assert_eq!(
        project_state(&timeout).unwrap().result,
        Some(WireRoundResult::Draw {
            reason: DrawReason::Timeout
        })
    );

    let mut mutual = initial_view();
    mutual
        .actors
        .iter_mut()
        .for_each(|actor| actor.alive = false);
    mutual.result = Some(RoundResult::Draw(
        crate::game::DrawReason::MutualElimination,
    ));
    assert!(project_state(&mutual).is_ok());
}

#[test]
fn rejects_rules_tick_clock_and_arena_shape_failures() {
    let mut view = initial_view();
    view.rules_version = 5;
    assert_eq!(
        project_state(&view),
        Err(ProjectionError::UnsupportedRulesVersion)
    );

    let mut view = initial_view();
    view.tick = Tick(MAX_SAFE_TICK + 1);
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidTick));

    for mutate in [
        |view: &mut crate::game::PublicState| view.running_updates = 10_801,
        |view: &mut crate::game::PublicState| view.remaining_ticks = 10_799,
    ] {
        let mut view = initial_view();
        mutate(&mut view);
        assert_eq!(
            project_state(&view),
            Err(ProjectionError::InvalidRunningClock)
        );
    }

    let mut view = initial_view();
    view.arena.width = 14;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidArena));
    let mut view = initial_view();
    view.arena.terrain.pop();
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidArena));
    let mut view = initial_view();
    view.arena.blocks = vec![CellId::new(2, 2), CellId::new(1, 1)];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidArena));
    let mut view = initial_view();
    view.arena.blocks = vec![CellId::new(0, 0)];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidArena));
}

#[test]
fn rejects_actor_identity_position_upgrade_and_count_failures() {
    let mut view = initial_view();
    view.actors[1].seat = SeatId::One;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidActors));
    let mut view = initial_view();
    view.actors[1].id = view.actors[0].id;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidActors));
    let mut view = initial_view();
    view.actors[0].position.x = -1;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidActors));
    let mut view = initial_view();
    view.actors[0].position.x = 15_360;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidActors));
    for mutate in [
        |view: &mut crate::game::PublicState| view.actors[0].bomb_capacity = 0,
        |view: &mut crate::game::PublicState| view.actors[0].bomb_range = 9,
        |view: &mut crate::game::PublicState| view.actors[0].live_bomb_count = 2,
    ] {
        let mut view = initial_view();
        mutate(&mut view);
        assert_eq!(project_state(&view), Err(ProjectionError::InvalidActors));
    }
}

#[test]
fn rejects_bomb_and_flame_bounds_identity_order_and_deadline_failures() {
    let mut state = state_at(CellId::new(1, 1), CellId::new(13, 11));
    state
        .step(&[crate::game::SeatInput {
            seat: SeatId::One,
            action: crate::game::InputAction::PlaceBomb,
        }])
        .unwrap();
    let active = state.public_view();

    let mut view = active.clone();
    view.bombs[0].owner = EntityId(999);
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.bombs[0].id = view.actors[0].id;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.bombs[0].range = 0;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.bombs[0].detonates_at = Tick(0);
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.actors[0].live_bomb_count = 0;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.bombs = vec![view.bombs[0]; 17];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));

    let mut second = active.bombs[0];
    second.id = EntityId(second.id.0 + 1);
    second.owner = active.actors[1].id;
    second.cell = CellId::new(12, 10);
    let mut view = active.clone();
    view.actors[1].live_bomb_count = 1;
    view.bombs = vec![second, active.bombs[0]];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.actors[1].live_bomb_count = 1;
    second.cell = active.bombs[0].cell;
    view.bombs.push(second);
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidBombs));
    let mut view = active.clone();
    view.bombs[0].detonates_at = Tick(MAX_SAFE_TICK + 1);
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidTick));

    let mut view = initial_view();
    view.flames = vec![
        crate::game::FlameView {
            cell: CellId::new(2, 5),
            expires_at: Tick(10),
        },
        crate::game::FlameView {
            cell: CellId::new(2, 5),
            expires_at: Tick(10),
        },
    ];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidFlames));
    let mut view = initial_view();
    view.flames = vec![crate::game::FlameView {
        cell: CellId::new(0, 0),
        expires_at: Tick(10),
    }];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidFlames));
    let mut view = initial_view();
    view.flames = vec![crate::game::FlameView {
        cell: CellId::new(2, 5),
        expires_at: Tick(0),
    }];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidFlames));
    let mut view = initial_view();
    view.flames = vec![crate::game::FlameView {
        cell: CellId::new(2, 5),
        expires_at: Tick(MAX_SAFE_TICK + 1),
    }];
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidTick));
}

#[test]
fn rejects_results_that_disagree_with_life_and_running_clock() {
    let mut view = initial_view();
    view.result = Some(RoundResult::Winner(SeatId::One));
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidResult));

    let mut view = initial_view();
    view.actors[1].alive = false;
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidResult));

    let mut view = initial_view();
    view.result = Some(RoundResult::Draw(crate::game::DrawReason::Timeout));
    assert_eq!(project_state(&view), Err(ProjectionError::InvalidResult));
}

#[path = "pickup_projection_tests.rs"]
mod pickups;
