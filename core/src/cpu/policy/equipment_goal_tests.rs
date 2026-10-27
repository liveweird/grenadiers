use crate::cpu::{
    CpuRng, Intent, MovementIntent, Observation, ObservedActor, ObservedPickup, WorkBudget,
};
use crate::game::{CellId, Direction, EntityId, Equipment, PickupKind, SeatId};

use super::equipment::classify;
use super::goals::{select, transit_allowed};
use super::tests::{key, observation};
use super::{UtilityMemory, plan};

fn actor(observation: &Observation) -> &crate::cpu::VisibleActor {
    let ObservedActor::Visible(actor) = &observation.parts().actors[1] else {
        unreachable!()
    };
    actor
}

fn with_equipment(position: CellId, equipment: Option<Equipment>) -> crate::cpu::ObservationParts {
    let mut parts = observation(position).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.equipment = equipment;
    parts
}

fn add_pickup(parts: &mut crate::cpu::ObservationParts, cell: CellId, kind: PickupKind) {
    parts.pickups.push(ObservedPickup { cell, kind });
}

fn guidance(observation: &Observation, current: CellId, seed: [u8; 32]) -> (Option<CellId>, u64) {
    let mut rng = CpuRng::from_seed(key(), seed);
    let selected = select(
        observation,
        actor(observation),
        current,
        &mut rng,
        &mut WorkBudget::new(200),
    )
    .map(|guidance| guidance.first_step);
    (selected, rng.draws())
}

#[test]
fn empty_or_kick_slot_seeks_punch_and_bracelet_at_equal_value() {
    let current = CellId::new(7, 7);
    for held in [None, Some(Equipment::Kick)] {
        for (kind, cell) in [
            (PickupKind::Punch, CellId::new(8, 7)),
            (PickupKind::Bracelet, CellId::new(7, 6)),
        ] {
            let mut parts = with_equipment(current, held);
            add_pickup(&mut parts, cell, kind);
            let observed = Observation::new(parts).unwrap();

            assert_eq!(classify(actor(&observed), kind), Some((Some(4), true)));
            assert_eq!(guidance(&observed, current, [31; 32]), (Some(cell), 0));
        }
    }
}

#[test]
fn punch_and_bracelet_tie_is_seeded_and_is_the_only_random_draw() {
    let current = CellId::new(7, 7);
    let mut parts = with_equipment(current, None);
    add_pickup(&mut parts, CellId::new(8, 7), PickupKind::Punch);
    add_pickup(&mut parts, CellId::new(7, 6), PickupKind::Bracelet);
    let observed = Observation::new(parts).unwrap();

    let first = guidance(&observed, current, [33; 32]);
    assert_eq!(first, guidance(&observed, current, [33; 32]));
    assert_eq!(first.1, 1);
}

#[test]
fn held_tactical_equipment_allows_only_duplicate_transit() {
    for (held, duplicate, replacement) in [
        (Equipment::Punch, PickupKind::Punch, PickupKind::Bracelet),
        (Equipment::Bracelet, PickupKind::Bracelet, PickupKind::Punch),
    ] {
        let mut parts = with_equipment(CellId::new(7, 7), Some(held));
        add_pickup(&mut parts, CellId::new(8, 7), duplicate);
        add_pickup(&mut parts, CellId::new(9, 7), replacement);
        add_pickup(&mut parts, CellId::new(7, 6), PickupKind::Kick);
        let observed = Observation::new(parts).unwrap();

        assert_eq!(classify(actor(&observed), duplicate), Some((None, true)));
        assert!(transit_allowed(
            &observed,
            actor(&observed),
            CellId::new(8, 7)
        ));
        for cell in [CellId::new(9, 7), CellId::new(7, 6)] {
            assert!(!transit_allowed(&observed, actor(&observed), cell));
        }
    }
}

#[test]
fn tactical_replacement_is_detoured_instead_of_used_as_transit() {
    let current = CellId::new(7, 7);
    let mut parts = with_equipment(current, Some(Equipment::Punch));
    add_pickup(&mut parts, CellId::new(8, 7), PickupKind::Bracelet);
    add_pickup(&mut parts, CellId::new(10, 7), PickupKind::BombCapacity);
    let observed = Observation::new(parts).unwrap();

    assert_ne!(
        guidance(&observed, current, [35; 32]).0,
        Some(CellId::new(8, 7))
    );
}

#[test]
fn remembered_step_is_revalidated_when_it_would_replace_tactical_equipment() {
    let current = CellId::new(7, 7);
    let next = CellId::new(8, 7);
    let mut parts = with_equipment(current, Some(Equipment::Bracelet));
    add_pickup(&mut parts, next, PickupKind::Punch);
    let observed = Observation::new(parts).unwrap();
    let memory = UtilityMemory::from_route(key(), &[next], false);
    let mut rng = CpuRng::from_seed(key(), [37; 32]);

    let planned = plan(&observed, &memory, &mut rng, &mut WorkBudget::new(100)).unwrap();

    assert_ne!(
        planned.intent.movement,
        MovementIntent::Direction(Direction::East)
    );
    assert_eq!(planned.intent.manual, None);
}

#[test]
fn buffs_and_upgrades_keep_priority_over_tactical_equipment() {
    let current = CellId::new(7, 7);
    for stronger in [
        PickupKind::Stopwatch,
        PickupKind::BombCapacity,
        PickupKind::WallPass,
        PickupKind::Heart,
    ] {
        let mut parts = with_equipment(current, Some(Equipment::Kick));
        add_pickup(&mut parts, CellId::new(8, 7), PickupKind::Punch);
        add_pickup(&mut parts, CellId::new(7, 9), stronger);
        let observed = Observation::new(parts).unwrap();

        assert_eq!(
            guidance(&observed, current, [39; 32]).0,
            Some(CellId::new(7, 8)),
            "{stronger:?}"
        );
    }
}

#[test]
fn tactical_goal_beats_block_openings_and_rival_pressure() {
    let current = CellId::new(7, 7);
    let mut parts = with_equipment(current, None);
    parts.arena.blocks.push(CellId::new(6, 7));
    add_pickup(&mut parts, CellId::new(8, 7), PickupKind::Bracelet);
    let ObservedActor::Visible(mut rival) = parts.actors[1].clone() else {
        unreachable!()
    };
    rival.id = EntityId(1);
    rival.seat = SeatId::One;
    rival.position = crate::game::Position::center(CellId::new(7, 5));
    parts.actors[0] = ObservedActor::Visible(rival);
    let observed = Observation::new(parts).unwrap();

    assert_eq!(
        guidance(&observed, current, [41; 32]).0,
        Some(CellId::new(8, 7))
    );
}

#[test]
fn heart_and_wall_pass_protection_and_upgrades_are_unchanged() {
    for (held, incoming, expected) in [
        (
            Some(Equipment::Punch),
            PickupKind::WallPass,
            (Some(7), true),
        ),
        (
            Some(Equipment::Bracelet),
            PickupKind::Heart,
            (Some(9), true),
        ),
        (Some(Equipment::Heart), PickupKind::WallPass, (None, false)),
        (Some(Equipment::WallPass), PickupKind::Heart, (None, false)),
    ] {
        let parts = with_equipment(CellId::new(7, 7), held);
        let observed = Observation::new(parts).unwrap();
        assert_eq!(classify(actor(&observed), incoming), Some(expected));
    }
}

#[test]
fn equipment_search_exhaustion_is_bounded_without_rng_or_input() {
    let current = CellId::new(7, 7);
    let mut parts = with_equipment(current, None);
    add_pickup(&mut parts, CellId::new(13, 7), PickupKind::Punch);
    let observed = Observation::new(parts).unwrap();
    let mut rng = CpuRng::from_seed(key(), [43; 32]);

    let planned = plan(
        &observed,
        &UtilityMemory::new(key()),
        &mut rng,
        &mut WorkBudget::new(1),
    )
    .unwrap();

    assert_eq!(planned.intent, Intent::STOP);
    assert_eq!(rng.draws(), 0);
}
