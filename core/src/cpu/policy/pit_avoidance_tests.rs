use crate::cpu::{
    CpuRng, Observation, ObservedActor, ObservedEnvironment, ObservedPickup, ObservedSpikePits,
    WorkBudget,
};
use crate::game::{
    CellId, Direction, EntityId, PickupKind, Position, SeatId, SpikePitPhase, SpikePitView,
    StormCloudConfig,
};
use crate::room::{SpikePitConfig, SpikePitSettings};

use super::planner::{plan_farm_v38, plan_hard_farm_v39, plan_hard_v28, plan_spike_pits_v26};
use super::tests::{key, observation};
use super::{
    ControlPlan, PolicyError, StrategicObjectiveKind, UtilityMemory, plan_farm_pits_v52,
    plan_hard_farm_pits_v53, plan_hard_pits_cautious_v53, plan_spike_pits_cautious_v52,
};

const FARM_PIT: CellId = CellId::new(3, 7);
const FARM_BLOCK: CellId = CellId::new(2, 7);
const FARM_PITS: [CellId; 4] = [
    CellId::new(3, 5),
    CellId::new(11, 5),
    FARM_PIT,
    CellId::new(11, 7),
];
type PlanFn = fn(
    &Observation,
    &UtilityMemory,
    &mut CpuRng,
    &mut WorkBudget,
) -> Result<ControlPlan, PolicyError>;

fn rules10(
    cpu: CellId,
    blocks: Vec<CellId>,
    pits: &[CellId],
    rival: Option<CellId>,
) -> Observation {
    let mut parts = observation(cpu).into_parts();
    parts.rules.version = 10;
    parts.arena.blocks = blocks;
    if let Some(cell) = rival {
        let ObservedActor::Visible(template) = &parts.actors[1] else {
            unreachable!()
        };
        let mut actor = template.clone();
        actor.id = EntityId(1);
        actor.seat = SeatId::One;
        actor.position = Position::center(cell);
        parts.actors[0] = ObservedActor::Visible(actor);
    }
    let timers = parts
        .actors
        .iter()
        .map(|actor| matches!(actor, ObservedActor::Visible(_)).then_some(0))
        .collect();
    let environment = ObservedEnvironment::new(StormCloudConfig::Disabled, None, timers);
    let overlay = (!pits.is_empty()).then(|| {
        ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            pits.iter()
                .copied()
                .map(|cell| SpikePitView {
                    cell,
                    phase: SpikePitPhase::Idle,
                    remaining_ticks: 0,
                })
                .collect(),
        )
    });
    Observation::new_rules_v10(parts, environment, None, overlay).unwrap()
}

fn run(planner: PlanFn, observed: &Observation, memory: UtilityMemory, seed: u8) -> ControlPlan {
    let mut rng = CpuRng::from_seed(key(), [seed; 32]);
    planner(observed, &memory, &mut rng, &mut WorkBudget::new(65_536)).unwrap()
}

#[test]
fn farm_pit_is_neither_the_selected_approach_nor_the_route_through_it() {
    let observed = rules10(CellId::new(5, 7), vec![FARM_BLOCK], &FARM_PITS, None);
    let legacy = run(plan_farm_v38, &observed, UtilityMemory::new(key()), 52);
    assert_eq!(
        legacy.next_memory.strategic_intent().unwrap().approach_cell,
        Some(FARM_PIT)
    );
    let cautious = run(plan_farm_pits_v52, &observed, UtilityMemory::new(key()), 52);
    let intent = cautious.next_memory.strategic_intent().unwrap();
    assert_eq!(
        intent.target,
        super::StrategicTarget::Block { cell: FARM_BLOCK }
    );
    assert_eq!(intent.approach_cell, Some(CellId::new(2, 6)));
    assert!(!cautious.next_memory.route().contains(&Some(FARM_PIT)));
}

#[test]
fn immediate_navigation_routes_around_a_pit_instead_of_crossing_it() {
    let mut observed = rules10(CellId::new(4, 7), Vec::new(), &FARM_PITS, None).into_parts();
    observed.pickups.push(ObservedPickup {
        cell: CellId::new(1, 7),
        kind: PickupKind::BombCapacity,
    });
    let environment =
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![None, Some(0)]);
    let overlay = ObservedSpikePits::new(
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        FARM_PITS
            .map(|cell| SpikePitView {
                cell,
                phase: SpikePitPhase::Idle,
                remaining_ticks: 0,
            })
            .into(),
    );
    let observed = Observation::new_rules_v10(observed, environment, None, Some(overlay)).unwrap();
    let legacy = run(
        plan_spike_pits_v26,
        &observed,
        UtilityMemory::new(key()),
        52,
    );
    assert_eq!(
        legacy.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::West)
    );
    let cautious = run(
        plan_spike_pits_cautious_v52,
        &observed,
        UtilityMemory::new(key()),
        52,
    );
    assert_ne!(
        cautious.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::West)
    );
    assert!(!cautious.next_memory.route().contains(&Some(FARM_PIT)));
}

#[test]
fn retained_route_and_target_are_rechecked_against_pit_sites() {
    let observed = rules10(CellId::new(5, 7), vec![FARM_BLOCK], &FARM_PITS, None);
    let route = UtilityMemory::from_route(key(), &[FARM_PIT], false);
    let legacy = run(plan_farm_v38, &observed, route, 52);
    assert_eq!(
        legacy.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::West)
    );
    let cautious = run(plan_farm_pits_v52, &observed, route, 52);
    assert!(!cautious.next_memory.route().contains(&Some(FARM_PIT)));

    let legacy_goal = run(plan_farm_v38, &observed, UtilityMemory::new(key()), 52)
        .next_memory
        .strategic_intent()
        .unwrap();
    let retained = UtilityMemory::new(key()).with_strategic_intent(legacy_goal);
    let cautious = run(plan_farm_pits_v52, &observed, retained, 52);
    let intent = cautious.next_memory.strategic_intent().unwrap();
    assert_eq!(intent.target, legacy_goal.target);
    assert_ne!(intent.approach_cell, Some(FARM_PIT));
}

#[test]
fn retained_route_pit_scan_obeys_the_exact_work_boundary() {
    let observed = rules10(CellId::new(5, 7), Vec::new(), &FARM_PITS, None);
    let memory = UtilityMemory::from_route(key(), &[CellId::new(5, 6)], false);
    let mut exact_rng = CpuRng::from_seed(key(), [52; 32]);
    let mut exact_work = WorkBudget::new(5);
    let exact =
        plan_spike_pits_cautious_v52(&observed, &memory, &mut exact_rng, &mut exact_work).unwrap();
    assert_eq!(
        exact.intent.movement,
        crate::cpu::MovementIntent::Direction(Direction::North)
    );
    assert_eq!(exact_work.spent(), 5);

    let mut short_rng = CpuRng::from_seed(key(), [52; 32]);
    let mut short_work = WorkBudget::new(4);
    let short =
        plan_spike_pits_cautious_v52(&observed, &memory, &mut short_rng, &mut short_work).unwrap();
    assert_eq!(short.intent.movement, crate::cpu::MovementIntent::Stop);
    assert_eq!(short_work.spent(), 1);
}

#[test]
fn hard_pressure_does_not_select_the_current_pit_as_its_attack_origin() {
    let center_pits = [CellId::new(7, 5), CellId::new(7, 7)];
    let mut observed = rules10(
        CellId::new(7, 7),
        Vec::new(),
        &center_pits,
        Some(CellId::new(10, 7)),
    )
    .into_parts();
    let ObservedActor::Visible(cpu) = &mut observed.actors[1] else {
        unreachable!()
    };
    cpu.bomb_range = 4;
    let observed = Observation::new_rules_v10(
        observed,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            center_pits
                .map(|cell| SpikePitView {
                    cell,
                    phase: SpikePitPhase::Idle,
                    remaining_ticks: 0,
                })
                .into(),
        )),
    )
    .unwrap();
    let cautious = run(
        plan_hard_pits_cautious_v53,
        &observed,
        UtilityMemory::new(key()),
        53,
    );
    let intent = cautious.next_memory.strategic_intent().unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
    assert_ne!(intent.approach_cell, Some(CellId::new(7, 7)));
}

#[test]
fn fallback_with_multiple_neighbors_excludes_every_pit_site() {
    let side_pits = [
        CellId::new(6, 5),
        CellId::new(8, 5),
        CellId::new(6, 7),
        CellId::new(8, 7),
    ];
    let observed = rules10(CellId::new(7, 7), Vec::new(), &side_pits, None);
    for seed in 0..8 {
        let cautious = run(
            plan_spike_pits_cautious_v52,
            &observed,
            UtilityMemory::new(key()),
            seed,
        );
        assert_eq!(
            cautious
                .next_memory
                .strategic_intent()
                .unwrap()
                .approach_cell,
            Some(CellId::new(7, 6))
        );
        assert!(
            !side_pits
                .iter()
                .any(|pit| cautious.next_memory.route().contains(&Some(*pit)))
        );
    }
}

#[test]
fn disabled_pits_preserve_each_base_plan_rng_and_work_exactly() {
    let observed = rules10(
        CellId::new(5, 7),
        vec![FARM_BLOCK],
        &[],
        Some(CellId::new(10, 7)),
    );
    for (base, cautious) in [
        (plan_farm_v38 as PlanFn, plan_farm_pits_v52 as PlanFn),
        (plan_hard_farm_v39, plan_hard_farm_pits_v53),
        (plan_spike_pits_v26, plan_spike_pits_cautious_v52),
        (plan_hard_v28, plan_hard_pits_cautious_v53),
    ] {
        let mut base_rng = CpuRng::from_seed(key(), [53; 32]);
        let mut cautious_rng = base_rng.clone();
        let mut base_work = WorkBudget::new(65_536);
        let mut cautious_work = WorkBudget::new(65_536);
        let base_plan = base(
            &observed,
            &UtilityMemory::new(key()),
            &mut base_rng,
            &mut base_work,
        )
        .unwrap();
        let cautious_plan = cautious(
            &observed,
            &UtilityMemory::new(key()),
            &mut cautious_rng,
            &mut cautious_work,
        )
        .unwrap();
        assert_eq!(cautious_plan, base_plan);
        assert_eq!(cautious_rng.draws(), base_rng.draws());
        assert_eq!(cautious_work.spent(), base_work.spent());
    }
}
