use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, EntityId, Position, SeatId, SimulationSeed, StormCloudConfig, Tick},
    protocol::{RoomCode, RoomId, v14, v16},
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

#[derive(Clone, Debug)]
struct BombEvent {
    seat: SeatId,
    cell: (u8, u8),
    range: u8,
    placed: Tick,
    placed_position: Position,
    route_after_placement: bool,
    recovery_after_placement: (bool, bool),
    detonated: Option<Tick>,
    detonation_position: Option<Position>,
    owner_alive_at_detonation: Option<bool>,
}

#[derive(Debug)]
struct OpeningResult {
    bombs: Vec<BombEvent>,
    eliminated: Vec<(SeatId, Tick)>,
    position_changes: [usize; 3],
}

struct OpeningEpisode {
    driver: Driver,
    planner: CpuPlanner,
    seats: [SeatId; 3],
    ids: [EntityId; 3],
    last_positions: [Position; 3],
    position_changes: [usize; 3],
    bombs: Vec<(u32, BombEvent)>,
    eliminated: Vec<(SeatId, Tick)>,
}

impl OpeningEpisode {
    fn new(
        board: BoardSize,
        difficulty: CpuDifficulty,
        cloud: bool,
        arena_seed: u8,
        cpu_seed: u8,
    ) -> Result<Self, crate::game::rooms::roster_runtime::RosterRuntimeError> {
        let configuration = RoomConfiguration::new(
            &[
                SeatControl::Human,
                SeatControl::Cpu,
                SeatControl::Cpu,
                SeatControl::Cpu,
            ],
            Some(difficulty),
            board,
        )
        .unwrap();
        let mut runtime = RosterRuntime::new_lairs(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 4_019,
                configuration,
            },
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
            if cloud {
                StormCloudConfig::Single
            } else {
                StormCloudConfig::Disabled
            },
            LairTuning::new(true, LairSettings::new(10, 3).unwrap()),
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
        let profile = match difficulty {
            CpuDifficulty::Easy => CpuProfile::LAIRS_EASY,
            CpuDifficulty::Standard => CpuProfile::LAIRS_STANDARD,
            CpuDifficulty::Hard => return Err(
                crate::game::rooms::roster_runtime::RosterRuntimeError::UnsupportedConfiguration,
            ),
        };
        let cpus = seats
            .into_iter()
            .map(|seat| CpuController::new(planner.clone(), seat, 4_019, [cpu_seed; 32], profile))
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
        let state = &inspection.simulation.unwrap().state;
        let ids = seats.map(|seat| state.actors()[seat.index()].id());
        let last_positions = seats.map(|seat| state.actors()[seat.index()].position());
        Ok(Self {
            driver,
            planner,
            seats,
            ids,
            last_positions,
            position_changes: [0; 3],
            bombs: Vec::new(),
            eliminated: Vec::new(),
        })
    }

    fn tick(&mut self) {
        self.planner.pump_for_test(usize::MAX);
        self.driver.tick(Instant::now()).unwrap();
        let inspection = self.driver.runtime.replay_inspection();
        let state = &inspection.simulation.unwrap().state;
        let notice = self.driver.notices.borrow().clone();
        if let RosterOwnerNotice::Publication(notice) = notice
            && let Some(publication) = notice.lair_cpu_publication()
        {
            let projection = v16::project_recipient(publication, v16::Seat::One).unwrap();
            for event in projection.events.as_slice() {
                match &event.kind {
                    v14::EventKind::BombPlaced { bomb } => {
                        if !self.bombs.iter().any(|(id, _)| *id == bomb.id.get())
                            && let Some(index) =
                                self.ids.iter().position(|id| id.0 == bomb.owner.get())
                        {
                            self.bombs.push((
                                bomb.id.get(),
                                BombEvent {
                                    seat: self.seats[index],
                                    cell: (bomb.cell.column(), bomb.cell.row()),
                                    range: bomb.range,
                                    placed: Tick(event.tick.get()),
                                    placed_position: state.actors()[self.seats[index].index()]
                                        .position(),
                                    route_after_placement: self.driver.cpus[index]
                                        .has_committed_route(),
                                    recovery_after_placement: self.driver.cpus[index]
                                        .recovery_state(),
                                    detonated: None,
                                    detonation_position: None,
                                    owner_alive_at_detonation: None,
                                },
                            ));
                        }
                    }
                    v14::EventKind::BombDetonated { bomb } => {
                        if let Some((_, recorded)) =
                            self.bombs.iter_mut().find(|(id, _)| *id == bomb.id.get())
                        {
                            recorded.detonated = Some(Tick(event.tick.get()));
                            recorded.detonation_position =
                                Some(state.actors()[recorded.seat.index()].position());
                            recorded.owner_alive_at_detonation =
                                Some(state.actors()[recorded.seat.index()].alive());
                        }
                    }
                    v14::EventKind::ActorEliminated { seat, .. } => {
                        let seat = match seat {
                            v16::Seat::One => SeatId::One,
                            v16::Seat::Two => SeatId::Two,
                            v16::Seat::Three => SeatId::Three,
                            v16::Seat::Four => SeatId::Four,
                        };
                        let elimination = (seat, Tick(event.tick.get()));
                        if !self.eliminated.contains(&elimination) {
                            self.eliminated.push(elimination);
                        }
                    }
                    _ => {}
                }
            }
        }
        for (index, seat) in self.seats.into_iter().enumerate() {
            let position = state.actors()[seat.index()].position();
            self.position_changes[index] += usize::from(position != self.last_positions[index]);
            self.last_positions[index] = position;
        }
    }

    fn run(mut self, ticks: usize) -> OpeningResult {
        for _ in 0..ticks {
            self.tick();
        }
        OpeningResult {
            bombs: self.bombs.into_iter().map(|(_, event)| event).collect(),
            eliminated: self.eliminated,
            position_changes: self.position_changes,
        }
    }
}

#[test]
fn default_four_seat_cpu_opening_recovers_from_simultaneous_route_invalidation() {
    for arena_seed in [0, 17, 36, 91] {
        let result = OpeningEpisode::new(
            BoardSize::FifteenByThirteen,
            CpuDifficulty::Standard,
            true,
            arena_seed,
            53,
        )
        .unwrap()
        .run(390);
        assert!(
            result.bombs.len() >= 3,
            "arena seed {arena_seed}: {result:#?}"
        );
        for (index, seat) in [SeatId::Two, SeatId::Three, SeatId::Four]
            .into_iter()
            .enumerate()
        {
            let bomb = result
                .bombs
                .iter()
                .filter(|bomb| bomb.seat == seat)
                .min_by_key(|bomb| bomb.placed)
                .unwrap_or_else(|| panic!("arena seed {arena_seed}: {result:#?}"));
            assert_eq!(
                bomb.placed,
                Tick(205),
                "arena seed {arena_seed}: {result:#?}"
            );
            assert_eq!(
                bomb.detonated,
                Some(Tick(385)),
                "arena seed {arena_seed}: {result:#?}"
            );
            assert!(
                !bomb.route_after_placement,
                "arena seed {arena_seed}: {result:#?}"
            );
            assert!(
                bomb.recovery_after_placement.0,
                "arena seed {arena_seed}: {result:#?}"
            );
            assert!(
                result.position_changes[index] > 0,
                "arena seed {arena_seed}: {result:#?}"
            );
            assert_ne!(
                bomb.detonation_position,
                Some(bomb.placed_position),
                "arena seed {arena_seed}: {result:#?}"
            );
            assert_eq!(
                bomb.owner_alive_at_detonation,
                Some(true),
                "arena seed {arena_seed}: {result:#?}"
            );
        }
        assert!(
            result.eliminated.is_empty(),
            "arena seed {arena_seed}: {result:#?}"
        );
    }
}

#[test]
fn every_board_and_difficulty_survives_each_cpu_first_owned_detonation() {
    let mut failures = Vec::new();
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
            // Running starts at physical tick 180. Tick 420 is beyond the
            // opening bombs' flames but before the first default lair spawn.
            let result = OpeningEpisode::new(board, difficulty, true, 0, 53)
                .unwrap()
                .run(240);
            for seat in [SeatId::Two, SeatId::Three, SeatId::Four] {
                let Some(first) = result
                    .bombs
                    .iter()
                    .filter(|bomb| bomb.seat == seat)
                    .min_by_key(|bomb| bomb.placed)
                else {
                    failures.push(format!("{board:?} {difficulty:?} {seat:?}: no owned bomb"));
                    continue;
                };
                if first.detonated.is_none() || first.owner_alive_at_detonation != Some(true) {
                    failures.push(format!(
                        "{board:?} {difficulty:?} {seat:?}: bomb cell {:?} range {}; {first:#?}",
                        first.cell, first.range
                    ));
                }
            }
            if result
                .eliminated
                .iter()
                .any(|(seat, _)| *seat != SeatId::One)
            {
                failures.push(format!(
                    "{board:?} {difficulty:?}: opening eliminations {:?}",
                    result.eliminated
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
