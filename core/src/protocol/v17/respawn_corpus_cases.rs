use super::server_conformance_tests::{FrameCase, snapshot_cases};
use super::server_test_support::{WorldCase, world_case};
use super::{Seat, capture_expanded};
use crate::game::{CellId, PlayerCount, StormCloudConfig, Tick};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings, RespawnLairTuning};

fn scene(advance: usize, seed: u8) -> WorldCase {
    world_case(
        BoardSize::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        advance,
        (false, false),
        [seed; 32],
    )
}

fn step(case: &mut WorldCase) {
    let report = case.state.step(&[]).unwrap();
    assert!(case.state.result().is_none());
    case.lifecycle.complete_running_tick(None).unwrap();
    case.capture = capture_expanded(&case.state, &[report]).unwrap();
    case.room = super::server_conformance_lifecycle::room_view_with_tuning(
        &case.lifecycle,
        StormCloudConfig::Disabled,
        RespawnLairTuning::new(true, RespawnLairSettings::new(5).unwrap()),
        true,
    );
}

fn kill(case: &mut WorldCase, lair: u8) {
    let mob = case
        .state
        .mobs()
        .into_iter()
        .find(|mob| mob.lair == lair)
        .unwrap();
    case.state.insert_test_flame(
        mob.position.cell().unwrap(),
        Tick(case.state.tick().0 + 100),
    );
    step(case);
}

pub(super) fn cases() -> Vec<FrameCase> {
    let mut cases = Vec::new();
    let mut independent = scene(300, 18);
    assert_eq!(independent.state.mobs().len(), 2);
    kill(&mut independent, 0);
    cases.extend(snapshot_cases(
        &independent,
        "oneLairCoolingOtherOccupied",
        &[Seat::One],
    ));
    for _ in 0..9 {
        step(&mut independent);
    }
    kill(&mut independent, 1);
    let statuses = independent.state.lair_statuses().unwrap();
    assert_ne!(statuses[0].next_spawn_at, statuses[1].next_spawn_at);
    cases.extend(snapshot_cases(
        &independent,
        "independentDeathCooldowns",
        &[Seat::One, Seat::Two],
    ));

    let mut blocked = scene(299, 16);
    let expires = Tick(blocked.state.tick().0 + 3);
    for lair in crate::game::lair_layout_v1(BoardSize::ElevenByNine) {
        for (dc, dr) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            blocked.state.insert_test_flame(
                CellId::new(
                    (i32::from(lair.column) + dc) as u16,
                    (i32::from(lair.row) + dr) as u16,
                ),
                expires,
            );
        }
    }
    step(&mut blocked);
    assert!(blocked.state.mobs().is_empty());
    assert!(
        blocked
            .capture
            .lairs
            .iter()
            .all(|lair| lair.spawn_remaining_ticks == Some(0))
    );
    cases.extend(snapshot_cases(
        &blocked,
        "blockedEligibleLairs",
        &[Seat::One],
    ));
    step(&mut blocked);
    step(&mut blocked);
    assert_eq!(blocked.state.mobs().len(), 2);
    cases.extend(snapshot_cases(
        &blocked,
        "spawnOnFirstClearTick",
        &[Seat::One],
    ));
    cases
}
