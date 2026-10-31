use std::collections::BTreeSet;
use std::sync::Arc;

use grenadiers_core::{
    cpu::ManualIntent,
    game::{EnabledItems, MobNavigationPolicy, SeatId, SimulationSeed, StormCloudConfig},
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
fn queued_cpu23_uses_bombs_collects_upgrades_and_avoids_three_ab_cycles() {
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
    let mut runtime = RosterRuntime::new_respawn_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: 9_023,
            configuration,
        },
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::GOLDEN_FLAME),
        StormCloudConfig::Single,
        RespawnLairTuning::new(true, RespawnLairSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(3);
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
    let seats = [SeatId::Two, SeatId::Three, SeatId::Four];
    let cpus = seats
        .into_iter()
        .map(|seat| {
            CpuController::new(
                planner.clone(),
                seat,
                9_023,
                [53_u8.wrapping_add(seat.index() as u8); 32],
                CpuProfile::RESPAWN_V23_STANDARD,
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
    let initial = &driver.runtime.replay_inspection().simulation.unwrap().state;
    let initial_blocks = initial.world().arena().blocks().len();
    let mut histories =
        seats.map(|seat| vec![initial.actors()[seat.index()].position().cell().unwrap()]);
    let mut delivered = 0;
    let mut failed = 0;
    let mut bomb_plans = 0;
    let mut max_work = 0;
    let mut collected_upgrade = false;
    let mut collected_at = [None; 3];
    let mut moved_after_collection = [false; 3];
    let mut final_blocks = initial_blocks;
    let mut physical_bombs = BTreeSet::new();
    let mut alive_cpu_ticks = 0_u64;

    for _ in 0..1_200_u64 {
        let report = planner.pump_for_test(usize::MAX);
        delivered += report.delivered;
        failed += report.failed;
        bomb_plans += report
            .planned_intents
            .iter()
            .filter(|intent| intent.manual == Some(ManualIntent::Bomb))
            .count();
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        driver.tick(Instant::now()).unwrap();
        let state = &driver.runtime.replay_inspection().simulation.unwrap().state;
        final_blocks = state.world().arena().blocks().len();
        physical_bombs.extend(state.world().bombs().keys().copied());
        for (index, seat) in seats.into_iter().enumerate() {
            let actor = &state.actors()[seat.index()];
            alive_cpu_ticks += u64::from(actor.alive());
            collected_upgrade |= actor.effective_bomb_range(state.tick()) > 2;
            if actor.effective_bomb_range(state.tick()) > 2 && collected_at[index].is_none() {
                collected_at[index] = Some(actor.position());
            }
            moved_after_collection[index] |=
                collected_at[index].is_some_and(|position| position != actor.position());
            let Some(cell) = actor.position().cell() else {
                continue;
            };
            if histories[index].last() != Some(&cell) {
                histories[index].push(cell);
            }
        }
        if driver.runtime.lifecycle().phase() != MatchPhase::Running {
            break;
        }
    }

    assert_eq!(failed, 0);
    assert!(delivered > 0);
    assert!(
        bomb_plans >= 5,
        "delivered={delivered} max_work={max_work} collected={collected_upgrade} histories={histories:?}"
    );
    assert!(final_blocks <= initial_blocks - 5);
    assert!(collected_upgrade);
    assert!(moved_after_collection.into_iter().any(|moved| moved));
    assert!(physical_bombs.len() >= 5);
    assert!(alive_cpu_ticks > 0);
    assert!(max_work <= 65_536);
    assert!(
        histories.iter().all(|cells| !has_three_ab_cycles(cells)),
        "{histories:?}"
    );
}

fn has_three_ab_cycles(cells: &[grenadiers_core::game::CellId]) -> bool {
    cells.windows(6).any(|window| {
        window[0] == window[2]
            && window[2] == window[4]
            && window[1] == window[3]
            && window[3] == window[5]
    })
}
