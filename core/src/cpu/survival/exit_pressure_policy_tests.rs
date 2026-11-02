use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::{CpuRng, MemoryKey, Observation, ObservedActor, WorkBudget};
use crate::game::{
    CellId, Direction, EffectKind, EntityId, Equipment, Position, SeatId, SpecialWeapon, Terrain,
};

use super::WORK_ALLOWANCE;
use super::exit_pressure_fixture::{CPU, NORTH_ORIGIN, START, WEST_ORIGIN, collected_basis_at};

fn approach(observation: &Observation, exit_coverage: bool) -> (Option<CellId>, u32) {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = if exit_coverage {
        crate::cpu::policy::plan_hard_pits_exit_pressure_v58(
            observation,
            &UtilityMemory::new(key),
            &mut CpuRng::from_seed(key, [58; 32]),
            &mut work,
        )
    } else {
        crate::cpu::policy::plan_hard_farm_pits_v53(
            observation,
            &UtilityMemory::new(key),
            &mut CpuRng::from_seed(key, [57; 32]),
            &mut work,
        )
    }
    .unwrap();
    let intent = planned.next_memory.strategic_intent().unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(
        intent.target,
        StrategicTarget::Opponent { seat: SeatId::One }
    );
    (intent.approach_cell, work.spent())
}

fn map(
    observation: Observation,
    change: impl FnOnce(&mut crate::cpu::ObservationParts),
) -> Observation {
    observation.map_parts(change).unwrap()
}

fn visible_actor(
    parts: &mut crate::cpu::ObservationParts,
    seat: SeatId,
) -> &mut crate::cpu::VisibleActor {
    parts
        .actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == seat => Some(actor),
            _ => None,
        })
        .unwrap()
}

#[test]
fn mirrored_geometry_promotes_the_mirrored_lateral_origin() {
    let original = collected_basis_at(START).observation;
    assert_eq!(approach(&original, true).0, Some(WEST_ORIGIN));
    let mirrored = map(original, |parts| {
        let cpu = visible_actor(parts, CPU);
        cpu.position = Position::center(CellId::new(9, 5));
        cpu.heading = Direction::West;
    });
    assert_eq!(approach(&mirrored, true).0, Some(CellId::new(9, 7)));
}

#[test]
fn equal_coverage_and_farther_coverage_retain_the_nearest_coordinate_baseline() {
    let equal = map(collected_basis_at(START).observation, |parts| {
        parts.arena.blocks.clear();
    });
    assert_eq!(approach(&equal, true).0, approach(&equal, false).0);

    let farther = map(collected_basis_at(START).observation, |parts| {
        let cpu = visible_actor(parts, CPU);
        cpu.position = Position::center(CellId::new(7, 3));
        cpu.heading = Direction::South;
    });
    assert_eq!(approach(&farther, true).0, approach(&farther, false).0);
}

#[test]
fn only_the_visible_rivals_enabled_wall_pass_opens_its_soft_blocked_branch() {
    let cpu_wall_pass = map(collected_basis_at(START).observation, |parts| {
        visible_actor(parts, CPU).equipment = Some(Equipment::WallPass);
    });
    assert_eq!(approach(&cpu_wall_pass, true).0, Some(WEST_ORIGIN));

    let rival_wall_pass = map(collected_basis_at(START).observation, |parts| {
        visible_actor(parts, SeatId::One).equipment = Some(Equipment::WallPass);
    });
    assert_eq!(approach(&rival_wall_pass, true).0, Some(NORTH_ORIGIN));
}

#[test]
fn piercing_promotes_an_origin_that_standard_blast_geometry_cannot_use() {
    let piercing = map(collected_basis_at(START).observation, |parts| {
        let south = CellId::new(7, 8);
        parts.arena.blocks.retain(|cell| *cell != south);
        let south_index = parts.arena.index(south).unwrap();
        parts.arena.terrain[south_index] = Terrain::Wall;
        parts.arena.blocks.push(CellId::new(6, 7));
        parts.arena.blocks.sort_unstable();
        visible_actor(parts, SeatId::One).equipment = Some(Equipment::WallPass);
        visible_actor(parts, CPU).weapon = Some(SpecialWeapon::Piercing);
    });
    assert_eq!(approach(&piercing, false).0, Some(NORTH_ORIGIN));
    assert_eq!(approach(&piercing, true).0, Some(WEST_ORIGIN));
}

#[test]
fn power_promotes_a_diagonal_origin_that_standard_blast_geometry_cannot_use() {
    let power = map(collected_basis_at(START).observation, |parts| {
        let diagonal = CellId::new(6, 6);
        let diagonal_index = parts.arena.index(diagonal).unwrap();
        parts.arena.terrain[diagonal_index] = Terrain::Floor;
        let cpu = visible_actor(parts, CPU);
        cpu.bomb_range = 2;
        cpu.weapon = Some(SpecialWeapon::Power);
    });
    assert_eq!(approach(&power, false).0, Some(NORTH_ORIGIN));
    assert_eq!(approach(&power, true).0, Some(CellId::new(6, 6)));
}

#[test]
fn golden_flame_must_remain_active_through_the_seven_tick_placement_offset() {
    let with_remaining = |remaining_ticks| {
        map(collected_basis_at(START).observation, |parts| {
            let cpu = visible_actor(parts, CPU);
            cpu.bomb_range = 1;
            cpu.effects.push(crate::cpu::ObservedEffect {
                kind: EffectKind::GoldenFlame,
                remaining_ticks,
            });
        })
    };
    assert_eq!(approach(&with_remaining(8), true).0, Some(WEST_ORIGIN));
    assert_eq!(
        approach(&with_remaining(7), true).0,
        Some(CellId::new(7, 6))
    );
}

#[test]
fn frozen_off_center_and_local_actor_ambiguity_fall_back_to_the_baseline() {
    let frozen = map(collected_basis_at(START).observation, |parts| {
        visible_actor(parts, SeatId::One).voluntary_actions_frozen = true;
    });
    let off_center = map(collected_basis_at(START).observation, |parts| {
        visible_actor(parts, SeatId::One).position.x += 1;
    });
    for observation in [frozen, off_center] {
        assert_eq!(
            approach(&observation, true).0,
            approach(&observation, false).0
        );
    }

    let mut parts = collected_basis_at(START).observation.into_parts();
    let mut local = visible_actor(&mut parts, SeatId::One).clone();
    local.id = EntityId(3);
    local.seat = SeatId::Three;
    local.position = Position::center(CellId::new(8, 7));
    local.alive = false;
    parts.actors.push(ObservedActor::Visible(local));
    let ambiguous = Observation::new_rules_v10(
        parts,
        crate::cpu::ObservedEnvironment::new(
            crate::game::StormCloudConfig::Disabled,
            None,
            vec![Some(0); 3],
        ),
        None,
        None,
    )
    .unwrap();
    assert_eq!(approach(&ambiguous, true).0, approach(&ambiguous, false).0);

    let mut parts = collected_basis_at(START).observation.into_parts();
    parts.actors.push(ObservedActor::Hidden {
        id: EntityId(3),
        seat: SeatId::Three,
        alive: true,
    });
    let hidden = Observation::new_rules_v10(
        parts,
        crate::cpu::ObservedEnvironment::new(
            crate::game::StormCloudConfig::Disabled,
            None,
            vec![Some(0), Some(0), None],
        ),
        None,
        None,
    )
    .unwrap();
    assert_eq!(approach(&hidden, true).0, approach(&hidden, false).0);
}
