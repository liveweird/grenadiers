use crate::cpu::policy::{PressureMode, StrategicTarget, UtilityMemory};
use crate::cpu::survival::farm_yield_fixture::{
    BASELINE_ORIGIN, CPU, FARM_TARGET, HIGH_YIELD_ORIGIN, START, collected_basis_at,
};
use crate::cpu::{BombKnowledge, MemoryKey, Observation, ObservedActor, WorkBudget};
use crate::game::{
    BlastKind, CellId, EffectKind, OilBarrelState, PickupKind, Position, SpecialWeapon, Tick,
};

use super::super::{select_pits, select_pits_v59};

fn choose(
    observation: &Observation,
    memory: &UtilityMemory,
    farm_origin_recovery: bool,
    allowance: u32,
) -> (super::super::GuidanceV24, u32) {
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == CPU => Some(actor),
            _ => None,
        })
        .unwrap();
    let mut work = WorkBudget::new(allowance);
    let guidance = select_pits_v59(
        observation,
        actor,
        actor.position.cell().unwrap(),
        memory,
        &mut work,
        farm_origin_recovery,
    )
    .unwrap();
    (guidance, work.spent())
}

fn key(observation: &Observation) -> MemoryKey {
    MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    }
}

#[test]
fn retained_same_target_keeps_its_eligible_origin_and_deadlines() {
    let observation = collected_basis_at(START).observation;
    let (fresh, _) = choose(
        &observation,
        &UtilityMemory::new(key(&observation)),
        true,
        65_536,
    );
    assert_eq!(fresh.intent.approach_cell, Some(HIGH_YIELD_ORIGIN));
    let (retained, work) = choose(
        &observation,
        &UtilityMemory::new(key(&observation)).with_strategic_intent(fresh.intent),
        true,
        65_536,
    );
    assert_eq!(retained.intent.target, fresh.intent.target);
    assert_eq!(retained.intent.approach_cell, fresh.intent.approach_cell);
    assert_eq!(
        retained.intent.committed_until,
        fresh.intent.committed_until
    );
    assert_eq!(
        retained.intent.progress_deadline,
        fresh.intent.progress_deadline
    );
    assert_eq!(work, 753);
}

#[test]
fn failed_origin_is_skipped_until_its_existing_deadline_releases() {
    let observation = collected_basis_at(START).observation;
    let memory =
        UtilityMemory::new(key(&observation)).defer_farm_approach(HIGH_YIELD_ORIGIN, Tick(1));
    let (deferred, _) = choose(&observation, &memory, true, 65_536);
    assert_eq!(deferred.intent.approach_cell, Some(BASELINE_ORIGIN));

    let later = observation
        .map_parts(|parts| {
            parts.tick = Tick(200);
            parts.rules.running_updates = 200;
        })
        .unwrap();
    let (released, _) = choose(&later, &memory, true, 65_536);
    assert_eq!(released.intent.approach_cell, Some(HIGH_YIELD_ORIGIN));
}

#[test]
fn retained_origin_replacement_does_not_count_movement_toward_the_invalid_old_origin() {
    let observation = collected_basis_at(START)
        .observation
        .map_parts(|parts| {
            parts.tick = Tick(50);
            parts.rules.running_updates = 50;
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.position = Position::center(CellId::new(5, 6));
            actor.position.y += 400;
        })
        .unwrap();
    let (fresh, _) = choose(
        &observation,
        &UtilityMemory::new(key(&observation)),
        true,
        65_536,
    );
    assert_eq!(fresh.intent.approach_cell, Some(HIGH_YIELD_ORIGIN));

    let replaced = observation
        .map_parts(|parts| {
            parts.tick = Tick(60);
            parts.rules.running_updates = 60;
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.position.y += 80;
            let rival = parts
                .actors
                .iter_mut()
                .find_map(|actor| match actor {
                    ObservedActor::Visible(actor) if actor.seat != CPU && actor.alive => {
                        Some(actor)
                    }
                    _ => None,
                })
                .unwrap();
            rival.position = Position::center(HIGH_YIELD_ORIGIN);
        })
        .unwrap();
    let current_position = match &replaced.parts().actors[CPU.index()] {
        ObservedActor::Visible(actor) => actor.position,
        _ => unreachable!(),
    };
    let (retained, _) = choose(
        &replaced,
        &UtilityMemory::new(key(&replaced)).with_strategic_intent(fresh.intent),
        true,
        65_536,
    );
    assert_eq!(retained.intent.approach_cell, Some(BASELINE_ORIGIN));
    assert_eq!(
        retained.intent.progress_deadline,
        fresh.intent.progress_deadline
    );
    assert_eq!(retained.intent.last_cpu_position, current_position);
    assert_eq!(retained.intent.last_distance, 4);
    assert!(
        distance(current_position, HIGH_YIELD_ORIGIN)
            < distance(fresh.intent.last_cpu_position, HIGH_YIELD_ORIGIN)
    );
    assert!(
        distance(current_position, BASELINE_ORIGIN)
            > distance(fresh.intent.last_cpu_position, BASELINE_ORIGIN)
    );
}

fn distance(position: Position, cell: CellId) -> u32 {
    let target = Position::center(cell);
    position.x.abs_diff(target.x) + position.y.abs_diff(target.y)
}

#[test]
fn cautious_and_recovery_variants_share_the_fresh_yield_choice() {
    let observation = collected_basis_at(START).observation;
    for recovery in [false, true] {
        let (chosen, _) = choose(
            &observation,
            &UtilityMemory::new(key(&observation)),
            recovery,
            65_536,
        );
        assert_eq!(
            chosen.intent.target,
            StrategicTarget::Block { cell: FARM_TARGET }
        );
        assert_eq!(chosen.intent.approach_cell, Some(HIGH_YIELD_ORIGIN));
    }
}

#[test]
fn subcell_pose_and_a_farther_higher_yield_origin_do_not_change_equal_distance_priority() {
    let observation = collected_basis_at(START).observation;
    let subcell = observation
        .clone()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.position.x += 1;
        })
        .unwrap();
    let (chosen, _) = choose(&subcell, &UtilityMemory::new(key(&subcell)), true, 65_536);
    assert_eq!(chosen.intent.approach_cell, Some(HIGH_YIELD_ORIGIN));

    let farther_only = observation
        .map_parts(|parts| {
            parts.arena.blocks.retain(|cell| *cell != CellId::new(7, 7));
            parts.arena.blocks.push(CellId::new(3, 11));
            parts.arena.blocks.sort_unstable();
        })
        .unwrap();
    let (chosen, _) = choose(
        &farther_only,
        &UtilityMemory::new(key(&farther_only)),
        true,
        65_536,
    );
    assert_eq!(chosen.intent.approach_cell, Some(BASELINE_ORIGIN));
}

#[test]
fn placement_geometry_counts_standard_piercing_power_and_timed_range_exactly() {
    let base = collected_basis_at(START).observation;
    let cases = [
        ("standard", base.clone()),
        (
            "piercing",
            with_weapon(base.clone(), SpecialWeapon::Piercing),
        ),
        ("power", with_weapon(base.clone(), SpecialWeapon::Power)),
        (
            "golden",
            with_effect(base.clone(), EffectKind::GoldenFlame, 8),
        ),
        (
            "expired_golden",
            with_effect(base, EffectKind::GoldenFlame, 7),
        ),
    ];
    let expected = [
        (BlastKind::Standard, 3, 1, 2, 205),
        (BlastKind::Piercing, 3, 1, 2, 209),
        (BlastKind::Power, 3, 1, 2, 619),
        (BlastKind::Standard, 8, 1, 2, 207),
        (BlastKind::Standard, 3, 1, 2, 205),
    ];
    for ((_, observation), expected) in cases.into_iter().zip(expected) {
        assert_eq!(yields(&observation), expected);
    }
}

#[test]
fn dynamic_hazards_and_exposed_teleporter_disable_yield_scoring() {
    let original = collected_basis_at(START).observation;
    let mut scenes = Vec::new();
    scenes.push(
        original
            .clone()
            .map_parts(|parts| parts.bombs = BombKnowledge::Unavailable)
            .unwrap(),
    );
    scenes.push(
        original
            .clone()
            .map_parts(|parts| {
                parts.flames.push(crate::cpu::ObservedFlame {
                    cell: CellId::new(1, 2),
                    remaining_ticks: 5,
                });
            })
            .unwrap(),
    );
    scenes.push(
        original
            .clone()
            .map_parts(|parts| {
                parts.oil.push(crate::cpu::ObservedOil {
                    cell: CellId::new(1, 2),
                    state: OilBarrelState::Unlit,
                });
            })
            .unwrap(),
    );
    scenes.push(
        original
            .clone()
            .map_parts(|parts| {
                parts.pickups.push(crate::cpu::ObservedPickup {
                    cell: CellId::new(1, 2),
                    kind: PickupKind::Teleporter,
                });
            })
            .unwrap(),
    );
    for scene in scenes {
        assert!(!super::supported_scene(
            &scene,
            &mut WorkBudget::new(65_536)
        ));
    }

    let ordinary_pickup = original
        .map_parts(|parts| {
            parts.pickups.push(crate::cpu::ObservedPickup {
                cell: CellId::new(1, 2),
                kind: PickupKind::BombCapacity,
            });
        })
        .unwrap();
    assert!(super::supported_scene(
        &ordinary_pickup,
        &mut WorkBudget::new(65_536)
    ));
}

fn with_weapon(observation: Observation, weapon: SpecialWeapon) -> Observation {
    observation
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.weapon = Some(weapon);
        })
        .unwrap()
}

fn with_effect(observation: Observation, kind: EffectKind, remaining_ticks: u16) -> Observation {
    observation
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.effects.push(crate::cpu::ObservedEffect {
                kind,
                remaining_ticks,
            });
        })
        .unwrap()
}

fn yields(observation: &Observation) -> (BlastKind, u16, u8, u8, u32) {
    let ObservedActor::Visible(actor) = &observation.parts().actors[CPU.index()] else {
        unreachable!()
    };
    let (kind, range) = super::super::super::pressure::weapon_geometry(observation, actor).unwrap();
    let mut work = WorkBudget::new(65_536);
    let geometry =
        crate::cpu::survival::ObservedBlastGeometry::new(observation, &mut work).unwrap();
    let baseline = geometry
        .visible_block_yield(BASELINE_ORIGIN, range, kind, FARM_TARGET, &mut work)
        .unwrap();
    let high = geometry
        .visible_block_yield(HIGH_YIELD_ORIGIN, range, kind, FARM_TARGET, &mut work)
        .unwrap();
    (kind, range, baseline, high, work.spent())
}

#[test]
fn partial_optional_comparison_charges_work_and_preserves_the_exact_baseline() {
    let observation = collected_basis_at(START).observation;
    let actor = match &observation.parts().actors[CPU.index()] {
        ObservedActor::Visible(actor) => actor,
        _ => unreachable!(),
    };
    let memory = UtilityMemory::new(key(&observation));
    let mut baseline_work = WorkBudget::new(5_200);
    let baseline = select_pits(
        &observation,
        actor,
        actor.position.cell().unwrap(),
        &memory,
        &mut baseline_work,
        PressureMode::ExitCoverage,
        true,
    )
    .unwrap();
    let mut work = WorkBudget::new(5_200);
    let selected = select_pits_v59(
        &observation,
        actor,
        actor.position.cell().unwrap(),
        &memory,
        &mut work,
        true,
    )
    .unwrap();
    assert_eq!(selected, baseline);
    assert_eq!(
        selected.intent.target,
        StrategicTarget::Block { cell: FARM_TARGET }
    );
    assert_eq!(selected.intent.approach_cell, Some(BASELINE_ORIGIN));
    assert_eq!(selected.first_step, CellId::new(4, 5));
    assert_eq!(selected.intent.score, 2);
    assert_eq!(baseline_work.spent(), 753);
    assert_eq!(work.spent(), 756);
}
