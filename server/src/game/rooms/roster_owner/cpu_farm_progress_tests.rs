use std::sync::Arc;

use grenadiers_core::{
    cpu::{
        MovementIntent, WorldId,
        policy::{StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason, StrategicTarget},
    },
    game::{EnabledItems, Position, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, MatchPhase, RespawnLairSettings, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl,
    },
};
use tokio::{sync::watch, time::Instant};

use super::{RosterOwnerNotice, driver::Driver};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

#[derive(Debug)]
struct Result {
    attained: usize,
    longest_stationary: u64,
    moving_plans: usize,
    stalled_intents: usize,
    failed: usize,
    max_work: u32,
}

fn run(seed: u8, profile: CpuProfile, lairs: bool) -> Result {
    let configuration = RoomConfiguration::new(
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Cpu,
            SeatControl::Cpu,
        ],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_respawn(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: 9_025,
            configuration,
        },
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Disabled,
        RespawnLairTuning::new(lairs, RespawnLairSettings::DEFAULT),
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.fill(seed);
            Ok(())
        }),
        Arc::new(move |_| Ok(SimulationSeed::new([seed; 32]))),
    )
    .unwrap();
    let admission = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(admission);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Running);
    let planner = CpuPlanner::without_workers();
    let cpus = [SeatId::Two, SeatId::Three, SeatId::Four]
        .into_iter()
        .map(|seat| {
            CpuController::new(
                planner.clone(),
                seat,
                9_025,
                [seed.wrapping_add(seat.index() as u8); 32],
                profile,
            )
        })
        .collect();
    let notice = runtime.current_notice().unwrap();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(91);
        Ok(())
    });
    let mut driver = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        cpus,
    );
    driver.publish(notice);

    let mut attained = 0;
    let mut stationary: [(Option<StrategicTarget>, Position, u64); 3] =
        [(None, Position { x: 0, y: 0 }, 0); 3];
    let mut longest_stationary = 0;
    let mut moving_plans = 0;
    let mut stalled_intents = 0;
    let mut failed = 0;
    let mut max_work = 0;
    for _ in 0..1_200 {
        let report = planner.pump_for_test(usize::MAX);
        moving_plans += report
            .planned_intents
            .iter()
            .filter(|intent| intent.movement != MovementIntent::Stop)
            .count();
        failed += report.failed;
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        driver.tick(Instant::now()).unwrap();
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.unwrap().state;
        for (index, seat) in [SeatId::Two, SeatId::Three, SeatId::Four]
            .into_iter()
            .enumerate()
        {
            let actor = &state.actors()[seat.index()];
            let position = actor.position();
            let summary = driver.cpus[index].intent_debug_summary_for(WorldId {
                room_instance: 9_025,
                physical_round: 1,
            });
            stalled_intents += usize::from(summary.is_some_and(|summary| {
                summary.last_switch_reason == StrategicSwitchReason::Stalled
            }));
            let target = summary.and_then(|summary| {
                (summary.objective == StrategicObjectiveKind::Farm
                    && matches!(
                        summary.phase,
                        StrategicPhase::Approach | StrategicPhase::Reengage
                    )
                    && summary.approach_cell == position.cell())
                .then_some(summary.target)
            });
            if target.is_some_and(|target| matches!(target, StrategicTarget::Block { .. })) {
                attained += 1;
                if stationary[index].0 == target && stationary[index].1 == position {
                    stationary[index].2 += 1;
                } else {
                    stationary[index] = (target, position, 1);
                }
                longest_stationary = longest_stationary.max(stationary[index].2);
            } else {
                stationary[index] = (None, position, 0);
            }
        }
        if driver.runtime.lifecycle().phase() != MatchPhase::Running {
            break;
        }
    }
    Result {
        attained,
        longest_stationary,
        moving_plans,
        stalled_intents,
        failed,
        max_work,
    }
}

#[test]
fn queued_cpu25_repositions_after_an_attained_farming_target_with_default_items() {
    for lairs in [false, true] {
        let archived = run(3, CpuProfile::RESPAWN_V24_STANDARD, lairs);
        assert!(
            archived.longest_stationary >= 180,
            "fixture no longer reproduces the archived CPU24 stall: {archived:?}"
        );

        let current = run(3, CpuProfile::RESPAWN_LAIRS_STANDARD, lairs);
        assert_eq!(current.failed, 0, "lairs={lairs}: {current:?}");
        assert!(current.attained > 0, "lairs={lairs}: {current:?}");
        assert!(current.moving_plans > 0, "lairs={lairs}: {current:?}");
        assert!(current.stalled_intents > 0, "lairs={lairs}: {current:?}");
        assert!(
            current.longest_stationary < 60,
            "lairs={lairs}: farming approach stayed stationary: {current:?}"
        );
        assert!(
            current.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
            "lairs={lairs}: {current:?}"
        );
    }
}
