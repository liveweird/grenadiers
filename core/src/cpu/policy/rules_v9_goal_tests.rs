use crate::cpu::{
    CpuRng, MemoryKey, Observation, ObservedActor, ObservedEffect, ObservedEnvironment,
    ObservedPickup, WorkBudget,
};
use crate::game::{
    CellId, EffectKind, EnabledItems, EntityId, PickupKind, Position, SeatId, SpecialWeapon,
    StormCloudConfig,
};

use super::goals::select;
use super::tests::{key, observation};

fn rules_v9(cpu: CellId, rival: CellId) -> Observation {
    let mut parts = observation(cpu).into_parts();
    parts.rules.version = 9;
    let mut visible = match parts.actors[1].clone() {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => unreachable!(),
    };
    visible.id = EntityId(1);
    visible.seat = SeatId::One;
    visible.position = Position::center(rival);
    parts.actors[0] = ObservedActor::Visible(visible);
    Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0), Some(0)]),
        None,
    )
    .unwrap()
}

fn guidance(observation: &Observation) -> CellId {
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == SeatId::Two => Some(actor),
            _ => None,
        })
        .unwrap();
    select(
        observation,
        actor,
        actor.position.cell().unwrap(),
        &mut CpuRng::from_seed(key(), [61; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
    .first_step
}

#[test]
fn rules_v9_seeks_a_reachable_firing_position_without_entering_the_rival_cell() {
    let observed = rules_v9(CellId::new(7, 7), CellId::new(11, 7));
    assert_eq!(guidance(&observed), CellId::new(8, 7));

    let adjacent = rules_v9(CellId::new(7, 7), CellId::new(8, 7));
    assert_ne!(guidance(&adjacent), CellId::new(8, 7));
}

#[test]
fn existing_useful_pickup_values_still_beat_rules_v9_pressure() {
    let mut parts = rules_v9(CellId::new(7, 7), CellId::new(11, 7)).into_parts();
    parts.pickups.push(ObservedPickup {
        cell: CellId::new(7, 6),
        kind: PickupKind::BombCapacity,
    });
    let observed = Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0), Some(0)]),
        None,
    )
    .unwrap();
    assert_eq!(guidance(&observed), CellId::new(7, 6));
}

fn power_scene(rivals: &[CellId], golden: bool) -> Observation {
    let mut parts = rules_v9(CellId::new(7, 7), rivals[0]).into_parts();
    let enabled = EnabledItems::from_bits(
        EnabledItems::POWER_BOMB.bits()
            | if golden {
                EnabledItems::GOLDEN_FLAME.bits()
            } else {
                0
            },
    )
    .unwrap();
    parts.rules.settings = parts.rules.settings.with_enabled_items(enabled);
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.weapon = Some(SpecialWeapon::Power);
    cpu.bomb_range = if golden { 2 } else { 8 };
    if golden {
        cpu.effects.push(ObservedEffect {
            kind: EffectKind::GoldenFlame,
            remaining_ticks: 8,
        });
    }
    for (index, cell) in rivals.iter().copied().enumerate().skip(1) {
        let ObservedActor::Visible(mut rival) = parts.actors[0].clone() else {
            unreachable!()
        };
        rival.id = EntityId(u32::try_from(index + 2).unwrap());
        rival.seat = SeatId::ALL[index + 1];
        rival.position = Position::center(cell);
        parts.actors.push(ObservedActor::Visible(rival));
    }
    let timers = vec![Some(0); parts.actors.len()];
    Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, timers),
        None,
    )
    .unwrap()
}

fn assert_power_progress(observation: &Observation) {
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == SeatId::Two => Some(actor),
            _ => None,
        })
        .unwrap();
    let mut guidance_work = WorkBudget::new(65_536);
    let selected = select(
        observation,
        actor,
        actor.position.cell().unwrap(),
        &mut CpuRng::from_seed(key(), [62; 32]),
        &mut guidance_work,
    );
    assert!(selected.is_some());

    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let plan = super::plan_respawn(
        observation,
        &super::UtilityMemory::new(key),
        &mut CpuRng::from_seed(key, [63; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_ne!(plan.intent.movement, crate::cpu::MovementIntent::Stop);
}

#[test]
fn max_range_power_pressure_keeps_full_policy_progress_within_allowance() {
    assert_power_progress(&power_scene(&[CellId::new(13, 11)], true));
}

#[test]
fn power_pressure_treats_two_or_three_visible_rivals_without_exhausting_guidance() {
    for rivals in [
        &[CellId::new(13, 11), CellId::new(1, 1)][..],
        &[CellId::new(13, 11), CellId::new(1, 1), CellId::new(13, 1)][..],
    ] {
        assert_power_progress(&power_scene(rivals, false));
    }
}
