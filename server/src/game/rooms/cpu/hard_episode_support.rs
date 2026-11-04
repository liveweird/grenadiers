use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use grenadiers_core::{
    cpu::WorldId,
    game::{
        ArenaConstructionProfile, EnabledItems, MobNavigationPolicy, SeatId, SimulationSeed,
        StepEventKind, StormCloudConfig,
    },
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuDifficulty, CpuProfile, RespawnLairSettings,
        RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings,
        SpikePitTuning,
    },
};
use tokio::{sync::watch, time::Instant};

use super::super::{RosterOwnerNotice, driver::Driver, validate_cpus};
use super::outcome::{BombHorizon, Outcome};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{AdmissionTrace, CpuController, CpuPlanner, PumpReport},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime, SpikeRuntimeEnvironment},
    },
    round_seed::RoundSeedSource,
};

const CPU_SEATS: [SeatId; 3] = [SeatId::Two, SeatId::Three, SeatId::Four];
const ROOM_INSTANCE: u64 = 4_028;

#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub label: &'static str,
    pub difficulty: CpuDifficulty,
    pub board: BoardSize,
    pub lairs: bool,
    pub seed: u8,
    pub native: bool,
}

pub(super) const fn current_profile(difficulty: CpuDifficulty) -> CpuProfile {
    match difficulty {
        CpuDifficulty::Easy => CpuProfile::CONTACT_PUNCH_V60_EASY,
        CpuDifficulty::Standard => CpuProfile::CONTACT_PUNCH_V60_STANDARD,
        CpuDifficulty::Hard => CpuProfile::HARD_V61,
    }
}

pub(super) fn run(case: Case, ticks: usize) -> Outcome {
    let profile = current_profile(case.difficulty);
    run_inner(case, ticks, profile, false)
}

pub(super) fn run_clearance(case: Case, ticks: usize) -> Outcome {
    let profile = match case.difficulty {
        CpuDifficulty::Easy => CpuProfile::RECOVERY_V62_EASY,
        CpuDifficulty::Standard => CpuProfile::RECOVERY_V62_STANDARD,
        CpuDifficulty::Hard => CpuProfile::HARD_V64,
    };
    run_inner(case, ticks, profile, true)
}

fn run_inner(case: Case, ticks: usize, profile: CpuProfile, clearance: bool) -> Outcome {
    let configuration = RoomConfiguration::new(
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Cpu,
            SeatControl::Cpu,
        ],
        Some(case.difficulty),
        case.board,
    )
    .unwrap();
    let identity = RosterIdentity {
        id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        registry_serial: ROOM_INSTANCE,
        configuration,
    };
    let settings = RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT);
    let environment = SpikeRuntimeEnvironment {
        cloud: StormCloudConfig::Single,
        lairs: RespawnLairTuning::new(case.lairs, RespawnLairSettings::DEFAULT),
        spike_pits: SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
        navigation: MobNavigationPolicy::TimedEscapeV3,
    };
    let seeds = RoundSeedSource::with_entropy(move |bytes| {
        bytes.fill(case.seed);
        Ok(())
    });
    let simulation_seed = Arc::new(move |epoch| {
        Ok(SimulationSeed::new(
            [case.seed.wrapping_add(epoch as u8); 32],
        ))
    });
    let mut runtime = if clearance {
        RosterRuntime::new_spike_v21_clearance_with_navigation(
            identity,
            settings,
            environment,
            seeds,
            simulation_seed,
        )
    } else {
        RosterRuntime::new_spike_v21_with_navigation(
            identity,
            settings,
            environment,
            seeds,
            simulation_seed,
        )
    }
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
    let planner = if case.native {
        CpuPlanner::new().unwrap()
    } else {
        CpuPlanner::without_workers()
    };
    let cpus = CPU_SEATS
        .map(|seat| {
            CpuController::new(
                planner.clone(),
                seat,
                ROOM_INSTANCE,
                [case.seed.wrapping_add(seat.index() as u8); 32],
                profile,
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    assert!(cpus.iter().all(|cpu| cpu.profile() == profile));
    assert_eq!(validate_cpus(&runtime, &cpus), Ok(()));
    let pre_driver = runtime.replay_inspection();
    let construction_profile = pre_driver.construction_profile;
    let simulation = pre_driver.simulation.as_ref().unwrap();
    let generation_version = simulation.generation.version();
    let pit_layout = simulation.state.rules().spike_pit_layout_policy();
    let multiplier = pre_driver
        .lifecycle
        .settings()
        .bomb_capacity_drop_multiplier();
    assert_eq!(
        pre_driver
            .lifecycle
            .settings()
            .bomb_capacity_drop_multiplier(),
        BombCapacityDropMultiplier::DEFAULT
    );
    let expected_construction = if clearance {
        ArenaConstructionProfile::WeightedLairClearanceV7
    } else {
        ArenaConstructionProfile::WeightedV6
    };
    assert_eq!(construction_profile, expected_construction);
    assert_eq!(generation_version, if clearance { 7 } else { 6 });
    let notice = runtime.current_notice().unwrap();
    assert!(notice.spike_v21_cpu_publication().is_some());
    assert_eq!(
        notice
            .spike_v21_room()
            .unwrap()
            .settings
            .bomb_capacity_drop_multiplier(),
        grenadiers_core::room::BombCapacityDropMultiplier::DEFAULT
    );
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
    let initial = driver.runtime.replay_inspection();
    let state = &initial.simulation.as_ref().unwrap().state;
    let starts = CPU_SEATS.map(|seat| state.actors()[seat.index()].position());
    let upgrades = CPU_SEATS.map(|seat| {
        let actor = &state.actors()[seat.index()];
        (actor.bomb_capacity(), actor.bomb_range(), actor.speed())
    });
    let mut known_bombs = state
        .world()
        .bombs()
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let initial_blocks = state.world().arena().blocks().len();
    let mut minimum_blocks = initial_blocks;
    let mut known_mobs = BTreeSet::new();
    let mut previous_mobs = BTreeMap::new();
    let mut flames_seen = false;
    let mut mob_moved = false;
    let mut moved = [false; 3];
    let mut owned_bombs = [0; 3];
    let mut event_placements = [0; 3];
    let mut first_bombs = [None; 3];
    let mut upgrades_seen = [[false; 3]; 3];
    let mut attack_intents = [0; 3];
    let mut attack_bombs = [0; 3];
    let mut completed_switches = [0; 3];
    let mut previous_completed = [false; 3];
    let mut ever_dead = [false; 3];
    let mut aggregate = PumpReport::default();
    for _ in 0..ticks {
        if !case.native {
            merge(&mut aggregate, planner.pump_for_test(usize::MAX));
        }
        driver.tick(Instant::now()).unwrap();
        let (_, report) = driver.runtime.last_step_trace_for_test().unwrap();
        let post_state = driver.runtime.state_for_test().unwrap();
        for event in &report.events {
            match event.kind {
                StepEventKind::BombPlaced(bomb) => {
                    if let Some(index) = CPU_SEATS
                        .iter()
                        .position(|seat| post_state.actors()[seat.index()].id() == bomb.owner)
                    {
                        event_placements[index] += 1;
                        first_bombs[index].get_or_insert(BombHorizon {
                            id: bomb.id,
                            placed_at: event.tick,
                            detonated_at: None,
                            finite_flames_clear_at: None,
                            owner_alive_at_clear: None,
                        });
                    }
                }
                StepEventKind::BombDetonated(bomb) => {
                    if let Some(horizon) = first_bombs
                        .iter_mut()
                        .flatten()
                        .find(|horizon| horizon.id == bomb.id)
                    {
                        horizon.detonated_at = Some(event.tick);
                        horizon.finite_flames_clear_at = post_state
                            .public_view()
                            .flames
                            .iter()
                            .filter(|flame| flame.expires_at.0 < u64::MAX)
                            .map(|flame| flame.expires_at)
                            .max();
                    }
                }
                _ => {}
            }
        }
        if case.native {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        for (index, horizon) in first_bombs.iter_mut().enumerate() {
            if let Some(horizon) = horizon
                && horizon.owner_alive_at_clear.is_none()
                && horizon
                    .finite_flames_clear_at
                    .is_some_and(|clear_at| state.tick() >= clear_at)
            {
                horizon.owner_alive_at_clear =
                    Some(state.actors()[CPU_SEATS[index].index()].alive());
            }
        }
        minimum_blocks = minimum_blocks.min(state.world().arena().blocks().len());
        flames_seen |= !state.public_view().flames.is_empty();
        let mobs = state.mobs();
        for mob in mobs {
            known_mobs.insert(mob.id);
            mob_moved |= previous_mobs
                .insert(mob.id, mob.position)
                .is_some_and(|position| position != mob.position);
        }
        let world = WorldId {
            room_instance: ROOM_INSTANCE,
            physical_round: 1,
        };
        let summaries: [Option<_>; 3] =
            std::array::from_fn(|index| driver.cpus[index].intent_debug_summary_for(world));
        for (&id, bomb) in state.world().bombs() {
            if known_bombs.insert(id)
                && let Some(index) = CPU_SEATS
                    .iter()
                    .position(|seat| state.actors()[seat.index()].id() == bomb.owner())
            {
                owned_bombs[index] += 1;
                attack_bombs[index] += usize::from(summaries[index].is_some_and(|summary| {
                    summary.objective
                        == grenadiers_core::cpu::policy::StrategicObjectiveKind::Attack
                }));
            }
        }
        for (index, seat) in CPU_SEATS.into_iter().enumerate() {
            let actor = &state.actors()[seat.index()];
            moved[index] |= actor.position() != starts[index];
            upgrades_seen[index][0] |= actor.bomb_capacity() > upgrades[index].0;
            upgrades_seen[index][1] |= actor.bomb_range() > upgrades[index].1;
            upgrades_seen[index][2] |= actor.speed() > upgrades[index].2;
            ever_dead[index] |= !actor.alive();
            let attack = summaries[index].is_some_and(|summary| {
                summary.objective == grenadiers_core::cpu::policy::StrategicObjectiveKind::Attack
            });
            attack_intents[index] += usize::from(attack);
            let completed = summaries[index].is_some_and(|summary| {
                summary.last_switch_reason
                    == grenadiers_core::cpu::policy::StrategicSwitchReason::Completed
            });
            completed_switches[index] += usize::from(completed && !previous_completed[index]);
            previous_completed[index] = completed;
        }
    }
    if case.native {
        aggregate = planner.trace_for_test();
    } else {
        merge(&mut aggregate, planner.pump_for_test(usize::MAX));
    }
    let inspection = driver.runtime.replay_inspection();
    let state = &inspection.simulation.as_ref().unwrap().state;
    let admissions: [AdmissionTrace; 3] = driver
        .cpus
        .iter()
        .map(CpuController::admission_trace_for_test)
        .collect::<Vec<_>>()
        .try_into()
        .expect("three CPU admission traces");
    let outcome = Outcome {
        profile,
        construction_profile,
        generation_version,
        pit_layout,
        multiplier,
        moved,
        owned_bombs,
        event_placements,
        destroyed_blocks: initial_blocks - minimum_blocks,
        flames_seen,
        mob_births: known_mobs.len(),
        mob_moved,
        first_bombs,
        upgrades: upgrades_seen,
        attack_intents,
        attack_bombs,
        completed_switches,
        alive: CPU_SEATS.map(|seat| state.actors()[seat.index()].alive()),
        ever_dead,
        dequeued: aggregate.dequeued,
        delivered: aggregate.delivered,
        cancelled: aggregate.cancelled,
        failed: aggregate.failed,
        dropped: aggregate.dropped,
        work_len: aggregate.work.len(),
        max_work: aggregate.work.into_iter().max().unwrap_or(0),
        admissions,
    };
    super::reporting::log(case, &outcome);
    outcome
}

fn merge(total: &mut PumpReport, next: PumpReport) {
    total.dequeued += next.dequeued;
    total.delivered += next.delivered;
    total.cancelled += next.cancelled;
    total.failed += next.failed;
    total.dropped += next.dropped;
    total.work.extend(next.work);
    total.planned_intents.extend(next.planned_intents);
    total.planner_times.extend(next.planner_times);
}
