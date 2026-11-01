use super::*;
use grenadiers_core::cpu::WorldId;
use grenadiers_core::cpu::policy::{
    StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason, StrategicTarget,
};
use grenadiers_core::game::CellId;

struct RealWorkerOutcome {
    moved: [bool; 3],
    owned_bomb_births: [usize; 3],
    useful_upgrades: [bool; 3],
    trace: crate::game::rooms::cpu::PumpReport,
    admissions: Vec<crate::game::rooms::cpu::AdmissionTrace>,
    recovery_seen: [bool; 3],
    recovery_productive: [bool; 3],
    recovery_unresolved: [bool; 3],
    max_recovery_delay: [u64; 3],
    stalled_switches: usize,
}

impl std::fmt::Debug for RealWorkerOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RealWorkerOutcome")
            .field("moved", &self.moved)
            .field("owned_bomb_births", &self.owned_bomb_births)
            .field("useful_upgrades", &self.useful_upgrades)
            .field("dequeued", &self.trace.dequeued)
            .field("delivered", &self.trace.delivered)
            .field("failed", &self.trace.failed)
            .field("max_work", &self.trace.work.iter().max())
            .field("admissions", &self.admissions)
            .field("recovery_seen", &self.recovery_seen)
            .field("recovery_productive", &self.recovery_productive)
            .field("recovery_unresolved", &self.recovery_unresolved)
            .field("max_recovery_delay", &self.max_recovery_delay)
            .field("stalled_switches", &self.stalled_switches)
            .finish()
    }
}

fn run(lairs: bool, profile: CpuProfile, pacing_ms: u64) -> RealWorkerOutcome {
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
    let mut runtime = RosterRuntime::new_spike_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: 4_027,
            configuration,
        },
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        RespawnLairTuning::new(lairs, RespawnLairSettings::DEFAULT),
        SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(73);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([73_u8.wrapping_add(epoch as u8); 32]))),
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
    let planner = CpuPlanner::new().unwrap();
    let cpus = [SeatId::Two, SeatId::Three, SeatId::Four]
        .into_iter()
        .map(|seat| {
            CpuController::new(
                planner.clone(),
                seat,
                4_027,
                [73_u8.wrapping_add(seat.index() as u8); 32],
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
    let inspection = driver.runtime.replay_inspection();
    let state = &inspection.simulation.as_ref().unwrap().state;
    let starts = [SeatId::Two, SeatId::Three, SeatId::Four]
        .map(|seat| state.actors()[seat.index()].position());
    let initial_upgrades = [SeatId::Two, SeatId::Three, SeatId::Four].map(|seat| {
        let actor = &state.actors()[seat.index()];
        (actor.bomb_capacity(), actor.bomb_range(), actor.speed())
    });
    let mut moved = [false; 3];
    let mut known_bombs: std::collections::BTreeSet<_> =
        state.world().bombs().keys().copied().collect();
    let mut owned_bomb_births = [0; 3];
    let mut useful_upgrades = [false; 3];
    let mut recovery_seen = [false; 3];
    let mut recovery_productive = [false; 3];
    let mut max_recovery_delay = [0; 3];
    let mut recovery: [(Option<CellId>, u64); 3] = [(None, 0); 3];
    let mut stalled_switches = 0;
    for _ in 0..840 {
        driver.tick(Instant::now()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(pacing_ms));
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let mut owned_bomb_this_tick = [false; 3];
        for (&id, bomb) in state.world().bombs() {
            if known_bombs.insert(id)
                && let Some(index) = [SeatId::Two, SeatId::Three, SeatId::Four]
                    .into_iter()
                    .position(|seat| state.actors()[seat.index()].id() == bomb.owner())
            {
                owned_bomb_births[index] += 1;
                owned_bomb_this_tick[index] = true;
            }
        }
        for (index, seat) in [SeatId::Two, SeatId::Three, SeatId::Four]
            .into_iter()
            .enumerate()
        {
            moved[index] |= state.actors()[seat.index()].position() != starts[index];
            let actor = &state.actors()[seat.index()];
            useful_upgrades[index] |= (actor.bomb_capacity(), actor.bomb_range(), actor.speed())
                != initial_upgrades[index];
            let summary = driver.cpus[index].intent_debug_summary_for(WorldId {
                room_instance: 4_027,
                physical_round: 1,
            });
            stalled_switches += usize::from(summary.is_some_and(|summary| {
                summary.last_switch_reason == StrategicSwitchReason::Stalled
            }));
            let has_slot = state.live_bomb_count(seat) < usize::from(actor.bomb_capacity());
            if recovery[index].0.is_none()
                && has_slot
                && let Some(summary) = summary
                && summary.objective == StrategicObjectiveKind::Farm
                && matches!(
                    summary.phase,
                    StrategicPhase::Approach | StrategicPhase::Reengage
                )
                && summary.approach_cell == actor.position().cell()
                && let StrategicTarget::Block { cell } = summary.target
            {
                recovery[index] = (Some(cell), 0);
                recovery_seen[index] = true;
            }
            if let Some(subject) = recovery[index].0 {
                recovery[index].1 += 1;
                max_recovery_delay[index] = max_recovery_delay[index].max(recovery[index].1);
                let subject_completed = !state.world().arena().blocks().contains(&subject);
                if owned_bomb_this_tick[index] || subject_completed {
                    recovery_productive[index] = true;
                    recovery[index] = (None, 0);
                }
            }
        }
    }
    let admissions = driver
        .cpus
        .iter()
        .map(CpuController::admission_trace_for_test)
        .collect();
    RealWorkerOutcome {
        moved,
        owned_bomb_births,
        useful_upgrades,
        trace: planner.trace_for_test(),
        admissions,
        recovery_seen,
        recovery_productive,
        recovery_unresolved: recovery.map(|(subject, _)| subject.is_some()),
        max_recovery_delay,
        stalled_switches,
    }
}

#[test]
fn real_workers_record_cpu26_farm_recovery_with_bounded_plans() {
    for lairs in [false, true] {
        let outcome = run(lairs, CpuProfile::SPIKE_PITS_STANDARD, 5);
        assert_operational(&outcome);
        if lairs {
            // This clock records an attained Farm approach with a free bomb slot,
            // not proof that planting remained certified throughout the interval.
            // Keep the observed CPU26 delay as diagnostic evidence without
            // treating the 186-tick threshold as a safety or liveness contract.
            assert!(
                outcome
                    .max_recovery_delay
                    .into_iter()
                    .any(|delay| delay > 186)
                    || outcome.recovery_unresolved.into_iter().any(|value| value),
                "CPU26 no longer reproduces the diagnosed lair-on delay: {outcome:?}"
            );
        }
    }
}

#[test]
fn real_workers_record_cpu27_farm_recovery_with_bounded_plans() {
    for lairs in [false, true] {
        // This opt-in profile is checked at the production 60 Hz interval so
        // accelerated simulation cannot manufacture stale admissions.
        let outcome = run(lairs, CpuProfile::RELIABLE_V27_STANDARD, 16);
        assert_operational(&outcome);
    }
}

fn assert_operational(outcome: &RealWorkerOutcome) {
    assert!(outcome.moved.into_iter().all(|moved| moved), "{outcome:?}");
    assert!(
        outcome
            .owned_bomb_births
            .into_iter()
            .all(|births| births > 0),
        "{outcome:?}"
    );
    assert!(outcome.trace.delivered > 0, "{outcome:?}");
    assert_eq!(
        outcome.trace.delivered, outcome.trace.dequeued,
        "{outcome:?}"
    );
    assert_eq!(
        (outcome.trace.cancelled, outcome.trace.dropped),
        (0, 0),
        "{outcome:?}"
    );
    assert_eq!(outcome.trace.failed, 0, "{outcome:?}");
    assert!(
        outcome
            .trace
            .work
            .iter()
            .all(|spent| *spent <= WORK_ALLOWANCE),
        "{outcome:?}"
    );
    assert_eq!(outcome.admissions.len(), 3, "{outcome:?}");
    assert!(
        outcome
            .admissions
            .iter()
            .all(|trace| *trace == crate::game::rooms::cpu::AdmissionTrace::default()),
        "CPU had an admission rejection: {outcome:?}"
    );
    assert!(outcome.stalled_switches < 840 * 3, "{outcome:?}");
    assert!(
        outcome.recovery_seen.into_iter().any(|seen| seen),
        "{outcome:?}"
    );
}
