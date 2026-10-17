use std::time::Instant;

use grenadiers_core::{
    game::{Direction, SeatId},
    protocol::v5::RoomPhase,
    room::MatchPhase,
};
use serde_json::json;

use super::{MatchAction, test_support::*};
use crate::{game::rooms::measurement, round_seed::RoundSeedSource};

#[test]
#[ignore = "optimized reference measurement; run through scripts/measure-game.sh"]
fn runtime_reference_measurement() {
    measurement::require_release();
    let source = RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(0x5a);
        Ok(())
    });
    let mut runtime = generated_runtime(source);
    start_first_round(&mut runtime);
    let epoch = runtime.lifecycle().round_attempt();
    for seat in SeatId::ALL {
        runtime
            .action(seat, epoch, MatchAction::Movement(Some(wallward(seat))))
            .unwrap();
    }
    for tick in 0_usize..30 {
        if tick.is_multiple_of(10) {
            refresh_wallward(&mut runtime);
        }
        runtime.settle_boundary([None; 2]).unwrap();
    }
    for seat in SeatId::ALL {
        runtime.action(seat, epoch, MatchAction::Bomb(1)).unwrap();
    }

    let mut ordinary = Vec::with_capacity(400);
    let mut publishing = Vec::with_capacity(200);
    let mut transitions = Vec::with_capacity(8);
    let mut max_frame_bytes = 0;
    let mut measured_ticks = 0_u64;
    loop {
        if runtime.lifecycle().phase() == MatchPhase::Running && measured_ticks.is_multiple_of(10) {
            refresh_wallward(&mut runtime);
        }
        let phase_before = runtime.lifecycle().phase();
        let round_before = runtime.lifecycle().series().actual_round_number();
        let started = Instant::now();
        let notice = runtime.settle_boundary([None; 2]).unwrap();
        let elapsed = started.elapsed();
        measured_ticks += 1;
        if let Some(notice) = &notice {
            publishing.push(elapsed);
            max_frame_bytes = max_frame_bytes.max(notice.snapshot.as_ref().map_or(0, |s| s.len()));
        } else {
            ordinary.push(elapsed);
        }
        if phase_before != runtime.lifecycle().phase()
            || round_before != runtime.lifecycle().series().actual_round_number()
        {
            transitions.push(elapsed);
        }
        if runtime.lifecycle().series().actual_round_number() == 2
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            break;
        }
        assert!(
            measured_ticks < 1_000,
            "second generated round did not start"
        );
    }

    assert_eq!(runtime.lifecycle().series().scores(), [0, 0]);
    assert_eq!(runtime.lifecycle().series().actual_round_number(), 2);
    assert!(matches!(
        runtime.current_notice().unwrap().room.phase,
        RoomPhase::Running
    ));
    let all = ordinary
        .iter()
        .chain(&publishing)
        .copied()
        .collect::<Vec<_>>();
    let p99 = measurement::percentile_duration(&all, 99);
    measurement::emit(
        "runtime",
        json!({
            "workload": "one generated room; draw by two spawn bombs; next-round construction",
            "budgetP99Us": measurement::RUNTIME_P99_BUDGET.as_micros(),
            "allTicks": measurement::summary(&all),
            "ordinaryTicks": measurement::summary(&ordinary),
            "publishingTicks": measurement::summary(&publishing),
            "phaseOrRoundTransitions": measurement::summary(&transitions),
            "maxEncodedSnapshotBytes": max_frame_bytes,
            "finalTick": runtime.lifecycle().tick().0,
        }),
    );
    assert!(
        p99 <= measurement::RUNTIME_P99_BUDGET,
        "runtime p99 {p99:?} exceeds {:?}",
        measurement::RUNTIME_P99_BUDGET
    );
}

fn wallward(seat: SeatId) -> Direction {
    match seat {
        SeatId::One => Direction::North,
        SeatId::Two => Direction::South,
    }
}

fn refresh_wallward(runtime: &mut super::MatchRuntime) {
    let epoch = runtime.lifecycle().round_attempt();
    for seat in SeatId::ALL {
        runtime
            .action(seat, epoch, MatchAction::Movement(Some(wallward(seat))))
            .unwrap();
    }
}
