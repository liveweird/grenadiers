use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget};
use crate::cpu::{
    BombKnowledge, CpuRng, ManualIntent, MemoryKey, ObservedBombTrigger, ObservedFlame,
    ObservedPickup, WorkBudget,
};
use crate::game::{
    CellId, Direction, InputAction, OilBarrelState, PickupKind, SeatId, SeatInput, Tick,
};

use super::counter_pressure_v29_tests::{
    CPU, CPU_CELL, RIVAL_CELL, execute, observe_state, ordinary_counter_state, plan,
    power_counter_state, state,
};
use super::reliable_v27_oil_tests::{
    REMOTE_OIL, execute as execute_oil, observe as oil_observe, state as oil_state,
};
use super::{WORK_ALLOWANCE, plan_counter_pressure_v29_with_origin, strategic_plant_still_targets};

fn assert_preexecution_clear(observation: &crate::cpu::Observation) {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let forecast = crate::cpu::forecast::forecast(
        observation,
        &history,
        None,
        crate::cpu::forecast::ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let execution = Tick(observation.parts().tick.0 + 7);
    assert!(
        forecast
            .cell(CPU_CELL)
            .unwrap()
            .uncertainty
            .iter()
            .all(|uncertain| {
                uncertain.kind != crate::cpu::forecast::HazardKind::Blocking
                    || uncertain.window.start > execution
                    || uncertain.window.end <= observation.parts().tick
            })
    );
}

#[test]
fn separated_timed_bomb_allows_v29_attack_and_authoritative_execution() {
    for lairs in [false, true] {
        let (mut state, settings) = ordinary_counter_state(lairs);
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            }])
            .unwrap();
        for direction in [Direction::West, Direction::North] {
            for _ in 0..24 {
                state
                    .step(&[SeatInput {
                        seat: SeatId::One,
                        action: InputAction::Press(direction),
                    }])
                    .unwrap();
            }
        }
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            }])
            .unwrap();
        for _ in 0..10 {
            state.step(&[]).unwrap();
        }
        assert_eq!(
            state.actor(SeatId::One).unwrap().position().cell(),
            Some(RIVAL_CELL)
        );
        let basis = observe_state(&state, settings);
        assert!(basis.parts().arena.blocks.is_empty());
        assert!(
            matches!(&basis.parts().bombs, BombKnowledge::Visible(bombs) if bombs.len() == 1 && bombs[0].cell == CellId::new(9, 7))
        );
        assert_preexecution_clear(&basis);
        assert_eq!(plan(&basis, 28).0.survival.control.intent.manual, None);
        let (counter, spent) = plan(&basis, 29);
        assert_eq!(
            counter.survival.control.intent.manual,
            Some(ManualIntent::Bomb)
        );
        assert_eq!(counter.contact.is_some(), lairs);
        assert!(spent <= WORK_ALLOWANCE);
        let intent = counter
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap();
        assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
        assert_eq!(
            intent.target,
            StrategicTarget::Opponent { seat: SeatId::One }
        );
        assert!(strategic_plant_still_targets(
            &basis,
            intent,
            counter
                .survival
                .execution
                .as_ref()
                .unwrap()
                .plant()
                .unwrap(),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ));
        execute(&mut state, settings, basis, counter);
    }
}

#[test]
fn power_counter_pressure_models_and_executes_an_early_own_chain() {
    for lairs in [false, true] {
        let (mut state, settings) = power_counter_state(lairs);
        state.step(&[]).unwrap();
        for _ in 0..12 {
            state
                .step(&[SeatInput {
                    seat: SeatId::One,
                    action: InputAction::Press(Direction::North),
                }])
                .unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            }])
            .unwrap();
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            }])
            .unwrap();
        for _ in 0..12 {
            state
                .step(&[SeatInput {
                    seat: SeatId::One,
                    action: InputAction::Press(Direction::North),
                }])
                .unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            }])
            .unwrap();
        for _ in 0..20 {
            state.step(&[]).unwrap();
        }
        assert_eq!(
            state.actor(SeatId::One).unwrap().position().cell(),
            Some(RIVAL_CELL)
        );
        let basis = observe_state(&state, settings);
        let BombKnowledge::Visible(bombs) = &basis.parts().bombs else {
            unreachable!()
        };
        assert_eq!(bombs.len(), 1);
        assert_eq!(
            basis.parts().arena.terrain[basis.parts().arena.index(CellId::new(6, 6)).unwrap()],
            crate::game::Terrain::Wall
        );
        assert_eq!(
            basis.parts().arena.terrain[basis.parts().arena.index(CellId::new(8, 6)).unwrap()],
            crate::game::Terrain::Wall
        );
        assert_preexecution_clear(&basis);
        assert_eq!(
            (bombs[0].cell, bombs[0].range, bombs[0].blast),
            (CellId::new(7, 6), 3, crate::game::BlastKind::Power,)
        );
        let ObservedBombTrigger::Timed {
            remaining_ticks,
            paused: false,
        } = bombs[0].trigger
        else {
            panic!("power bomb must remain an unpaused timer")
        };
        assert_eq!(plan(&basis, 28).0.survival.control.intent.manual, None);
        let (counter, spent) = plan(&basis, 29);
        assert_eq!(
            counter.survival.control.intent.manual,
            Some(ManualIntent::Bomb)
        );
        assert_eq!(counter.contact.is_some(), lairs);
        assert!(spent <= WORK_ALLOWANCE);
        let certificate = counter
            .survival
            .execution
            .as_ref()
            .unwrap()
            .plant()
            .unwrap();
        let chain_tick = Tick(basis.parts().tick.0 + u64::from(remaining_ticks));
        assert!(chain_tick > Tick(certificate.placement().placed_at.0 + 2));
        assert!(chain_tick < certificate.placement().detonates_at);
        assert_eq!(
            certificate.expected_live_bombs(
                certificate.placement().owner,
                chain_tick,
                crate::cpu::routing::PlantPresence::Present(crate::game::EntityId(999)),
            ),
            Some(0)
        );
        execute(&mut state, settings, basis, counter);
    }
}

#[test]
fn counter_pressure_guard_rejects_unproven_bomb_and_reveal_states() {
    let (mut state, settings) = state(false, false);
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let basis = observe_state(&state, settings);
    let variants = [
        basis
            .clone()
            .map_parts(|parts| parts.arena.blocks.push(CellId::new(3, 3)))
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(3, 3),
                    remaining_ticks: 2,
                })
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                parts.pickups.push(ObservedPickup {
                    cell: CellId::new(3, 3),
                    kind: PickupKind::Teleporter,
                })
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| parts.bombs = BombKnowledge::Unavailable)
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                bombs[0].trigger = ObservedBombTrigger::Timed {
                    remaining_ticks: 179,
                    paused: true,
                };
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                bombs[0].trigger = ObservedBombTrigger::Remote;
            })
            .unwrap(),
        basis
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                bombs[0].trigger = ObservedBombTrigger::Contact;
            })
            .unwrap(),
    ];
    for variant in variants {
        assert_eq!(plan(&variant, 29).0.survival.control.intent.manual, None);
    }
}

#[test]
fn running_update_and_fresh_visible_teleporter_are_required_even_without_bombs() {
    let (mut state, settings) = state(false, true);
    let initial = observe_state(&state, settings);
    assert!(initial.parts().tick > Tick(0));
    assert_eq!(initial.parts().rules.running_updates, 0);
    assert!(initial.parts().pickups.is_empty());
    assert_eq!(plan(&initial, 29).0.survival.control.intent.manual, None);
    state.step(&[]).unwrap();
    let fresh = observe_state(&state, settings);
    assert!(
        fresh
            .parts()
            .pickups
            .iter()
            .any(|pickup| pickup.kind == PickupKind::Teleporter)
    );
    assert_eq!(plan(&fresh, 29).0.survival.control.intent.manual, None);
}

#[test]
fn empty_bomb_farming_after_an_update_retains_v27_and_v29_support() {
    let (mut state, settings) = oil_state(false, Some(REMOTE_OIL), true, false);
    state.step(&[]).unwrap();
    let observed = oil_observe(&state, settings);
    assert!(matches!(&observed.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(!observed.parts().arena.blocks.is_empty());
    assert!(
        observed
            .parts()
            .oil
            .iter()
            .any(|oil| oil.state == OilBarrelState::Burning)
    );
    let reliable = plan(&observed, 27).0;
    assert_eq!(
        reliable.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );

    let key = MemoryKey {
        world: observed.parts().world,
        cpu_seat: CPU,
    };
    let mut hazards = ObservedHazardMemory::new(key);
    hazards.observe(&observed).unwrap();
    let mut rng = CpuRng::from_seed(key, [29; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let counter = plan_counter_pressure_v29_with_origin(
        &observed,
        &reliable.survival.control.next_memory,
        &hazards,
        observed.parts().tick,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    assert_eq!(
        counter.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert_eq!(
        counter
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap()
            .objective,
        StrategicObjectiveKind::Farm
    );
    assert!(
        counter
            .survival
            .execution
            .as_ref()
            .unwrap()
            .plant()
            .is_some()
    );
    assert!(work.spent() <= WORK_ALLOWANCE);
    execute_oil(&mut state, settings, observed, counter);
}
