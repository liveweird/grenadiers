use std::sync::Arc;

use grenadiers_core::{
    cpu::survival::WORK_ALLOWANCE,
    game::{EnabledItems, Position, SeatId, SimulationSeed, SpikePitPhase, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, MatchPhase, RespawnLairSettings, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
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
    difficulty: CpuDifficulty,
    cloud: StormCloudConfig,
    pits: SpikePitTuning,
    seed: u8,
}

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    position: Position,
    alive: bool,
    accepted_motion_ticks: usize,
    accepted_bomb_births: usize,
    planned_moves: usize,
    delivered: usize,
    max_work: u32,
    saw_warning: bool,
    saw_raised: bool,
    activation_tick: Option<u64>,
    post_activation_publications: usize,
    post_activation_motion_ticks: usize,
    safe_egress: bool,
    publications: usize,
    publication_motion: usize,
}

struct Episode {
    driver: Driver,
    planner: CpuPlanner,
    last_position: Position,
    last_bombs: usize,
    last_pits: Vec<grenadiers_core::game::SpikePitView>,
    activated_cell: Option<grenadiers_core::game::CellId>,
    last_publication_tick: u64,
    last_publication_position: Position,
    outcome: Outcome,
}

impl Episode {
    fn new(case: Case) -> Self {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(case.difficulty),
            BoardSize::FifteenByThirteen,
        )
        .unwrap();
        let mut runtime = RosterRuntime::new_spike_with_navigation(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 4010,
                configuration,
            },
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
            case.cloud,
            RespawnLairTuning::new(false, RespawnLairSettings::new(10).unwrap()),
            case.pits,
            RoundSeedSource::with_entropy(move |bytes| {
                bytes.fill(case.seed);
                Ok(())
            }),
            Arc::new(move |epoch| {
                Ok(SimulationSeed::new(
                    [case.seed.wrapping_add(epoch as u8); 32],
                ))
            }),
            grenadiers_core::game::MobNavigationPolicy::EscapeV2,
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
            CpuDifficulty::Easy => CpuProfile::SPIKE_PITS_EASY,
            CpuDifficulty::Standard => CpuProfile::SPIKE_PITS_STANDARD,
        };
        let cpu = CpuController::new(planner.clone(), SeatId::Two, 4010, [case.seed; 32], profile);
        let notice = runtime.current_notice().unwrap();
        assert!(notice.spike_cpu_publication().is_some());
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
            vec![cpu],
        );
        driver.publish(notice);
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let position = state.actors()[SeatId::Two.index()].position();
        let bombs = state.world().bombs().len();
        let last_pits = state.spike_pits().unwrap_or_default();
        let last_publication_tick = driver
            .last
            .spike_cpu_publication()
            .and_then(|publication| {
                grenadiers_core::protocol::v18::project_recipient(
                    publication,
                    grenadiers_core::protocol::v18::Seat::Two,
                )
                .ok()
            })
            .map_or(0, |projected| projected.state.physical_tick.get());
        Self {
            driver,
            planner,
            last_position: position,
            last_bombs: bombs,
            last_pits,
            activated_cell: None,
            last_publication_tick,
            last_publication_position: position,
            outcome: Outcome {
                position,
                alive: true,
                accepted_motion_ticks: 0,
                accepted_bomb_births: 0,
                planned_moves: 0,
                delivered: 0,
                max_work: 0,
                saw_warning: false,
                saw_raised: false,
                activation_tick: None,
                post_activation_publications: 0,
                post_activation_motion_ticks: 0,
                safe_egress: false,
                publications: 0,
                publication_motion: 0,
            },
        }
    }

    fn tick(&mut self) {
        let report = self.planner.pump_for_test(usize::MAX);
        self.outcome.delivered += report.delivered;
        self.outcome.planned_moves += report
            .planned_intents
            .iter()
            .filter(|intent| intent.movement != grenadiers_core::cpu::MovementIntent::Stop)
            .count();
        self.outcome.max_work = self
            .outcome
            .max_work
            .max(report.work.into_iter().max().unwrap_or(0));
        self.driver.tick(Instant::now()).unwrap();
        let inspection = self.driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let actor = &state.actors()[SeatId::Two.index()];
        let position = actor.position();
        self.outcome.accepted_motion_ticks += usize::from(position != self.last_position);
        if self.outcome.activation_tick.is_some() && position != self.last_position {
            self.outcome.post_activation_motion_ticks += 1;
        }
        self.last_position = position;
        self.outcome.position = position;
        self.outcome.alive = actor.alive();
        let bombs = state.world().bombs().len();
        self.outcome.accepted_bomb_births += usize::from(bombs > self.last_bombs);
        self.last_bombs = bombs;
        let pits = state.spike_pits().unwrap_or_default();
        self.outcome.saw_warning |= pits.iter().any(|pit| pit.phase == SpikePitPhase::Warning);
        self.outcome.saw_raised |= pits.iter().any(|pit| pit.phase == SpikePitPhase::Raised);
        if self.outcome.activation_tick.is_none()
            && let Some(activated) = pits.iter().find(|pit| {
                pit.phase == SpikePitPhase::Warning
                    && self.last_pits.iter().any(|previous| {
                        previous.cell == pit.cell && previous.phase == SpikePitPhase::Idle
                    })
            })
        {
            self.outcome.activation_tick = Some(state.tick().0);
            self.activated_cell = Some(activated.cell);
        }
        if let Some(cell) = self.activated_cell
            && !grenadiers_core::game::position_contacts_spike_pit(position, cell)
        {
            self.outcome.safe_egress = true;
        }
        self.last_pits = pits;
        if let Some(publication) = self.driver.last.spike_cpu_publication()
            && let Ok(projected) = grenadiers_core::protocol::v18::project_recipient(
                publication,
                grenadiers_core::protocol::v18::Seat::Two,
            )
        {
            let tick = projected.state.physical_tick.get();
            if tick > self.last_publication_tick {
                self.outcome.publications += 1;
                self.outcome.publication_motion +=
                    usize::from(position != self.last_publication_position);
                if self
                    .outcome
                    .activation_tick
                    .is_some_and(|activation| tick > activation)
                {
                    self.outcome.post_activation_publications += 1;
                }
                self.last_publication_tick = tick;
                self.last_publication_position = position;
            }
        }
    }

    fn run(mut self, ticks: usize) -> Outcome {
        for _ in 0..ticks {
            self.tick();
        }
        self.outcome
    }
}

fn tuning(enabled: bool, warning: u64, raised: u64) -> SpikePitTuning {
    SpikePitTuning::new(enabled, SpikePitSettings::new(warning, raised).unwrap())
}

#[test]
fn queued_cpu26_intentionally_avoids_idle_pits_without_publication_stall() {
    let outcome = Episode::new(Case {
        difficulty: CpuDifficulty::Standard,
        cloud: StormCloudConfig::Disabled,
        pits: tuning(true, 6, 60),
        seed: 41,
    })
    .run(840);
    assert!(outcome.alive, "{outcome:?}");
    assert!(outcome.delivered > 0, "{outcome:?}");
    assert!(outcome.planned_moves > 0, "{outcome:?}");
    assert!(outcome.accepted_motion_ticks > 0, "{outcome:?}");
    assert!(outcome.accepted_bomb_births > 0, "{outcome:?}");
    assert_eq!(outcome.activation_tick, None, "{outcome:?}");
    assert!(!outcome.saw_warning && !outcome.saw_raised, "{outcome:?}");
    assert!(outcome.publications > 10, "{outcome:?}");
    assert!(outcome.publication_motion > 0, "{outcome:?}");
    assert!(outcome.max_work <= WORK_ALLOWANCE, "{outcome:?}");
}

#[test]
fn easy_cpu26_progresses_with_storm_and_maximum_raised_duration() {
    let outcome = Episode::new(Case {
        difficulty: CpuDifficulty::Easy,
        cloud: StormCloudConfig::Single,
        pits: tuning(true, 120, 600),
        seed: 53,
    })
    .run(840);
    assert!(outcome.alive, "{outcome:?}");
    assert!(outcome.delivered > 0, "{outcome:?}");
    assert!(outcome.accepted_motion_ticks > 0, "{outcome:?}");
    assert!(outcome.max_work <= WORK_ALLOWANCE, "{outcome:?}");
}

#[test]
fn disabled_pits_are_deterministic_and_keep_queued_navigation_playable() {
    let case = Case {
        difficulty: CpuDifficulty::Standard,
        cloud: StormCloudConfig::Disabled,
        pits: tuning(false, 6, 600),
        seed: 67,
    };
    let first = Episode::new(case).run(360);
    let second = Episode::new(case).run(360);
    assert_eq!(first, second);
    assert!(first.alive && first.accepted_motion_ticks > 0, "{first:?}");
    assert!(!first.saw_warning && !first.saw_raised, "{first:?}");
}
