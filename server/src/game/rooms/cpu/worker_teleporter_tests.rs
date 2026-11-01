use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use grenadiers_core::cpu::{
    Observation, ObservedActor, ObservedEnvironment, ObservedPickup, VisibleActor,
};
use grenadiers_core::game::{
    CellId, Direction, EnabledItems, EntityId, PickupKind, Position, SeatId, SpeedLevel,
};
use grenadiers_core::room::CpuPolicy;

use super::{CpuPlanner, tests::spike_job, worker::WorkerPlan};

#[test]
fn old_profiles_never_emit_the_new_typed_step() {
    for policy in [CpuPolicy::UtilityV44, CpuPolicy::UtilityV45] {
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert!(report.work[0] <= WORK_ALLOWANCE);
        assert!(matches!(
            result.recv().unwrap().outcome,
            WorkerPlan::Ordinary { .. }
        ));
    }
}

#[test]
fn opted_in_profiles_preserve_the_ordinary_fallback_without_a_target() {
    for policy in [CpuPolicy::UtilityV46, CpuPolicy::UtilityV47] {
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert!(report.work[0] <= WORK_ALLOWANCE);
        assert!(matches!(
            result.recv().unwrap().outcome,
            WorkerPlan::Ordinary { .. }
        ));
    }
}

#[test]
fn opted_in_worker_emits_a_bounded_typed_approach_for_an_exposed_target() {
    let planner = CpuPlanner::without_workers();
    let (mut job, result) = spike_job(CpuPolicy::UtilityV46);
    job.observation = teleporter_observation();
    job.hazard_memory = grenadiers_core::cpu::forecast::ObservedHazardMemory::new(job.memory.key());
    job.hazard_memory.observe(&job.observation).unwrap();
    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (1, 0));
    assert!(report.work[0] <= WORK_ALLOWANCE);
    assert!(matches!(
        result.recv().unwrap().outcome,
        WorkerPlan::TeleporterStep(_)
    ));
}

pub(super) fn teleporter_observation() -> Observation {
    let (job, _result) = spike_job(CpuPolicy::UtilityV46);
    let mut parts = job.observation.into_parts();
    {
        parts.rules.settings = parts.rules.settings.with_enabled_items(EnabledItems::ALL);
        parts.arena.blocks.clear();
        parts.actors = SeatId::DUEL
            .map(|seat| {
                ObservedActor::Visible(VisibleActor {
                    id: EntityId(seat.index() as u32 + 1),
                    seat,
                    position: Position::center(if seat == parts.recipient {
                        CellId::new(7, 7)
                    } else {
                        CellId::new(2, 2)
                    }),
                    heading: Direction::South,
                    speed: SpeedLevel::Base,
                    alive: true,
                    bomb_capacity: 1,
                    bomb_range: 2,
                    live_bomb_count: 0,
                    equipment: None,
                    weapon: None,
                    effects: Vec::new(),
                    damage_grace_remaining_ticks: 0,
                    voluntary_actions_frozen: false,
                    last_teleport_tick: None,
                    unknown: grenadiers_core::cpu::UnknownActorFacts::ALL,
                })
            })
            .into();
        parts.pickups = vec![ObservedPickup {
            cell: CellId::new(7, 9),
            kind: PickupKind::Teleporter,
        }];
    }
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            grenadiers_core::game::StormCloudConfig::Disabled,
            None,
            vec![Some(0); 2],
        ),
        None,
        None,
    )
    .unwrap()
}
