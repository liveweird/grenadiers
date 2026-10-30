use std::sync::Arc;

use grenadiers_core::{
    cpu::survival::WORK_ALLOWANCE,
    game::{EnabledItems, Position, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, LairSettings, LairTuning, MatchPhase,
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

#[derive(Clone, Copy, Debug)]
struct Case {
    board: BoardSize,
    count: usize,
    difficulty: CpuDifficulty,
    cloud: StormCloudConfig,
    lairs: bool,
}

struct Episode {
    driver: Driver,
    planner: CpuPlanner,
    seats: Vec<SeatId>,
    initial: Vec<Position>,
    moved: Vec<bool>,
    delivered: usize,
    moving_plans: usize,
    max_work: u32,
    planted: bool,
    survived_detonation: bool,
}

impl Episode {
    fn new(case: Case) -> Self {
        let controls = std::iter::once(SeatControl::Human)
            .chain(std::iter::repeat_n(SeatControl::Cpu, case.count - 1))
            .collect::<Vec<_>>();
        let configuration =
            RoomConfiguration::new(&controls, Some(case.difficulty), case.board).unwrap();
        let tuning = LairTuning::new(case.lairs, LairSettings::new(5, 2).unwrap());
        let mut runtime = RosterRuntime::new_lairs(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 3117,
                configuration,
            },
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
            case.cloud,
            tuning,
            RoundSeedSource::with_entropy(move |bytes| {
                bytes.fill(17 + case.count as u8 + case.board.width() as u8);
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
        let profile = match case.difficulty {
            CpuDifficulty::Easy => CpuProfile::LAIRS_EASY,
            CpuDifficulty::Standard => CpuProfile::LAIRS_STANDARD,
        };
        let seats = configuration.cpu_seats().collect::<Vec<_>>();
        let cpus = seats
            .iter()
            .copied()
            .map(|seat| CpuController::new(planner.clone(), seat, 3117, [53; 32], profile))
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
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let initial = seats
            .iter()
            .map(|seat| state.actors()[seat.index()].position())
            .collect::<Vec<_>>();
        let cpu_count = seats.len();
        Self {
            driver,
            planner,
            seats,
            initial,
            moved: vec![false; cpu_count],
            delivered: 0,
            moving_plans: 0,
            max_work: 0,
            planted: false,
            survived_detonation: false,
        }
    }

    fn tick(&mut self) {
        let report = self.planner.pump_for_test(usize::MAX);
        self.delivered += report.delivered;
        self.moving_plans += report
            .planned_intents
            .iter()
            .filter(|intent| intent.movement != grenadiers_core::cpu::MovementIntent::Stop)
            .count();
        self.max_work = self
            .max_work
            .max(report.work.into_iter().max().unwrap_or(0));
        self.driver.tick(Instant::now()).unwrap();
        let inspection = self.driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let ids = self
            .seats
            .iter()
            .map(|seat| state.actors()[seat.index()].id())
            .collect::<Vec<_>>();
        let has_own_bomb = state
            .world()
            .bombs()
            .values()
            .any(|bomb| ids.contains(&bomb.owner()));
        self.planted |= has_own_bomb;
        if self.planted
            && !has_own_bomb
            && self
                .seats
                .iter()
                .all(|seat| state.actors()[seat.index()].alive())
        {
            self.survived_detonation = true;
        }
        for (index, seat) in self.seats.iter().enumerate() {
            self.moved[index] |= state.actors()[seat.index()].position() != self.initial[index];
        }
    }

    fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    fn advance_without_worker(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.driver.tick(Instant::now()).unwrap();
        }
    }

    fn positions(&self) -> Vec<Position> {
        let inspection = self.driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        self.seats
            .iter()
            .map(|seat| state.actors()[seat.index()].position())
            .collect()
    }

    fn mob_count(&self) -> usize {
        self.driver
            .runtime
            .replay_inspection()
            .simulation
            .unwrap()
            .state
            .mobs()
            .len()
    }
}

#[test]
fn queued_cpu19_moves_on_every_rules8_board_count_difficulty_and_environment() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in 2..=4 {
            for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
                for cloud in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
                    for lairs in [false, true] {
                        let case = Case {
                            board,
                            count,
                            difficulty,
                            cloud,
                            lairs,
                        };
                        let mut episode = Episode::new(case);
                        episode.run(300);
                        assert!(episode.moved.iter().all(|moved| *moved), "{case:?}");
                        assert!(episode.delivered > 0, "{case:?}");
                        assert!(episode.moving_plans > 0, "{case:?}");
                        assert!(episode.max_work <= WORK_ALLOWANCE, "{case:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn clear_rules8_scene_plants_and_exits_its_own_blast() {
    let mut episode = Episode::new(Case {
        board: BoardSize::ElevenByNine,
        count: 2,
        difficulty: CpuDifficulty::Standard,
        cloud: StormCloudConfig::Disabled,
        lairs: true,
    });
    episode.run(2_400);
    assert!(episode.planted);
    assert!(episode.survived_detonation);
    assert!(episode.max_work <= WORK_ALLOWANCE);
}

#[test]
fn queued_cpu19_keeps_progressing_after_real_mobs_spawn() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
            let case = Case {
                board,
                count: if board == BoardSize::ThirteenByEleven {
                    3
                } else {
                    2
                },
                difficulty,
                cloud: StormCloudConfig::Disabled,
                lairs: true,
            };
            let mut episode = Episode::new(case);
            for _ in 0..900 {
                episode.advance_without_worker(1);
                if episode.mob_count() > 0 {
                    break;
                }
            }
            assert!(episode.mob_count() > 0, "{case:?}");
            let before = episode.positions();
            let plans_before = episode.moving_plans;
            let mut saw_stopped_handoff = false;
            for _ in 0..180 {
                episode.tick();
                saw_stopped_handoff |= episode
                    .seats
                    .iter()
                    .any(|seat| episode.driver.cpu_stationary[seat.index()]);
            }
            let after = episode.positions();
            assert!(
                before
                    .iter()
                    .zip(&after)
                    .any(|(before, after)| before != after),
                "no post-spawn physical progress for {case:?}"
            );
            assert!(episode.moving_plans > plans_before, "{case:?}");
            assert!(saw_stopped_handoff, "{case:?}");
            assert!(episode.max_work <= WORK_ALLOWANCE, "{case:?}");
        }
    }
}

#[test]
fn invalid_contact_on_a_stopped_waiting_route_holds_stop_and_cannot_submit_a_leg() {
    let mut episode = Episode::new(Case {
        board: BoardSize::ElevenByNine,
        count: 2,
        difficulty: CpuDifficulty::Standard,
        cloud: StormCloudConfig::Disabled,
        lairs: true,
    });
    let mut found = None;
    for _ in 0..2_400 {
        episode.tick();
        if episode.driver.cpus[0].has_committed_route()
            && episode.driver.cpus[0].has_committed_contact()
            && episode.driver.cpu_stationary[SeatId::Two.index()]
        {
            found = Some(0);
            break;
        }
    }
    let index = found.expect("real CPU19 plant route reaches a stopped wait");
    assert!(episode.driver.expire_contact_for_test(index));
    assert!(!episode.driver.cpus[index].has_committed_route());
    assert_eq!(
        episode.driver.cpu_stop_at[SeatId::Two.index()],
        Some(episode.driver.runtime.lifecycle().tick())
    );
    episode.driver.service_cpu().unwrap();
    assert!(episode.driver.cpu_stationary[SeatId::Two.index()]);
    assert!(!episode.driver.cpus[index].has_committed_route());
}

#[test]
fn scheduled_route_stop_confirmation_retains_its_contact_lease() {
    let mut episode = Episode::new(Case {
        board: BoardSize::ElevenByNine,
        count: 2,
        difficulty: CpuDifficulty::Standard,
        cloud: StormCloudConfig::Disabled,
        lairs: true,
    });
    let mut confirmed = false;
    let mut awaiting_confirmation = None;
    for _ in 0..2_400 {
        episode.planner.pump_for_test(usize::MAX);
        let was_stationary = episode.driver.cpu_stationary[SeatId::Two.index()];
        episode.driver.service_cpu().unwrap();
        let cpu = &episode.driver.cpus[0];
        if awaiting_confirmation.is_none()
            && !was_stationary
            && episode.driver.cpu_stationary[SeatId::Two.index()]
            && cpu.has_committed_route()
            && cpu.has_committed_contact()
        {
            awaiting_confirmation = cpu
                .latest_observation_for_test()
                .map(|observation| observation.parts().tick)
        }
        if let Some(notice) = episode.driver.runtime.settle_boundary([None; 4]).unwrap() {
            episode.driver.publish(notice);
        }
        if awaiting_confirmation.is_some_and(|before| {
            episode.driver.cpus[0]
                .latest_observation_for_test()
                .is_some_and(|observation| observation.parts().tick > before)
        }) {
            assert!(episode.driver.cpus[0].has_committed_route());
            assert!(episode.driver.cpus[0].has_committed_contact());
            assert!(episode.driver.cpu_stationary[SeatId::Two.index()]);
            confirmed = true;
            break;
        }
    }
    assert!(
        confirmed,
        "real rules8 route reaches a confirmed scheduled stop"
    );
}
