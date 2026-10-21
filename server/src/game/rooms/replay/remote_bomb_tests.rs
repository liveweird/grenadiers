use grenadiers_core::game::{
    BlastKind, BombTrigger, CellId, Direction, EnabledItems, SeatId, SpecialWeapon,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

#[test]
fn generated_remote_charge_survives_resume_and_replay_until_manual_detonation() {
    let mut runtime = start_optional_runtime(EnabledItems::REMOTE_BOMB, 31);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Remote)
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);

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
    assert_eq!(bomb.cell(), CellId::new(1, 2));
    assert_eq!(bomb.trigger(), BombTrigger::Remote);
    assert_eq!(bomb.blast(), BlastKind::Standard);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        None
    );
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    for _ in 0..=183 {
        runtime.tick().unwrap();
    }
    assert_eq!(
        runtime.sim().unwrap().state.world().bombs().get(&bomb.id()),
        Some(&bomb)
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    plant(&mut runtime, 3);
    assert!(
        !runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .contains_key(&bomb.id())
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    wait_clear(&mut runtime);
    plant(&mut runtime, 4);
    let ordinary = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert!(matches!(ordinary.trigger(), BombTrigger::Timed { .. }));
    assert_eq!(ordinary.blast(), BlastKind::Standard);
}
