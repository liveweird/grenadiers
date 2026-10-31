use super::{ActorInit, State};
use crate::game::{
    BombTrigger, CellId, Direction, EntityId, MobNavigationPolicy, Position, RulesConfig, SeatId,
    SpeedLevel, StormCloudConfig, Terrain, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings};

fn state(policy: MobNavigationPolicy) -> State {
    let arena = fixed_arena();
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
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        policy,
    );
    let mut state = State::new(rules, World::new(arena), actors).unwrap();
    let blocks = state.world.arena.blocks().clone();
    state.world.arena.remove_blocks(&blocks);
    state
}

fn run_danger_episode(policy: MobNavigationPolicy) -> (CellId, Vec<Position>) {
    let mut game = state(policy);
    while game.tick().0 < 300 {
        game.step(&[]).unwrap();
    }
    let mob = game.mobs()[0];
    let origin = mob.position.cell().unwrap();
    let candidates = [
        CellId::new(origin.column, origin.row - 1),
        CellId::new(origin.column + 1, origin.row),
        CellId::new(origin.column, origin.row + 1),
        CellId::new(origin.column - 1, origin.row),
    ];
    let bomb = candidates
        .into_iter()
        .find(|&cell| game.world().arena().terrain(cell) == Some(Terrain::Floor))
        .unwrap();
    game.insert_test_bomb(
        EntityId(90),
        bomb,
        EntityId(10),
        1,
        game.tick(),
        BombTrigger::Timed {
            detonates_at: Tick(10_000),
        },
    )
    .unwrap();
    assert!(crate::game::explosions::bomb_danger_cells(game.world()).contains(&origin));

    let mut positions = Vec::new();
    for _ in 0..80 {
        game.step(&[]).unwrap();
        positions.push(
            game.mobs()
                .into_iter()
                .find(|view| view.id == mob.id)
                .unwrap()
                .position,
        );
    }
    assert!(game.world().bombs().contains_key(&EntityId(90)));
    (origin, positions)
}

#[test]
fn state_escape_v2_physically_leaves_persistent_center_danger() {
    let (origin, positions) = run_danger_episode(MobNavigationPolicy::EscapeV2);

    assert!(
        positions
            .iter()
            .skip(19)
            .any(|position| position.cell() != Some(origin))
    );
}

#[test]
fn state_legacy_v1_retains_its_center_oscillation_cycle() {
    let (origin, positions) = run_danger_episode(MobNavigationPolicy::LegacyV1);

    assert!(
        positions
            .iter()
            .all(|position| position.cell() == Some(origin))
    );
    assert!(positions.windows(4).any(|window| {
        window[0] == window[2] && window[1] == window[3] && window[0] != window[1]
    }));
}
