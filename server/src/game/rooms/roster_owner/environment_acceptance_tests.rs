use std::sync::Arc;

use grenadiers_core::{
    cpu::survival::WORK_ALLOWANCE,
    game::{Direction, Position, SeatId, SimulationSeed, StormCloudConfig, Tick},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, MatchPhase, RoomConfiguration, RoomSettings,
        SeatControl,
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
    environment: StormCloudConfig,
    arena_seed: u8,
    cpu_seed: u8,
}

struct Episode {
    driver: Driver,
    planner: CpuPlanner,
    cpu_seats: Vec<SeatId>,
    initial_positions: Vec<Position>,
    moved: Vec<bool>,
    planted: bool,
    planted_at: Option<Tick>,
    survived_own_detonation: bool,
    saw_slow: bool,
    saw_refresh: bool,
    saw_expiry: bool,
    last_slow: Vec<Option<Tick>>,
    max_work: u32,
    delivered: usize,
    cancelled: usize,
    planned_movement: usize,
    planned_manual: usize,
    min_cloud_actor_distance: i32,
}

impl Episode {
    fn new(case: Case) -> Result<Self, crate::game::rooms::roster_runtime::RosterRuntimeError> {
        let controls = std::iter::once(SeatControl::Human)
            .chain(std::iter::repeat_n(SeatControl::Cpu, case.count - 1))
            .collect::<Vec<_>>();
        let configuration =
            RoomConfiguration::new(&controls, Some(case.difficulty), case.board).unwrap();
        let mut runtime = RosterRuntime::new_environment(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 3114,
                configuration,
            },
            RoomSettings::DEFAULT,
            case.environment,
            RoundSeedSource::with_entropy(move |bytes| {
                bytes.fill(case.arena_seed);
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
            CpuDifficulty::Easy => CpuProfile::ENVIRONMENT_EASY,
            CpuDifficulty::Standard => CpuProfile::ENVIRONMENT_STANDARD,
            CpuDifficulty::Hard => return Err(
                crate::game::rooms::roster_runtime::RosterRuntimeError::UnsupportedConfiguration,
            ),
        };
        assert_eq!(profile.policy.version(), 18);
        let cpu_seats = configuration.cpu_seats().collect::<Vec<_>>();
        let cpus = cpu_seats
            .iter()
            .copied()
            .map(|seat| {
                CpuController::new(
                    planner.clone(),
                    seat,
                    3114,
                    [case.cpu_seed.wrapping_add(seat.index() as u8); 32],
                    profile,
                )
            })
            .collect();
        let notice = runtime.current_notice().unwrap();
        assert!(notice.environment_cpu_publication().is_some());
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
        let initial_positions = cpu_seats
            .iter()
            .map(|seat| state.actors()[seat.index()].position())
            .collect::<Vec<_>>();
        let cpu_count = cpu_seats.len();
        Ok(Self {
            driver,
            planner,
            cpu_seats,
            initial_positions,
            moved: vec![false; cpu_count],
            planted: false,
            planted_at: None,
            survived_own_detonation: false,
            saw_slow: false,
            saw_refresh: false,
            saw_expiry: false,
            last_slow: vec![None; case.count],
            max_work: 0,
            delivered: 0,
            cancelled: 0,
            planned_movement: 0,
            planned_manual: 0,
            min_cloud_actor_distance: i32::MAX,
        })
    }

    fn tick(&mut self, pump: bool) {
        if pump {
            let report = self.planner.pump_for_test(usize::MAX);
            self.delivered += report.delivered;
            self.cancelled += report.cancelled;
            self.planned_movement += report
                .planned_intents
                .iter()
                .filter(|intent| {
                    !matches!(intent.movement, grenadiers_core::cpu::MovementIntent::Stop)
                })
                .count();
            self.planned_manual += report
                .planned_intents
                .iter()
                .filter(|intent| intent.manual.is_some())
                .count();
            self.max_work = self
                .max_work
                .max(report.work.into_iter().max().unwrap_or(0));
        }
        self.driver.tick(Instant::now()).unwrap();
        let inspection = self.driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let cpu_ids = self
            .cpu_seats
            .iter()
            .map(|seat| state.actors()[seat.index()].id())
            .collect::<Vec<_>>();
        let own_bombs = state
            .world()
            .bombs()
            .values()
            .filter(|bomb| cpu_ids.contains(&bomb.owner()))
            .collect::<Vec<_>>();
        if let Some(first) = own_bombs.first() {
            self.planted = true;
            self.planted_at.get_or_insert(first.placed_at());
        } else if self.planted
            && self
                .cpu_seats
                .iter()
                .all(|seat| state.actors()[seat.index()].alive())
        {
            self.survived_own_detonation = true;
        }
        for (index, seat) in self.cpu_seats.iter().copied().enumerate() {
            let actor = &state.actors()[seat.index()];
            self.moved[index] |= actor.position() != self.initial_positions[index];
        }
        for (index, actor) in state.actors().iter().enumerate() {
            if let Some(cloud) = state.storm_cloud() {
                self.min_cloud_actor_distance = self.min_cloud_actor_distance.min(
                    (cloud.position.x - actor.position().x)
                        .abs()
                        .max((cloud.position.y - actor.position().y).abs()),
                );
            }
            let slow = actor.storm_slow_until();
            self.saw_slow |= slow.is_some();
            self.saw_refresh |= self.last_slow[index]
                .zip(slow)
                .is_some_and(|(before, after)| after > before);
            self.saw_expiry |= self.last_slow[index].is_some() && slow.is_none();
            self.last_slow[index] = slow;
        }
    }

    fn run(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.tick(true);
        }
    }
}

#[test]
fn cpu18_real_driver_moves_on_the_compact_environment_matrix_with_bounded_work() {
    let boards = [
        BoardSize::FifteenByThirteen,
        BoardSize::ThirteenByEleven,
        BoardSize::ElevenByNine,
    ];
    let mut cases = Vec::new();
    for (board_index, board) in boards.into_iter().enumerate() {
        for (pair, environment) in [StormCloudConfig::Disabled, StormCloudConfig::Single]
            .into_iter()
            .enumerate()
        {
            for offset in 0..3 {
                cases.push(Case {
                    board,
                    count: 2 + (board_index + pair + offset) % 3,
                    difficulty: if (pair + offset) % 2 == 0 {
                        CpuDifficulty::Easy
                    } else {
                        CpuDifficulty::Standard
                    },
                    environment,
                    arena_seed: 11 + (board_index * 29 + pair * 7 + offset) as u8,
                    cpu_seed: 37 + (board_index * 17 + pair * 5 + offset) as u8,
                });
            }
        }
    }
    assert_eq!(cases.len(), 18);
    for case in cases {
        let mut episode = Episode::new(case).unwrap();
        episode.run(240);
        assert!(
            episode.moved.iter().all(|moved| *moved),
            "CPU did not move for {case:?}: moved={:?} delivered={} planned_movement={} planned_manual={} max_work={}",
            episode.moved,
            episode.delivered,
            episode.planned_movement,
            episode.planned_manual,
            episode.max_work,
        );
        assert!(episode.delivered > 0);
        assert!(episode.max_work <= WORK_ALLOWANCE);
    }
}

#[test]
fn single_cloud_cpu18_plants_and_escapes_its_own_detonation() {
    let mut episode = Episode::new(Case {
        board: BoardSize::ElevenByNine,
        count: 2,
        difficulty: CpuDifficulty::Standard,
        environment: StormCloudConfig::Single,
        arena_seed: 73,
        cpu_seed: 151,
    })
    .unwrap();
    episode.run(2_400);
    assert!(
        episode.planted,
        "purpose-built episode never planted: moved={:?} delivered={} planned_movement={} planned_manual={} max_work={}",
        episode.moved,
        episode.delivered,
        episode.planned_movement,
        episode.planned_manual,
        episode.max_work,
    );
    assert!(
        episode.survived_own_detonation,
        "CPU did not survive through its own observed detonation"
    );
    assert!(episode.max_work <= WORK_ALLOWANCE);
}

#[test]
fn single_cloud_real_contact_refreshes_or_expires_in_a_bounded_idle_round() {
    let mut episode = Episode::new(Case {
        board: BoardSize::ElevenByNine,
        count: 2,
        difficulty: CpuDifficulty::Standard,
        environment: StormCloudConfig::Single,
        arena_seed: 73,
        cpu_seed: 151,
    })
    .unwrap();
    episode
        .driver
        .runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    for _ in 0..10_800 {
        episode.tick(false);
    }
    assert!(
        episode.saw_slow,
        "no actor contacted the real cloud; closest center distance={}",
        episode.min_cloud_actor_distance,
    );
    assert!(episode.saw_refresh || episode.saw_expiry);
}

#[test]
fn queued_stale_plan_is_cancelled_then_fresh_work_still_reaches_physics() {
    let mut episode = Episode::new(Case {
        board: BoardSize::ThirteenByEleven,
        count: 3,
        difficulty: CpuDifficulty::Easy,
        environment: StormCloudConfig::Single,
        arena_seed: 37,
        cpu_seed: 17,
    })
    .unwrap();
    for _ in 0..12 {
        episode.tick(false);
    }
    episode.run(300);
    assert!(
        episode.cancelled > 0,
        "superseded queued work was not cancelled"
    );
    assert!(episode.delivered > 0);
    assert!(episode.moved.iter().all(|moved| *moved));
    assert!(episode.max_work <= WORK_ALLOWANCE);
}
