use std::collections::{BTreeMap, BTreeSet};

use super::mob_escape::{EscapeEnvironment, escape_direction};
use super::mob_hazards::{TimedMobHazards, WorkBudget};
use super::{
    ActorInit, Arena, BlastKind, Bomb, BombTrigger, CellId, Direction, EffectKind, EntityId,
    MobNavigationPolicy, OilState, Position, RulesConfig, SeatId, SpeedLevel, State,
    StormCloudConfig, Tick, World,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings};

const MOB_ID: EntityId = EntityId(90);
const BOMB_ID: EntityId = EntityId(91);
const START: CellId = CellId::new(3, 3);

fn corridor_arena(extra_blocks: BTreeSet<CellId>) -> Arena {
    Arena::new(
        super::fixtures::classic_terrain(),
        extra_blocks,
        super::fixtures::classic_spawns(),
    )
    .unwrap()
}

fn state(policy: MobNavigationPolicy, deadline: u64) -> State {
    state_with_range(policy, deadline, 3)
}

fn state_with_range(policy: MobNavigationPolicy, deadline: u64, range: u16) -> State {
    let arena = corridor_arena(BTreeSet::from([CellId::new(2, 3), CellId::new(4, 3)]));
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(arena.spawns()[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v9_with_mob_navigation(
        RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(60).unwrap()),
        policy,
    );
    let mut state = State::new(rules, World::new(arena), actors).unwrap();
    state.insert_test_mob(MOB_ID, Position::center(START), Direction::South);
    state
        .insert_test_bomb(
            BOMB_ID,
            CellId::new(3, 5),
            EntityId(10),
            range,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(deadline),
            },
        )
        .unwrap();
    state
}

#[test]
fn nav2_waits_in_a_multi_cell_blast_corridor_while_nav3_reaches_refuge() {
    let mut nav2 = state(MobNavigationPolicy::EscapeV2, 100);
    let mut nav3 = state(MobNavigationPolicy::TimedEscapeV3, 100);

    for _ in 0..90 {
        nav2.step(&[]).unwrap();
        nav3.step(&[]).unwrap();
    }

    assert_eq!(nav2.mobs()[0].position, Position::center(START));
    assert_eq!(nav3.mobs()[0].position.cell(), Some(CellId::new(3, 1)));
    for _ in 90..100 {
        nav2.step(&[]).unwrap();
        nav3.step(&[]).unwrap();
    }
    assert!(nav2.mobs().is_empty());
    assert_eq!(nav3.mobs().len(), 1);
}

#[test]
fn insufficient_fuse_does_not_authorize_the_threatened_transit_cell() {
    let mut game = state(MobNavigationPolicy::TimedEscapeV3, 50);

    for _ in 0..20 {
        game.step(&[]).unwrap();
    }

    assert_eq!(game.mobs()[0].position, Position::center(START));
}

#[test]
fn whole_footprint_clearance_on_the_explosion_tick_is_accepted() {
    let mut game = state(MobNavigationPolicy::TimedEscapeV3, 73);

    for _ in 0..73 {
        game.step(&[]).unwrap();
    }

    assert_eq!(game.mobs().len(), 1);
    assert_eq!(game.mobs()[0].position.cell(), Some(CellId::new(3, 1)));
}

fn world_with_bombs(bombs: impl IntoIterator<Item = Bomb>) -> World {
    let mut world = World::new(corridor_arena(BTreeSet::from([
        CellId::new(2, 3),
        CellId::new(4, 3),
    ])));
    for bomb in bombs {
        world.bombs.insert(bomb.id(), bomb);
    }
    world
}

fn open_world_with_bombs(bombs: impl IntoIterator<Item = Bomb>) -> World {
    let mut world = World::new(corridor_arena(BTreeSet::new()));
    for bomb in bombs {
        world.bombs.insert(bomb.id(), bomb);
    }
    world
}

fn timed_bomb(id: u32, cell: CellId, range: u16, deadline: u64) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        EntityId(10),
        range,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(deadline),
        },
    )
    .unwrap()
}

#[test]
fn early_chain_ignition_rejects_a_route_that_the_late_fuse_would_allow() {
    let chained = timed_bomb(2, CellId::new(3, 5), 3, 200);
    let trigger = timed_bomb(1, CellId::new(1, 5), 3, 50);
    let world = world_with_bombs([trigger, chained]);
    let mut budget = WorkBudget::new();
    let hazards = TimedMobHazards::build(&world, Tick(1), &mut budget).unwrap();

    assert_eq!(
        escape_direction(
            EscapeEnvironment {
                world: &world,
                spike_pits: None,
                hazards: &hazards,
                tick: Tick(1),
            },
            &mut budget,
            Position::center(START),
            0,
        ),
        None
    );
}

#[test]
fn unresolved_remote_hazard_withholds_the_timed_escape_exception() {
    let remote = Bomb::new(
        EntityId(1),
        CellId::new(3, 5),
        EntityId(10),
        3,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    let world = world_with_bombs([remote]);
    let mut budget = WorkBudget::new();
    assert!(TimedMobHazards::build(&world, Tick(1), &mut budget).is_none());

    assert_eq!(planned_direction(&world, Tick(1)), None);
}

#[test]
fn moving_timed_bomb_withholds_the_timed_escape_exception() {
    let mut moving = timed_bomb(1, CellId::new(5, 3), 1, 100);
    moving.arm_kick(Direction::West);
    let world = world_with_bombs([moving]);

    assert!(!super::explosions::bomb_danger_cells(&world).contains(&START));
    assert_eq!(planned_direction(&world, Tick(1)), None);
}

#[test]
fn removable_soft_block_cannot_hide_a_later_blast_expansion() {
    let target = CellId::new(3, 1);
    let arena = corridor_arena(BTreeSet::from([CellId::new(5, 1)]));
    let world = expansion_world(World::new(arena));

    assert_future_expansion_is_covered(world, target);
}

#[test]
fn removable_unlit_oil_cannot_hide_a_later_blast_expansion() {
    let target = CellId::new(3, 1);
    let arena = corridor_arena(BTreeSet::new());
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(5, 1), OilState::Unlit)]),
    )
    .unwrap();

    assert_future_expansion_is_covered(expansion_world(world), target);
}

fn expansion_world(mut world: World) -> World {
    for bomb in [
        timed_bomb(1, CellId::new(5, 3), 2, 50),
        timed_bomb(2, CellId::new(7, 1), 5, 200),
    ] {
        world.bombs.insert(bomb.id(), bomb);
    }
    world
}

fn assert_future_expansion_is_covered(world: World, target: CellId) {
    assert!(!super::explosions::bomb_danger_cells(&world).contains(&target));
    let mut budget = WorkBudget::new();
    let hazards = TimedMobHazards::build(&world, Tick(1), &mut budget).unwrap();
    assert_eq!(hazards.deadline(target), Some(Tick(200)));
}

#[test]
fn v3_routes_out_of_an_expansion_only_cell_before_the_block_is_removed() {
    let target = CellId::new(3, 1);
    let arena = corridor_arena(BTreeSet::from([CellId::new(5, 1)]));
    let world = expansion_world(World::new(arena));
    assert!(!super::explosions::bomb_danger_cells(&world).contains(&target));
    let spawns = world.arena().spawns().to_vec();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v9_with_mob_navigation(
        RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(60).unwrap()),
        MobNavigationPolicy::TimedEscapeV3,
    );
    let mut game = State::new(rules, world, actors).unwrap();
    game.insert_test_mob(MOB_ID, Position::center(target), Direction::West);

    game.step(&[]).unwrap();

    assert_eq!(game.mobs()[0].heading, Direction::South);
    assert!(game.mobs()[0].position.y > Position::center(target).y);
}

#[test]
fn route_turns_only_after_reaching_the_threatened_cell_center() {
    let mut game = state_with_range(MobNavigationPolicy::TimedEscapeV3, 150, 4);
    for _ in 0..79 {
        game.step(&[]).unwrap();
        assert_eq!(game.mobs()[0].heading, Direction::North);
    }

    game.step(&[]).unwrap();
    assert_eq!(game.mobs()[0].position, Position::center(CellId::new(3, 1)));
    game.step(&[]).unwrap();
    assert!(matches!(
        game.mobs()[0].heading,
        Direction::East | Direction::West
    ));
}

#[test]
fn newly_blocked_route_is_reaudited_without_crossing_the_bomb() {
    let mut game = state(MobNavigationPolicy::TimedEscapeV3, 140);
    for _ in 0..20 {
        game.step(&[]).unwrap();
    }
    game.insert_test_bomb(
        EntityId(92),
        CellId::new(3, 1),
        EntityId(10),
        1,
        game.tick(),
        BombTrigger::Timed {
            detonates_at: Tick(1_000),
        },
    )
    .unwrap();

    for _ in 0..50 {
        game.step(&[]).unwrap();
    }
    assert_ne!(game.mobs()[0].position.cell(), Some(CellId::new(3, 1)));
}

#[test]
fn changed_hazard_selects_a_different_refuge_on_the_next_audit() {
    let timed = timed_bomb(1, CellId::new(3, 5), 3, 100);
    let open = open_world_with_bombs([timed]);
    assert_eq!(planned_direction(&open, Tick(1)), Some(1));

    let timed = timed_bomb(1, CellId::new(3, 5), 3, 100);
    let blocker = Bomb::new(
        EntityId(2),
        CellId::new(4, 3),
        EntityId(10),
        0,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(1_000),
        },
    )
    .unwrap();
    let changed = open_world_with_bombs([timed, blocker]);
    assert_eq!(planned_direction(&changed, Tick(1)), Some(3));
}

fn planned_direction(world: &World, tick: Tick) -> Option<u8> {
    let mut budget = WorkBudget::new();
    let hazards = TimedMobHazards::build(world, tick, &mut budget)?;
    escape_direction(
        EscapeEnvironment {
            world,
            spike_pits: None,
            hazards: &hazards,
            tick,
        },
        &mut budget,
        Position::center(START),
        0,
    )
}

#[test]
fn stopwatch_freeze_preserves_position_and_thaw_replans_with_the_paused_fuse() {
    let mut game = state(MobNavigationPolicy::TimedEscapeV3, 100);
    game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));
    for _ in 0..179 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.mobs()[0].position, Position::center(START));

    for _ in 179..270 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.mobs()[0].position.cell(), Some(CellId::new(3, 1)));
}

#[test]
fn deadline_arithmetic_overflow_falls_back_without_a_route() {
    let bomb = timed_bomb(1, CellId::new(3, 5), 3, u64::MAX);
    let world = world_with_bombs([bomb]);

    assert_eq!(planned_direction(&world, Tick(u64::MAX - 10)), None);
}

#[test]
fn projection_work_exhaustion_returns_no_partial_hazard_model() {
    let bombs = (100..170).map(|id| {
        Bomb::new_with_blast(
            EntityId(id),
            CellId::new(7, 5),
            EntityId(10),
            50,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1_000),
            },
            BlastKind::Power,
        )
        .unwrap()
    });
    let world = world_with_bombs(bombs);
    let mut budget = WorkBudget::new();

    assert!(TimedMobHazards::build(&world, Tick(1), &mut budget).is_none());
}
