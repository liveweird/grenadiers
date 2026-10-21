use grenadiers_core::{
    game::{CellId, Direction, EffectKind, EnabledItems, SeatId},
    protocol::Counter,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

#[test]
fn stopwatch_freeze_action_floor_thaw_and_natural_bomb_replay_deterministically() {
    let mut runtime = start_optional_runtime(EnabledItems::STOPWATCH, 31);
    collect_generated_pickup(&mut runtime);
    assert!(
        runtime.sim().unwrap().state.actors()[0]
            .effects()
            .active(EffectKind::Stopwatch, runtime.sim().unwrap().state.tick())
    );

    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    plant(&mut runtime, 2);
    let bomb = *runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);

    let epoch = runtime.lifecycle().round_attempt();
    let frozen_start = runtime.sim().unwrap().state.actors()[1].position();
    runtime
        .action(
            SeatId::Two,
            epoch,
            MatchAction::Movement(Some(Direction::South)),
        )
        .unwrap();
    runtime
        .action(SeatId::Two, epoch, MatchAction::Bomb(1))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(
        runtime.sim().unwrap().state.actors()[1].position(),
        frozen_start
    );
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    let prepared = runtime.prepare_resume(SeatId::Two).unwrap();
    assert_eq!(prepared.last_bomb_action_id, Some(Counter::new(1).unwrap()));
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    let mut frozen_ticks = 0;
    while runtime.sim().unwrap().state.voluntary_actions_frozen()[1] {
        if frozen_ticks % 10 == 0 {
            runtime
                .action(
                    SeatId::Two,
                    epoch,
                    MatchAction::Movement(Some(Direction::South)),
                )
                .unwrap();
        }
        runtime.tick().unwrap();
        frozen_ticks += 1;
    }
    runtime.tick().unwrap();
    assert_ne!(
        runtime.sim().unwrap().state.actors()[1].position(),
        frozen_start
    );
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    runtime
        .action(SeatId::Two, epoch, MatchAction::Movement(None))
        .unwrap();
    while runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .contains_key(&bomb.id())
    {
        runtime.tick().unwrap();
    }
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .hazards()
            .contains_key(&CellId::new(1, 2))
    );
    wait_clear(&mut runtime);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}
