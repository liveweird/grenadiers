use std::collections::BTreeSet;
use std::sync::Arc;

use grenadiers_core::{
    cpu::WorldId,
    game::{EnabledItems, MobNavigationPolicy, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, RespawnLairSettings, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
    },
};
use tokio::{sync::watch, time::Instant};

use super::super::{RosterOwnerNotice, driver::Driver, validate_cpus};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{AdmissionTrace, CpuController, CpuPlanner, PumpReport},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Outcome {
    pub profile: CpuProfile,
    pub moved: [bool; 3],
    pub owned_bombs: [usize; 3],
    pub upgrades: [[bool; 3]; 3],
    pub attack_intents: [usize; 3],
    pub attack_bombs: [usize; 3],
    pub completed_switches: [usize; 3],
    pub alive: [bool; 3],
    pub ever_dead: [bool; 3],
    pub dequeued: usize,
    pub delivered: usize,
    pub cancelled: usize,
    pub failed: usize,
    pub dropped: usize,
    pub work_len: usize,
    pub max_work: u32,
    pub admissions: [AdmissionTrace; 3],
}

pub(super) const fn current_profile(difficulty: CpuDifficulty) -> CpuProfile {
    match difficulty {
        CpuDifficulty::Easy => CpuProfile::PICKUP_V31_EASY,
        CpuDifficulty::Standard => CpuProfile::PICKUP_V31_STANDARD,
        CpuDifficulty::Hard => CpuProfile::HARD_V32,
    }
}

pub(super) fn run(case: Case, ticks: usize) -> Outcome {
    let profile = current_profile(case.difficulty);
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
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: ROOM_INSTANCE,
            configuration,
        },
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        RespawnLairTuning::new(case.lairs, RespawnLairSettings::DEFAULT),
        SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.fill(case.seed);
            Ok(())
        }),
        Arc::new(move |epoch| {
            Ok(SimulationSeed::new(
                [case.seed.wrapping_add(epoch as u8); 32],
            ))
        }),
        MobNavigationPolicy::TimedEscapeV3,
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
    let notice = runtime.current_notice().unwrap();
    assert!(notice.spike_v19_cpu_publication().is_some());
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
    let mut moved = [false; 3];
    let mut owned_bombs = [0; 3];
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
        if case.native {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
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
        moved,
        owned_bombs,
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
    log(case, &outcome);
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

fn rejection_total(trace: &AdmissionTrace) -> u32 {
    trace.expired_or_wrong_decision
        + trace.missed_manual_tick
        + trace.stale_scene
        + trace.invalid_hold
        + trace.contact
        + trace.strategic_plant
        + trace.invalid_shape
        + trace.pulse
        + trace.route
        + trace.duplicate_decision
}

fn admission_sum(admissions: &[AdmissionTrace; 3], value: impl Fn(&AdmissionTrace) -> u32) -> u32 {
    admissions.iter().map(value).sum()
}

fn log(case: Case, outcome: &Outcome) {
    eprintln!(
        "{}",
        serde_json::json!({
            "case": case.label,
            "profile": outcome.profile.policy.version(),
            "board": format!("{:?}", case.board),
            "lairs": case.lairs,
            "native": case.native,
            "moved": outcome.moved,
            "ownedBombs": outcome.owned_bombs,
            "upgrades": outcome.upgrades,
            "attackIntents": outcome.attack_intents,
            "attackBombs": outcome.attack_bombs,
            "completed": outcome.completed_switches,
            "alive": outcome.alive,
            "everDead": outcome.ever_dead,
            "dequeued": outcome.dequeued,
            "delivered": outcome.delivered,
            "cancelled": outcome.cancelled,
            "dropped": outcome.dropped,
            "maxWork": outcome.max_work,
            "admissionDenials": {
                "total": admission_sum(&outcome.admissions, rejection_total),
                "expired": admission_sum(&outcome.admissions, |trace| trace.expired_or_wrong_decision),
                "missedManual": admission_sum(&outcome.admissions, |trace| trace.missed_manual_tick),
                "stale": admission_sum(&outcome.admissions, |trace| trace.stale_scene),
                "contact": admission_sum(&outcome.admissions, |trace| trace.contact),
                "strategic": admission_sum(&outcome.admissions, |trace| trace.strategic_plant),
                "pulse": admission_sum(&outcome.admissions, |trace| trace.pulse),
                "route": admission_sum(&outcome.admissions, |trace| trace.route),
            },
        })
    );
}

pub(super) fn assert_operational(outcome: &Outcome) {
    assert!(outcome.moved.into_iter().all(|value| value), "{outcome:?}");
    assert!(
        outcome.owned_bombs.into_iter().all(|value| value > 0),
        "{outcome:?}"
    );
    assert!(outcome.delivered > 0, "{outcome:?}");
    assert_eq!(
        outcome.dequeued,
        outcome.delivered + outcome.cancelled + outcome.failed + outcome.dropped,
        "{outcome:?}"
    );
    assert_eq!(outcome.dequeued, outcome.work_len, "{outcome:?}");
    assert_eq!(outcome.failed, 0, "{outcome:?}");
    assert!(
        outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
        "{outcome:?}"
    );
}
