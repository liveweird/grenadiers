use crate::cpu::{
    CpuRng, MovementIntent, ObservedActor, ObservedEffect, ObservedPickup, WorkBudget,
};
use crate::game::effects::GOLDEN_FLAME_RANGE;
use crate::game::{
    CellId, Direction, EffectKind, EntityId, Equipment, PickupKind, SpeedLevel, Terrain,
};

use super::goals::{select, transit_allowed};
use super::tests::{key, observation};
use super::{UtilityMemory, plan};

fn cpu_actor(observation: &crate::cpu::Observation) -> &crate::cpu::VisibleActor {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
        .unwrap()
}

fn movement(observation: &crate::cpu::Observation, work: u32) -> MovementIntent {
    let mut rng = CpuRng::from_seed(key(), [19; 32]);
    plan(
        observation,
        &UtilityMemory::new(key()),
        &mut rng,
        &mut WorkBudget::new(work),
    )
    .unwrap()
    .intent
    .movement
}

fn add_pickup(parts: &mut crate::cpu::ObservationParts, column: u16, row: u16, kind: PickupKind) {
    parts.pickups.push(ObservedPickup {
        cell: CellId::new(column, row),
        kind,
    });
}

#[test]
fn farther_useful_pickup_wins_and_curse_is_not_a_transit_cell() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    add_pickup(&mut parts, 7, 6, PickupKind::ReverseControls);
    add_pickup(&mut parts, 9, 7, PickupKind::BombCapacity);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_eq!(
        movement(&observation, 80),
        MovementIntent::Direction(Direction::East)
    );
    assert!(!transit_allowed(
        &observation,
        cpu_actor(&observation),
        CellId::new(7, 6)
    ));
}

#[test]
fn undesirable_pickup_in_a_direct_lane_is_detoured_around() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    add_pickup(&mut parts, 8, 7, PickupKind::RemoteBomb);
    add_pickup(&mut parts, 10, 7, PickupKind::MovementSpeed);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_ne!(
        movement(&observation, 100),
        MovementIntent::Direction(Direction::East)
    );
}

#[test]
fn reachable_lower_value_goal_beats_an_enclosed_heart() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let heart = CellId::new(10, 7);
    for cell in [
        CellId::new(10, 6),
        CellId::new(11, 7),
        CellId::new(10, 8),
        CellId::new(9, 7),
    ] {
        let index =
            usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
        parts.arena.terrain[index] = Terrain::Wall;
    }
    parts.pickups.push(ObservedPickup {
        cell: heart,
        kind: PickupKind::Heart,
    });
    add_pickup(&mut parts, 6, 7, PickupKind::MovementSpeed);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_eq!(
        movement(&observation, 300),
        MovementIntent::Direction(Direction::West)
    );
}

#[test]
fn sole_enclosed_goal_falls_back_to_a_safe_neighbor() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let heart = CellId::new(10, 7);
    for cell in [
        CellId::new(10, 6),
        CellId::new(11, 7),
        CellId::new(10, 8),
        CellId::new(9, 7),
    ] {
        let index =
            usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
        parts.arena.terrain[index] = Terrain::Wall;
    }
    parts.pickups.push(ObservedPickup {
        cell: heart,
        kind: PickupKind::Heart,
    });
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_ne!(movement(&observation, 300), MovementIntent::Stop);
}

#[test]
fn harmless_zero_value_pickups_remain_open_corridor_transit() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.bomb_capacity = 8;
    actor.effects.push(ObservedEffect {
        kind: EffectKind::Shield,
        remaining_ticks: 299,
    });
    add_pickup(&mut parts, 8, 7, PickupKind::BombCapacity);
    add_pickup(&mut parts, 9, 7, PickupKind::Shield);
    add_pickup(&mut parts, 10, 7, PickupKind::MovementSpeed);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_eq!(
        movement(&observation, 100),
        MovementIntent::Direction(Direction::East)
    );
    assert!(transit_allowed(
        &observation,
        cpu_actor(&observation),
        CellId::new(8, 7)
    ));
    assert!(transit_allowed(
        &observation,
        cpu_actor(&observation),
        CellId::new(9, 7)
    ));
}

#[test]
fn held_heart_is_not_replaced_in_transit_by_unused_equipment() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.equipment = Some(Equipment::Heart);
    add_pickup(&mut parts, 8, 7, PickupKind::Punch);
    add_pickup(&mut parts, 10, 7, PickupKind::BombCapacity);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_ne!(
        movement(&observation, 100),
        MovementIntent::Direction(Direction::East)
    );
    assert!(!transit_allowed(
        &observation,
        cpu_actor(&observation),
        CellId::new(8, 7)
    ));
}

#[test]
fn remembered_ordinary_step_is_revalidated_after_bad_pickup_appears() {
    let current = CellId::new(7, 7);
    let next = CellId::new(8, 7);
    let mut parts = observation(current).into_parts();
    add_pickup(&mut parts, 8, 7, PickupKind::Teleporter);
    let observation = crate::cpu::Observation::new(parts).unwrap();
    let memory = UtilityMemory::from_route(key(), &[next], false);
    let mut rng = CpuRng::from_seed(key(), [21; 32]);
    let planned = plan(&observation, &memory, &mut rng, &mut WorkBudget::new(40)).unwrap();

    assert_ne!(
        planned.intent.movement,
        MovementIntent::Direction(Direction::East)
    );
    assert_eq!(planned.intent.manual, None);
}

#[test]
fn exact_goal_ties_are_the_only_reason_to_draw_randomness() {
    let current = CellId::new(7, 7);
    let mut unique_parts = observation(current).into_parts();
    add_pickup(&mut unique_parts, 8, 7, PickupKind::BombCapacity);
    let unique = crate::cpu::Observation::new(unique_parts).unwrap();
    let mut unique_rng = CpuRng::from_seed(key(), [25; 32]);
    let guidance = select(
        &unique,
        cpu_actor(&unique),
        current,
        &mut unique_rng,
        &mut WorkBudget::new(40),
    )
    .unwrap();
    assert_eq!(guidance.first_step, CellId::new(8, 7));
    assert_eq!(unique_rng.draws(), 0);

    let mut tied_parts = observation(current).into_parts();
    add_pickup(&mut tied_parts, 8, 7, PickupKind::BombCapacity);
    add_pickup(&mut tied_parts, 7, 6, PickupKind::BombCapacity);
    let tied = crate::cpu::Observation::new(tied_parts).unwrap();
    let mut tied_rng = CpuRng::from_seed(key(), [25; 32]);
    assert!(
        select(
            &tied,
            cpu_actor(&tied),
            current,
            &mut tied_rng,
            &mut WorkBudget::new(40),
        )
        .is_some()
    );
    assert_eq!(tied_rng.draws(), 1);
}

#[test]
fn hidden_rival_facts_do_not_change_deterministic_fallback() {
    let mut first_parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Hidden { id, .. } = &mut first_parts.actors[0] else {
        unreachable!()
    };
    *id = EntityId(31);
    let mut second_parts = first_parts.clone();
    let ObservedActor::Hidden { id, .. } = &mut second_parts.actors[0] else {
        unreachable!()
    };
    *id = EntityId(32);
    let first = crate::cpu::Observation::new(first_parts).unwrap();
    let second = crate::cpu::Observation::new(second_parts).unwrap();

    assert_eq!(movement(&first, 40), movement(&second, 40));
}

#[test]
fn exhausted_goal_search_stops_without_manual_input_or_extra_rng() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    add_pickup(&mut parts, 13, 7, PickupKind::BombCapacity);
    let observation = crate::cpu::Observation::new(parts).unwrap();
    let mut rng = CpuRng::from_seed(key(), [27; 32]);
    let planned = plan(
        &observation,
        &UtilityMemory::new(key()),
        &mut rng,
        &mut WorkBudget::new(1),
    )
    .unwrap();

    assert_eq!(planned.intent, crate::cpu::Intent::STOP);
    assert_eq!(rng.draws(), 0);
}

#[test]
fn candidate_preparation_is_charged_before_search_or_rng() {
    let current = CellId::new(7, 7);
    let mut parts = observation(current).into_parts();
    add_pickup(&mut parts, 8, 7, PickupKind::BombCapacity);
    let observation = crate::cpu::Observation::new(parts).unwrap();
    let mut rng = CpuRng::from_seed(key(), [29; 32]);
    let mut work = WorkBudget::new(2);

    assert!(
        select(
            &observation,
            cpu_actor(&observation),
            current,
            &mut rng,
            &mut work,
        )
        .is_none()
    );
    assert_eq!(work.spent(), 0);
    assert_eq!(rng.draws(), 0);
}

#[test]
fn all_twenty_five_pickup_kinds_have_explicit_transit_policy() {
    let harmful = [
        PickupKind::RemoteBomb,
        PickupKind::Landmine,
        PickupKind::Diarrhea,
        PickupKind::SlowFoot,
        PickupKind::ShortFlame,
        PickupKind::LongFuse,
        PickupKind::ReverseControls,
        PickupKind::InvisibleBombs,
        PickupKind::OilBarrel,
        PickupKind::Teleporter,
    ];
    let harmless = [
        PickupKind::BombCapacity,
        PickupKind::BlastRange,
        PickupKind::MovementSpeed,
        PickupKind::Kick,
        PickupKind::Punch,
        PickupKind::Bracelet,
        PickupKind::WallPass,
        PickupKind::Heart,
        PickupKind::Shield,
        PickupKind::Stopwatch,
        PickupKind::InvisibilityCloak,
        PickupKind::GoldenFlame,
        PickupKind::ShortFuse,
        PickupKind::PiercingBomb,
        PickupKind::PowerBomb,
    ];
    assert_eq!(harmful.len() + harmless.len(), 25);
    for (expected, kind) in harmful
        .into_iter()
        .map(|kind| (false, kind))
        .chain(harmless.into_iter().map(|kind| (true, kind)))
    {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        add_pickup(&mut parts, 8, 7, kind);
        let observation = crate::cpu::Observation::new(parts).unwrap();
        assert_eq!(
            transit_allowed(&observation, cpu_actor(&observation), CellId::new(8, 7)),
            expected,
            "{kind:?}"
        );
    }
}

#[test]
fn speed_cap_is_harmless_but_not_selected_as_a_goal() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.speed = SpeedLevel::Four;
    add_pickup(&mut parts, 8, 7, PickupKind::MovementSpeed);
    add_pickup(&mut parts, 7, 9, PickupKind::BlastRange);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_eq!(
        movement(&observation, 80),
        MovementIntent::Direction(Direction::South)
    );
}

#[test]
fn golden_flame_at_its_authoritative_range_is_harmless_not_a_goal() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.bomb_range = GOLDEN_FLAME_RANGE;
    add_pickup(&mut parts, 8, 7, PickupKind::GoldenFlame);
    add_pickup(&mut parts, 7, 9, PickupKind::MovementSpeed);
    let observation = crate::cpu::Observation::new(parts).unwrap();

    assert_eq!(
        movement(&observation, 80),
        MovementIntent::Direction(Direction::South)
    );
    assert!(transit_allowed(
        &observation,
        cpu_actor(&observation),
        CellId::new(8, 7)
    ));
}
