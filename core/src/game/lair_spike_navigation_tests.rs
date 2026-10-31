use super::State;
use crate::game::{
    Direction, EffectKind, EnabledItems, EntityId, MobNavigationPolicy, Position, RulesConfig,
    SeatId, SpikePitPhase, StormCloudConfig, Tick, World, fixed_arena, position_contacts_spike_pit,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

const ESCAPE_PIT: crate::game::CellId = crate::game::CellId::new(7, 5);
const SPAWN_PIT: crate::game::CellId = crate::game::CellId::new(6, 5);

fn state(pit: crate::game::CellId) -> State {
    let mut world = World::new(fixed_arena());
    let blocks = world.arena.blocks().clone();
    world.arena.remove_blocks(&blocks);
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::EMPTY);
    let rules = RulesConfig::environment_v10_with_mob_navigation(
        settings,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        SpikePitConfig::Enabled(SpikePitSettings::default()),
        MobNavigationPolicy::EscapeV2,
    );
    let actors = super::super::spike_pit_state_tests::actors(
        crate::game::CellId::new(1, 1),
        crate::game::CellId::new(13, 11),
    );
    let mut game = State::new(rules, world, actors).unwrap();
    let pit_count = game.spike_pits().unwrap().len();
    for index in 0..pit_count {
        game.set_test_spike_pit_cell(index, pit);
    }
    game
}

fn step_to(game: &mut State, tick: u64) {
    while game.tick().0 < tick {
        game.step(&[]).unwrap();
    }
}

#[test]
fn occupied_pit_egress_makes_progress_on_every_axis_and_survives_the_rise() {
    for heading in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let mut game = state(ESCAPE_PIT);
        let id = EntityId(90);
        game.lairs
            .as_mut()
            .unwrap()
            .insert_test_mob(id, 0, Position::center(ESCAPE_PIT), heading);
        let start = Position::center(ESCAPE_PIT);
        let mut previous_progress = 0;
        for _ in 0..34 {
            game.step(&[]).unwrap();
            let position = game
                .mobs()
                .into_iter()
                .find(|mob| mob.id == id)
                .unwrap()
                .position;
            let progress = manhattan(start, position);
            assert!(
                progress >= previous_progress,
                "reversed while escaping {heading:?}"
            );
            previous_progress = progress;
        }
        assert!(
            previous_progress > 800,
            "made too little progress {heading:?}: {previous_progress}"
        );
        step_to(&mut game, 37);
        let mob = game.mobs().into_iter().find(|mob| mob.id == id).unwrap();
        assert_eq!(mob.heading, heading);
        assert!(!position_contacts_spike_pit(mob.position, ESCAPE_PIT));
        assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Raised);
    }
}

#[test]
fn default_warning_allows_a_newborn_to_clear_its_spawn_pit() {
    let mut game = state(SPAWN_PIT);
    step_to(&mut game, 300);
    let newborn = game
        .mobs()
        .into_iter()
        .find(|mob| mob.lair == 0)
        .expect("first lair spawned");
    assert_eq!(newborn.position, Position::center(SPAWN_PIT));
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Warning);

    step_to(&mut game, 336);
    let escaped = game
        .mobs()
        .into_iter()
        .find(|mob| mob.id == newborn.id)
        .expect("newborn survived the rise");
    assert!(!position_contacts_spike_pit(escaped.position, SPAWN_PIT));
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Raised);
}

#[test]
fn frozen_occupant_still_dies_when_the_pit_rises() {
    let mut game = state(ESCAPE_PIT);
    let id = EntityId(90);
    game.lairs.as_mut().unwrap().insert_test_mob(
        id,
        0,
        Position::center(ESCAPE_PIT),
        Direction::East,
    );
    game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));

    step_to(&mut game, 37);

    assert!(game.mobs().into_iter().all(|mob| mob.id != id));
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Raised);
}

#[test]
fn footprint_frontier_stops_a_mob_before_pit_contact() {
    let mut game = state(ESCAPE_PIT);
    let id = EntityId(90);
    let origin = Position::center(crate::game::CellId::new(
        ESCAPE_PIT.column - 1,
        ESCAPE_PIT.row,
    ));
    game.lairs
        .as_mut()
        .unwrap()
        .insert_test_mob(id, 0, origin, Direction::East);

    for _ in 0..20 {
        game.step(&[]).unwrap();
        let mob = game.mobs().into_iter().find(|mob| mob.id == id).unwrap();
        assert!(!position_contacts_spike_pit(mob.position, ESCAPE_PIT));
    }
    assert_eq!(game.spike_pits().unwrap()[0].phase, SpikePitPhase::Idle);
}

#[test]
fn accidental_overlap_retreats_without_direction_oscillation() {
    let mut game = state(ESCAPE_PIT);
    let id = EntityId(90);
    let pit_left = i32::from(ESCAPE_PIT.column) * crate::game::types::UNITS_PER_CELL;
    let position = Position {
        x: pit_left - crate::game::types::HALF_FOOTPRINT + 1,
        y: Position::center(ESCAPE_PIT).y,
    };
    game.lairs
        .as_mut()
        .unwrap()
        .insert_test_mob(id, 0, position, Direction::East);

    let mut previous_x = position.x;
    for _ in 0..8 {
        game.step(&[]).unwrap();
        let mob = game.mobs().into_iter().find(|mob| mob.id == id).unwrap();
        assert!(mob.position.x < previous_x);
        assert_eq!(mob.heading, Direction::West);
        previous_x = mob.position.x;
    }
    assert!(!position_contacts_spike_pit(
        game.mobs()
            .into_iter()
            .find(|mob| mob.id == id)
            .unwrap()
            .position,
        ESCAPE_PIT,
    ));
}

fn manhattan(left: Position, right: Position) -> u64 {
    i64::from(left.x).abs_diff(i64::from(right.x)) + i64::from(left.y).abs_diff(i64::from(right.y))
}
