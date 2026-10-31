use std::sync::Arc;

use grenadiers_core::{
    game::{MobNavigationPolicy, SeatId, SimulationSeed, StormCloudConfig},
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

#[test]
fn queued_cpu24_plans_with_bounded_work_and_an_explicit_intent() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_respawn_with_navigation(
        RosterIdentity {
            id: RoomId::new("1123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("BBCDEFGH").unwrap(),
            registry_serial: 9_024,
            configuration,
        },
        RoomSettings::DEFAULT,
        StormCloudConfig::Single,
        RespawnLairTuning::new(true, RespawnLairSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(24);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        MobNavigationPolicy::EscapeV2,
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
    let controller = CpuController::new(
        planner.clone(),
        SeatId::Two,
        9_024,
        [24; 32],
        CpuProfile::RESPAWN_V24_STANDARD,
    );
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
        vec![controller],
    );
    driver.publish(notice);

    let mut delivered = 0;
    let mut failed = 0;
    let mut max_work = 0;
    let mut saw_intent = false;
    let mut history = Vec::new();
    for _ in 0..1_200 {
        let report = planner.pump_for_test(usize::MAX);
        delivered += report.delivered;
        failed += report.failed;
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        driver.tick(Instant::now()).unwrap();
        if let Some(summary) =
            driver.cpus[0].intent_debug_summary_for(grenadiers_core::cpu::WorldId {
                room_instance: 9_024,
                physical_round: 1,
            })
        {
            saw_intent = true;
            let state = (summary.target, summary.phase, summary.last_switch_reason);
            if history.last() != Some(&state) {
                history.push(state);
            }
        }
    }

    eprintln!(
        "cpu24 queued metrics: delivered={delivered} failed={failed} max_work={max_work} saw_intent={saw_intent} history={history:?}"
    );

    assert_eq!(failed, 0);
    assert!(delivered > 0);
    assert!(saw_intent);
    assert!(history.len() >= 3, "history={history:?}");
    assert!(history.iter().any(|(target, _, _)| matches!(
        target,
        grenadiers_core::cpu::policy::StrategicTarget::Block { .. }
    )));
    assert!(history.iter().any(|(target, _, _)| matches!(
        target,
        grenadiers_core::cpu::policy::StrategicTarget::Pickup { .. }
    )));
    assert!(max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
}
