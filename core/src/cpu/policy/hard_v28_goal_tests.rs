use crate::cpu::{
    Observation, ObservedActor, ObservedEffect, ObservedEnvironment, ObservedOil, ObservedPickup,
    WorkBudget,
};
use crate::game::{
    CellId, EffectKind, EntityId, OilBarrelState, PickupKind, Position, SeatId, SpecialWeapon,
    StormCloudConfig,
};

use super::goals::{select_v24, select_v25, select_v28};
use super::tests::{key, observation};
use super::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};

fn scene(rivals: &[CellId]) -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.version = 10;
    parts.arena.blocks = vec![CellId::new(3, 3)];
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.bomb_range = 4;
    let template = cpu.clone();
    parts.actors[0] = rival(&template, SeatId::One, EntityId(1), rivals[0]);
    for (index, cell) in rivals.iter().copied().enumerate().skip(1) {
        parts.actors.push(rival(
            &template,
            SeatId::ALL[index + 1],
            EntityId(u32::try_from(index + 2).unwrap()),
            cell,
        ));
    }
    let timers = vec![Some(0); parts.actors.len()];
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, timers),
        None,
        None,
    )
    .unwrap()
}

fn rival(
    template: &crate::cpu::VisibleActor,
    seat: SeatId,
    id: EntityId,
    cell: CellId,
) -> ObservedActor {
    let mut rival = template.clone();
    rival.id = id;
    rival.seat = seat;
    rival.position = Position::center(cell);
    rival.weapon = None;
    ObservedActor::Visible(rival)
}

fn select(
    observation: &Observation,
    version: u8,
    memory: UtilityMemory,
    allowance: u32,
) -> (super::goals::GuidanceV24, u32) {
    let actor = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == SeatId::Two => Some(actor),
            _ => None,
        })
        .unwrap();
    let mut work = WorkBudget::new(allowance);
    let guidance = match version {
        24 => select_v24(
            observation,
            actor,
            actor.position.cell().unwrap(),
            &memory,
            &mut work,
        ),
        25 => select_v25(
            observation,
            actor,
            actor.position.cell().unwrap(),
            &memory,
            &mut work,
        ),
        28 => select_v28(
            observation,
            actor,
            actor.position.cell().unwrap(),
            &memory,
            &mut work,
        ),
        _ => unreachable!(),
    }
    .unwrap();
    (guidance, work.spent())
}

#[test]
fn v28_alone_values_exact_range_pressure_above_farming() {
    let observed = scene(&[CellId::new(10, 7)]);
    for version in [24, 25] {
        let (guidance, _) = select(&observed, version, UtilityMemory::new(key()), 65_536);
        assert_eq!(guidance.intent.objective, StrategicObjectiveKind::Farm);
    }
    let (guidance, spent) = select(&observed, 28, UtilityMemory::new(key()), 65_536);
    assert_eq!(guidance.intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(
        guidance.intent.target,
        StrategicTarget::Opponent { seat: SeatId::One }
    );
    assert_eq!(guidance.intent.approach_cell, Some(CellId::new(7, 7)));
    assert!(spent <= 65_536);
}

#[test]
fn far_rival_pressure_probes_firing_origins_instead_of_nearby_floors() {
    let mut parts = scene(&[CellId::new(13, 11)]).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.position = Position::center(CellId::new(1, 1));
    cpu.bomb_range = 2;
    let environment = ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]);

    for farm in [false, true] {
        parts.arena.blocks = if farm {
            vec![CellId::new(3, 3)]
        } else {
            Vec::new()
        };
        let observed =
            Observation::new_rules_v10(parts.clone(), environment.clone(), None, None).unwrap();
        let (legacy, _) = select(&observed, 24, UtilityMemory::new(key()), 65_536);
        assert_eq!(
            legacy.intent.objective,
            if farm {
                StrategicObjectiveKind::Farm
            } else {
                StrategicObjectiveKind::Attack
            }
        );

        let (hard, spent) = select(&observed, 28, UtilityMemory::new(key()), 65_536);
        assert_eq!(hard.intent.objective, StrategicObjectiveKind::Attack);
        assert_eq!(
            hard.intent.target,
            StrategicTarget::Opponent { seat: SeatId::One }
        );
        let origin = hard.intent.approach_cell.unwrap();
        assert!(origin.column.abs_diff(13) + origin.row.abs_diff(11) <= 2);
        assert!(spent <= 65_536);
    }
}

#[test]
fn pickup_and_exact_obstructions_still_beat_or_reject_pressure() {
    let mut parts = scene(&[CellId::new(10, 7)]).into_parts();
    parts.pickups.push(ObservedPickup {
        cell: CellId::new(7, 6),
        kind: PickupKind::BombCapacity,
    });
    let environment = ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]);
    let pickup = Observation::new_rules_v10(parts, environment, None, None).unwrap();
    assert_eq!(
        select(&pickup, 28, UtilityMemory::new(key()), 65_536)
            .0
            .intent
            .objective,
        StrategicObjectiveKind::Collect
    );

    for obstruct in [true, false] {
        let mut parts = scene(&[CellId::new(10, 7)]).into_parts();
        for cell in [
            CellId::new(10, 6),
            CellId::new(11, 7),
            CellId::new(10, 8),
            CellId::new(9, 7),
        ] {
            if obstruct {
                let index = parts.arena.index(cell).unwrap();
                parts.arena.terrain[index] = crate::game::Terrain::Wall;
            } else {
                parts.oil.push(ObservedOil {
                    cell,
                    state: OilBarrelState::Unlit,
                });
            }
        }
        let observed = Observation::new_rules_v10(
            parts,
            ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            select(&observed, 28, UtilityMemory::new(key()), 65_536)
                .0
                .intent
                .objective,
            StrategicObjectiveKind::Farm
        );
    }
}

#[test]
fn expired_range_hidden_rivals_and_low_work_fall_back_without_false_attack() {
    let mut parts = scene(&[CellId::new(10, 7)]).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.bomb_range = 2;
    cpu.effects.push(ObservedEffect {
        kind: EffectKind::GoldenFlame,
        remaining_ticks: 7,
    });
    for cell in [
        CellId::new(7, 6),
        CellId::new(8, 7),
        CellId::new(7, 8),
        CellId::new(6, 7),
    ] {
        parts.pickups.push(ObservedPickup {
            cell,
            kind: PickupKind::Diarrhea,
        });
    }
    let observed = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
        None,
    )
    .unwrap();
    assert_ne!(
        select(&observed, 28, UtilityMemory::new(key()), 65_536)
            .0
            .intent
            .objective,
        StrategicObjectiveKind::Attack
    );

    let mut parts = scene(&[CellId::new(10, 7)]).into_parts();
    parts.actors[0] = ObservedActor::Hidden {
        id: EntityId(1),
        seat: SeatId::One,
        alive: true,
    };
    let hidden = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![None, Some(0)]),
        None,
        None,
    )
    .unwrap();
    assert_ne!(
        select(&hidden, 28, UtilityMemory::new(key()), 65_536)
            .0
            .intent
            .objective,
        StrategicObjectiveKind::Attack
    );

    let mut parts = scene(&[CellId::new(10, 7)]).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(crate::game::EnabledItems::EMPTY);
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.weapon = Some(SpecialWeapon::Remote);
    let unsupported = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        None,
        None,
    )
    .unwrap();
    assert_ne!(
        select(&unsupported, 28, UtilityMemory::new(key()), 65_536)
            .0
            .intent
            .objective,
        StrategicObjectiveKind::Attack
    );
    let (fallback, spent) = select(
        &scene(&[CellId::new(10, 7)]),
        28,
        UtilityMemory::new(key()),
        3_000,
    );
    assert_eq!(fallback.intent.objective, StrategicObjectiveKind::Farm);
    assert!(spent <= 3_000);
}

#[test]
fn power_pressure_is_bounded_and_retained_without_ab_switches() {
    let mut parts =
        scene(&[CellId::new(10, 10), CellId::new(4, 10), CellId::new(10, 4)]).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(crate::game::EnabledItems::POWER_BOMB);
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.weapon = Some(SpecialWeapon::Power);
    cpu.bomb_range = 8;
    let mut observed = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 4]),
        None,
        None,
    )
    .unwrap();
    let (first, first_work) = select(&observed, 28, UtilityMemory::new(key()), 65_536);
    assert_eq!(first.intent.objective, StrategicObjectiveKind::Attack);
    assert!(first_work <= 65_536);
    let deadline = first.intent.progress_deadline;
    let commitment = first.intent.committed_until;
    let mut memory = UtilityMemory::new(key()).with_strategic_intent(first.intent);
    for tick in [11, 12, 130] {
        observed = observed
            .map_parts(|parts| {
                parts.tick = crate::game::Tick(tick);
                parts.rules.running_updates = u16::try_from(tick).unwrap();
                parts.rules.remaining_ticks -= 1;
            })
            .unwrap();
        let (next, spent) = select(&observed, 28, memory, 65_536);
        assert_eq!(next.intent.target, first.intent.target);
        assert_eq!(next.intent.progress_deadline, deadline);
        assert_eq!(next.intent.committed_until, commitment);
        assert!(spent <= 65_536);
        memory = memory.with_strategic_intent(next.intent);
    }
}
