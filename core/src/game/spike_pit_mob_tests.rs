use super::spike_pit_state_tests::{PIT, SAFE, actors, enabled, step_to};
use super::State;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{
    BombTrigger, CellId, Direction, EnabledItems, EntityId, MovementSegment, MovementTrace,
    Position, RulesConfig, SeatId, StormCloudConfig, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings};

fn mob_state(items: EnabledItems) -> State {
    State::new(
        RulesConfig::environment_v10(
            RoomSettings::DEFAULT.with_enabled_items(items),
            StormCloudConfig::Disabled,
            RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
            enabled(),
        ),
        World::new(fixed_arena()),
        actors(CellId::new(7, 5), SAFE),
    )
    .unwrap()
}

#[test]
fn idle_newborn_contact_arms_warning_then_death_schedules_one_replacement() {
    let mut game = mob_state(EnabledItems::EMPTY);
    step_to(&mut game, 300);

    assert!(game.mobs().iter().any(|mob| mob.position == Position::center(PIT)));
    assert_eq!(game.spike_pits().unwrap()[0].phase, crate::game::SpikePitPhase::Warning);
    step_to(&mut game, 306);
    assert!(!game.mobs().iter().any(|mob| mob.lair == 0));
    assert_eq!(game.lair_statuses().unwrap()[0].next_spawn_at, Some(Tick(606)));

    game.step(&[]).unwrap();
    assert_eq!(game.lair_statuses().unwrap()[0].next_spawn_at, Some(Tick(606)));
}

#[test]
fn warning_pit_is_excluded_from_lair_spawn() {
    let mut game = mob_state(EnabledItems::EMPTY);
    step_to(&mut game, 298);
    game.set_test_actor_pose(SeatId::One, Position::center(PIT), Direction::East);
    game.step(&[]).unwrap();
    game.set_test_actor_pose(SeatId::One, Position::center(CellId::new(7, 5)), Direction::East);
    game.step(&[]).unwrap();

    assert_eq!(game.spike_pits().unwrap()[0].phase, crate::game::SpikePitPhase::Warning);
    assert!(game.mobs().iter().filter(|mob| mob.lair == 0).all(|mob| {
        mob.position != Position::center(PIT)
    }));
}

#[test]
fn mob_trace_reaches_contact_mine_before_raised_pit_removes_the_mob() {
    let mut game = mob_state(EnabledItems::LANDMINE);
    game.set_test_actor_pose(SeatId::One, Position::center(PIT), Direction::East);
    game.step(&[]).unwrap();
    game.set_test_actor_pose(
        SeatId::One,
        Position::center(CellId::new(7, 5)),
        Direction::East,
    );
    step_to(&mut game, 7);

    let mob_id = EntityId(50);
    let tangent = Position {
        x: i32::from(PIT.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
        y: Position::center(PIT).y,
    };
    game.lairs
        .as_mut()
        .unwrap()
        .insert_test_mob(mob_id, 0, tangent, Direction::East);
    game.insert_test_bomb(
        EntityId(99),
        PIT,
        EntityId(1),
        1,
        Tick(7),
        BombTrigger::Contact,
    )
    .unwrap();
    let end = Position {
        x: tangent.x + 25,
        y: tangent.y,
    };
    let mut trace = MovementTrace::new(tangent);
    trace.push(MovementSegment::new(tangent, end));
    let mine_targets = game
        .lairs
        .as_ref()
        .unwrap()
        .mine_contacts(game.world(), std::slice::from_ref(&trace));

    game.advance_spike_pits_for_mob_traces(&[(mob_id, trace)], Tick(7))
        .unwrap();
    assert!(mine_targets.contains(&EntityId(99)));
    assert!(!game.mobs().iter().any(|mob| mob.id == mob_id));
    assert_eq!(game.lair_statuses().unwrap()[0].next_spawn_at, Some(Tick(307)));
}

#[test]
fn pit_killed_mob_trace_triggers_mine_but_cannot_harm_player() {
    let mut game = mob_state(EnabledItems::LANDMINE);
    game.set_test_actor_pose(SeatId::One, Position::center(PIT), Direction::East);
    game.step(&[]).unwrap();
    game.set_test_actor_pose(
        SeatId::One,
        Position::center(CellId::new(7, 5)),
        Direction::East,
    );
    step_to(&mut game, 7);

    let left = i32::from(PIT.column) * UNITS_PER_CELL;
    let top = i32::from(PIT.row) * UNITS_PER_CELL;
    let mob_start = Position {
        x: left - HALF_FOOTPRINT,
        y: top - HALF_FOOTPRINT + 1,
    };
    let player = Position {
        x: mob_start.x + HALF_FOOTPRINT * 2 + 8,
        y: top - HALF_FOOTPRINT,
    };
    game.set_test_actor_pose(SeatId::One, player, Direction::West);
    let mob_id = EntityId(50);
    game.lairs
        .as_mut()
        .unwrap()
        .insert_test_mob(mob_id, 0, mob_start, Direction::East);
    let mine_id = EntityId(99);
    let mine_cell = CellId::new(PIT.column - 1, PIT.row - 1);
    game.insert_test_bomb(
        mine_id,
        mine_cell,
        EntityId(1),
        0,
        Tick(7),
        BombTrigger::Contact,
    )
    .unwrap();

    game.step(&[]).unwrap();

    assert!(!game.mobs().iter().any(|mob| mob.id == mob_id));
    assert!(game.actors()[0].alive());
    assert!(!game.world().bombs().contains_key(&mine_id));
}
