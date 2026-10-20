use grenadiers_core::game::{
    BlastKind, CellId, Direction, EnabledItems, SeatId, SpecialWeapon, Terrain,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

#[test]
fn generated_power_charge_survives_resume_replays_and_returns_to_standard() {
    let mut runtime = start_optional_runtime(EnabledItems::POWER_BOMB, 31);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Power)
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Power)
    );

    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
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
    assert_eq!(
        (bomb.cell(), bomb.range(), bomb.blast()),
        (CellId::new(1, 2), 2, BlastKind::Power)
    );
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        None
    );
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    for _ in 0..=180 {
        runtime.tick().unwrap();
        if runtime
            .sim()
            .unwrap()
            .state
            .world()
            .hazards()
            .contains_key(&CellId::new(3, 2))
        {
            break;
        }
    }
    let world = runtime.sim().unwrap().state.world();
    assert!(world.hazards().contains_key(&CellId::new(3, 2)));
    assert!(!world.hazards().contains_key(&CellId::new(2, 2)));
    assert!(!world.hazards().contains_key(&CellId::new(3, 1)));
    assert!(!world.arena().blocks().contains(&CellId::new(3, 2)));
    assert_eq!(
        world.arena().terrain(CellId::new(2, 2)),
        Some(Terrain::Wall)
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    wait_clear(&mut runtime);
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
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}
