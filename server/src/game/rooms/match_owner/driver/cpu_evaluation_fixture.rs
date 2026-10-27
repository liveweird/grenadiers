use std::sync::Arc;

use grenadiers_core::game::{EnabledItems, RulesConfig, SeatId};
use grenadiers_core::room::{CpuProfile, MatchPhase, RoomMode, RoomSettings};
use tokio::{sync::watch, time::Instant};

use super::super::super::super::super::match_runtime::{MatchAction, MatchRuntime};
use super::super::super::super::super::profile::RoomProfile;
use super::super::super::super::super::{
    RandomFill,
    cpu::{CpuController, CpuPlanner},
};
use super::super::super::super::{MatchOwnerNotice, test_support::identity};
use crate::round_seed::RoundSeedSource;

use super::super::super::Driver;
use super::{EvaluationCase, HumanScript};

pub(super) fn default_cases() -> Vec<EvaluationCase> {
    const IDS: [&str; 16] = [
        "a0-c0-idle",
        "a0-c0-retreat",
        "a0-c1-idle",
        "a0-c1-retreat",
        "a1-c0-idle",
        "a1-c0-retreat",
        "a1-c1-idle",
        "a1-c1-retreat",
        "a2-c0-idle",
        "a2-c0-retreat",
        "a2-c1-idle",
        "a2-c1-retreat",
        "a3-c0-idle",
        "a3-c0-retreat",
        "a3-c1-idle",
        "a3-c1-retreat",
    ];
    let arenas = [
        ("a0", [11; 32]),
        ("a1", [37; 32]),
        ("a2", [73; 32]),
        ("a3", [109; 32]),
    ];
    let cpus = [("c0", [17; 32]), ("c1", [151; 32])];
    let scripts = [HumanScript::Idle, HumanScript::BombRetreat];
    let mut cases = Vec::with_capacity(16);
    let mut index = 0;
    for (arena_id, arena_seed) in arenas {
        for (cpu_id, cpu_seed) in cpus {
            for script in scripts {
                cases.push(EvaluationCase::new(
                    IDS[index], arena_id, arena_seed, cpu_id, cpu_seed, script, 1_800,
                ));
                index += 1;
            }
        }
    }
    cases
}

pub(super) fn full_round_cases() -> Vec<EvaluationCase> {
    let cap = RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL))
        .round_ticks();
    default_cases()
        .into_iter()
        .map(|case| EvaluationCase {
            tick_cap: cap,
            ..case
        })
        .collect()
}

pub(super) fn running_recording_driver(
    arena_seed: [u8; 32],
    cpu_seed: [u8; 32],
    profile: CpuProfile,
) -> (Driver, CpuPlanner, RoomSettings) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let mut room = identity(RoomProfile::ExpandedV5);
    room.mode = RoomMode::Solo(profile);
    room.private_cpu_seed = Some(cpu_seed);
    let planner = CpuPlanner::without_workers();
    let cpu = Some(CpuController::new(
        planner.clone(),
        grenadiers_core::game::SeatId::Two,
        room.registry_serial,
        cpu_seed,
        profile,
    ));
    let mut runtime = MatchRuntime::new_recording(
        room,
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.copy_from_slice(&arena_seed);
            Ok(())
        }),
    );
    let initial = runtime.initial_notice().unwrap();
    let (notices, _) = watch::channel(MatchOwnerNotice::Publication(initial.clone()));
    let random_fill: Arc<RandomFill> = Arc::new(|bytes: &mut [u8]| {
        bytes.fill(31);
        Ok(())
    });
    let mut driver = Driver::new(runtime, initial, notices, Instant::now(), random_fill, cpu);
    let creator = driver.runtime.prepare_solo_creator(SeatId::One).unwrap();
    driver.runtime.commit_admission(creator);
    driver
        .runtime
        .action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .unwrap();
    driver
        .runtime
        .cpu_ready(grenadiers_core::game::SeatId::Two)
        .unwrap();
    let settings_revision = driver.runtime.replay_inspection().settings_revision;
    driver
        .runtime
        .action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision,
            },
        )
        .unwrap();
    while driver.runtime.lifecycle().phase() != MatchPhase::Running {
        driver.runtime.settle_boundary([None; 2]).unwrap();
    }
    (driver, planner, settings)
}
