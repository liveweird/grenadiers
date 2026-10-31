use std::sync::Arc;

use grenadiers_core::{
    cpu::{
        CpuRng, MemoryKey, WorkBudget, forecast::ObservedHazardMemory, policy::UtilityMemory,
        survival::plan_respawn,
    },
    game::{EnabledItems, Position, SeatId, SimulationSeed, StormCloudConfig, Tick},
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
struct Snapshot {
    tick: Tick,
    alive: [bool; 3],
    last_moves: [Tick; 3],
}

struct Episode {
    driver: Driver,
    planner: CpuPlanner,
    seats: [SeatId; 3],
    last_positions: [Position; 3],
    last_moves: [Tick; 3],
    delivered: usize,
    failed: usize,
    moving_plans: usize,
    stop_plans: usize,
    max_work: u32,
    collected_at: [Option<(Tick, Position)>; 3],
    moved_after_collection: [bool; 3],
}

impl Episode {
    fn new(arena_seed: u8, cpu_seed: u8, profile: CpuProfile, lairs: bool) -> Self {
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
                registry_serial: 9_021,
                configuration,
            },
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::GOLDEN_FLAME),
            if lairs {
                StormCloudConfig::Single
            } else {
                StormCloudConfig::Disabled
            },
            RespawnLairTuning::new(lairs, RespawnLairSettings::DEFAULT),
            RoundSeedSource::with_entropy(move |bytes| {
                bytes.fill(arena_seed);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
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
                    9_021,
                    [cpu_seed.wrapping_add(seat.index() as u8); 32],
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
        let state = &driver.runtime.replay_inspection().simulation.unwrap().state;
        let last_positions = seats.map(|seat| state.actors()[seat.index()].position());
        let tick = state.tick();
        Self {
            driver,
            planner,
            seats,
            last_positions,
            last_moves: [tick; 3],
            delivered: 0,
            failed: 0,
            moving_plans: 0,
            stop_plans: 0,
            max_work: 0,
            collected_at: [None; 3],
            moved_after_collection: [false; 3],
        }
    }

    fn tick(&mut self) {
        let report = self.planner.pump_for_test(usize::MAX);
        self.delivered += report.delivered;
        self.failed += report.failed;
        self.moving_plans += report
            .planned_intents
            .iter()
            .filter(|intent| intent.movement != grenadiers_core::cpu::MovementIntent::Stop)
            .count();
        self.stop_plans += report
            .planned_intents
            .iter()
            .filter(|intent| intent.movement == grenadiers_core::cpu::MovementIntent::Stop)
            .count();
        self.max_work = self
            .max_work
            .max(report.work.into_iter().max().unwrap_or(0));
        self.driver.tick(Instant::now()).unwrap();
        let state = &self
            .driver
            .runtime
            .replay_inspection()
            .simulation
            .unwrap()
            .state;
        for (index, seat) in self.seats.into_iter().enumerate() {
            let position = state.actors()[seat.index()].position();
            if position != self.last_positions[index] {
                self.last_positions[index] = position;
                self.last_moves[index] = state.tick();
            }
            let actor = &state.actors()[seat.index()];
            if self.collected_at[index].is_none() && actor.effective_bomb_range(state.tick()) > 2 {
                self.collected_at[index] = Some((state.tick(), position));
            }
            if self.collected_at[index].is_some_and(|(_, collected)| collected != position) {
                self.moved_after_collection[index] = true;
            }
        }
    }

    fn snapshot(&self) -> Snapshot {
        let inspection = self.driver.runtime.replay_inspection();
        let state = &inspection.simulation.unwrap().state;
        Snapshot {
            tick: state.tick(),
            alive: self.seats.map(|seat| state.actors()[seat.index()].alive()),
            last_moves: self.last_moves,
        }
    }

    fn diagnosis(&self, index: usize) -> String {
        let observation = self.driver.cpus[index]
            .latest_observation_for_test()
            .unwrap();
        let key = MemoryKey {
            world: observation.parts().world,
            cpu_seat: observation.parts().recipient,
        };
        let mut history = ObservedHazardMemory::new(key);
        history.observe(&observation).unwrap();
        let mut work = WorkBudget::new(65_536);
        let plan = plan_respawn(
            &observation,
            &UtilityMemory::new(key),
            &history,
            observation.parts().tick,
            &mut CpuRng::from_seed(key, [53; 32]),
            &mut work,
        )
        .unwrap();
        format!(
            "pickups={:?} plan={:?} lease={} execution={} contact={} work={}",
            observation.parts().pickups,
            plan.survival.control.intent,
            plan.survival.lease.is_some(),
            plan.survival.execution.is_some(),
            plan.contact.is_some(),
            work.spent(),
        )
    }
}

fn run(profile: CpuProfile, lairs: bool) -> Episode {
    let mut episode = Episode::new(3, 53, profile, lairs);
    for _ in 0..3_600 {
        episode.tick();
        if episode.driver.runtime.lifecycle().phase() != MatchPhase::Running {
            break;
        }
    }
    episode
}

#[test]
fn cpu22_collects_golden_flame_and_continues_after_cpu21_stalls() {
    let old = run(CpuProfile::RESPAWN_V21_STANDARD, true);
    let old_snapshot = old.snapshot();
    let stalled = old_snapshot
        .alive
        .iter()
        .enumerate()
        .find(|(index, alive)| {
            **alive
                && old_snapshot
                    .tick
                    .0
                    .saturating_sub(old_snapshot.last_moves[*index].0)
                    > 300
        });
    let (stalled_index, _) = stalled.expect("seed 3 must preserve the published CPU21 stall");
    let diagnosis = old.diagnosis(stalled_index);
    assert!(
        diagnosis.contains("lease=false execution=false contact=true"),
        "{diagnosis}"
    );

    for lairs in [true, false] {
        let fixed = run(CpuProfile::RESPAWN_LAIRS_STANDARD, lairs);
        assert_eq!(fixed.failed, 0, "lairs={lairs} {:#?}", fixed.snapshot());
        assert!(
            fixed.collected_at.iter().any(Option::is_some),
            "lairs={lairs} no Golden Flame collected: {:#?}",
            fixed.snapshot()
        );
        assert!(
            fixed.moved_after_collection.iter().any(|moved| *moved),
            "lairs={lairs} no movement after collection: {:#?}",
            fixed.snapshot()
        );
        let snapshot = fixed.snapshot();
        assert!(
            snapshot.alive.iter().enumerate().all(|(index, alive)| {
                !*alive || snapshot.tick.0.saturating_sub(snapshot.last_moves[index].0) <= 300
            }),
            "lairs={lairs} surviving CPU stopped progressing: {snapshot:#?}"
        );
    }
}
