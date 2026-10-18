use grenadiers_core::{
    game::{BlastKind, CellId, Direction, EnabledItems, SeatId, SpecialWeapon},
    room::MatchPhase,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

#[test]
fn generated_piercing_charge_survives_resume_and_destroys_two_blocks_before_standard_reuse() {
    let mut runtime = start_optional_runtime(EnabledItems::PIERCING_BOMB, 31);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Piercing)
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Piercing)
    );

    plant(&mut runtime, 2);
    let bomb = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(bomb.blast(), BlastKind::Piercing);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        None
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    let blocks = runtime.sim().unwrap().state.world().arena().blocks();
    assert!(!blocks.contains(&CellId::new(4, 1)));
    assert!(!blocks.contains(&CellId::new(5, 1)));
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    plant(&mut runtime, 3);
    assert_eq!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .values()
            .next()
            .unwrap()
            .blast(),
        BlastKind::Standard
    );
    for _ in 0..=180 {
        if !runtime.sim().unwrap().state.actors()[0].alive() {
            break;
        }
        runtime.tick().unwrap();
    }
    assert!(!runtime.sim().unwrap().state.actors()[0].alive());
    for _ in 0..1000 {
        if runtime.lifecycle().round_attempt() == 2
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            break;
        }
        runtime.tick().unwrap();
    }
    assert_eq!(runtime.lifecycle().round_attempt(), 2);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        None
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}
