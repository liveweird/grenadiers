use std::sync::Arc;

use grenadiers_core::{
    cpu::WorldId,
    game::{EnabledItems, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, v16, v16::ResumeToken},
    room::{CpuProfile, LairSettings, LairTuning, MatchPhase, RoomSettings},
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError};
use crate::{
    game::rooms::{
        cpu::{CpuController, CpuPlanner},
        roster_runtime::tests::{configuration, identity},
    },
    round_seed::RoundSeedSource,
};

fn tuning(enabled: bool, seconds: u8, mobs: u8) -> LairTuning {
    LairTuning::new(enabled, LairSettings::new(seconds, mobs).unwrap())
}

fn runtime(initial: LairTuning) -> RosterRuntime {
    RosterRuntime::new_lairs(
        identity(configuration(2)),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        initial,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(41);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

fn admit(runtime: &mut RosterRuntime) {
    for seat in SeatId::DUEL {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
}

#[test]
fn lair_settings_are_one_atomic_revision_and_retain_disabled_numbers() {
    let initial = tuning(false, 17, 4);
    let mut runtime = runtime(initial);
    admit(&mut runtime);
    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    assert!(matches!(
        runtime.lifecycle.phase(),
        MatchPhase::Countdown { .. }
    ));

    let changed = tuning(false, 29, 7);
    let base = runtime.lifecycle.settings();
    let notice = runtime
        .update_lair_settings(SeatId::One, 1, 1, base, StormCloudConfig::Disabled, changed)
        .unwrap()
        .unwrap();
    assert_eq!(runtime.settings_revision, 2);
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    assert!(!runtime.lifecycle.ready(SeatId::One).unwrap());
    assert!(!runtime.lifecycle.ready(SeatId::Two).unwrap());
    assert_eq!(runtime.lair_tuning(), Some(changed));
    let room = notice.lair_room().unwrap();
    assert_eq!(room.rules_version.get(), 8);
    assert!(!room.settings.environment.storm_cloud);
    assert!(!room.settings.environment.lairs.enabled);
    assert_eq!(room.settings.environment.lairs.spawn_seconds, 29);
    assert_eq!(room.settings.environment.lairs.max_living_mobs, 7);

    let before = runtime.replay_inspection();
    assert_eq!(
        runtime.update_lair_settings(SeatId::One, 2, 1, base, StormCloudConfig::Disabled, changed,),
        Err(RosterRuntimeError::StaleSettings)
    );
    assert_eq!(runtime.replay_inspection(), before);
}

#[test]
fn lair_running_publication_and_admission_stay_rules_v8() {
    let enabled = tuning(true, 5, 2);
    let mut runtime = runtime(enabled);
    admit(&mut runtime);
    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    let mut notice = None;
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        notice = runtime.settle_boundary([None; 4]).unwrap().or(notice);
    }
    assert!(notice.unwrap().lair_cpu_publication().is_some());
    let sim = runtime.sim.as_ref().unwrap();
    assert_eq!(sim.state.rules().version(), 8);
    assert_eq!(sim.state.rules().lair_config(), enabled.config());

    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(prepared.lair_room().unwrap().rules_version.get(), 8);
    let encoded = prepared
        .encode(
            Counter::new(7).unwrap(),
            ResumeToken::new("a".repeat(64)).unwrap(),
        )
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["room"]["rulesVersion"], 8);
    assert!(value["state"]["lairs"].is_array());
    assert!(value["state"]["mobs"].is_array());
}

#[test]
fn queued_cpu19_worker_plans_the_native_v16_observation() {
    let mut runtime = runtime(tuning(true, 5, 2));
    admit(&mut runtime);
    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    let mut notice = None;
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        notice = runtime.settle_boundary([None; 4]).unwrap().or(notice);
    }
    let planner = CpuPlanner::without_workers();
    let mut controller = CpuController::new(
        planner.clone(),
        SeatId::Two,
        7,
        [19; 32],
        CpuProfile::LAIRS_STANDARD,
    );
    let world = WorldId {
        room_instance: 7,
        physical_round: 1,
    };
    let first = v16::observe_cpu_recipient(
        notice.unwrap().lair_cpu_publication().unwrap(),
        v16::Seat::Two,
        world,
    )
    .unwrap();
    assert!(controller.accept_observation(&first));
    controller.on_stop_admitted(first.parts().tick);

    runtime.settle_boundary([None; 4]).unwrap();
    let fresh_notice = runtime.publish_current().unwrap();
    let fresh = v16::observe_cpu_recipient(
        fresh_notice.lair_cpu_publication().unwrap(),
        v16::Seat::Two,
        world,
    )
    .unwrap();
    assert!(controller.accept_observation(&fresh));
    assert!(controller.submit(fresh, 1));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (1, 0));
    assert!(report.work[0] <= 65_536);
}
