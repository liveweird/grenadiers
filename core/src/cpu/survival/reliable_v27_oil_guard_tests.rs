use crate::cpu::{
    BombKnowledge, CpuRng, ManualIntent, MemoryKey, ObservedBomb, ObservedBombTrigger,
    ObservedFlame, ObservedOil, ObservedPickup, UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, OilBarrelState, PickupKind, Tick};

use super::reliable_v27_oil_tests::{REMOTE_OIL, SAFE_EXIT, execute, observe, plan, state};
use super::{CommittedRoute, RouteStatus, WORK_ALLOWANCE, plan_reliable_v27_with_origin};

#[test]
fn v27_entry_is_closed_to_rules_before_tactic_selection() {
    let observed = super::rules_v9_occupancy_tests::observed(CellId::new(7, 7), None);
    let key = MemoryKey {
        world: observed.parts().world,
        cpu_seat: observed.parts().recipient,
    };
    assert!(matches!(
        plan_reliable_v27_with_origin(
            &observed,
            &crate::cpu::policy::UtilityMemory::new(key),
            &crate::cpu::forecast::ObservedHazardMemory::new(key),
            observed.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [27; 32]),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ),
        Err(crate::cpu::policy::PolicyError::InvalidObservation)
    ));
}

#[test]
fn concealed_teleporter_is_revealed_only_to_the_fresh_endpoint_observation() {
    let (mut state, room_settings) = state(false, Some(REMOTE_OIL), true, true);
    let basis = observe(&state, room_settings);
    assert!(basis.parts().pickups.is_empty());
    let reliable = plan(&basis, true).0;
    assert_eq!(
        reliable.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );

    execute(&mut state, room_settings, basis, reliable);
    assert!(state.actor(crate::game::SeatId::Two).unwrap().alive());
    assert!(state.public_view().pickups.iter().any(|pickup| {
        pickup.cell == CellId::new(8, 7) && pickup.kind == PickupKind::Teleporter
    }));
}

#[test]
fn v27_retains_oil_escape_and_teleporter_uncertainty_controls() {
    let (blocked, settings) = state(false, Some(SAFE_EXIT), true, false);
    let blocked = plan(&observe(&blocked, settings), true).0;
    assert_eq!(blocked.survival.control.intent.manual, None);

    let (without_oil, settings) = state(false, None, true, false);
    assert_eq!(
        plan(&observe(&without_oil, settings), false)
            .0
            .survival
            .control
            .intent
            .manual,
        Some(ManualIntent::Bomb)
    );
    let (teleporter_disabled, settings) = state(false, Some(REMOTE_OIL), false, false);
    assert_eq!(
        plan(&observe(&teleporter_disabled, settings), false)
            .0
            .survival
            .control
            .intent
            .manual,
        Some(ManualIntent::Bomb)
    );
}

#[test]
fn v27_still_refuses_bombs_flames_visible_teleporters_and_unknown_bombs() {
    let (state, room_settings) = state(false, Some(REMOTE_OIL), true, false);
    let basis = observe(&state, room_settings);
    let variants = [
        basis
            .clone()
            .map_parts(|parts| {
                parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                    id: EntityId(90),
                    placed_at: parts.tick,
                    cell: CellId::new(3, 3),
                    owner: EntityId(1),
                    range: 1,
                    trigger: ObservedBombTrigger::Remote,
                    blast: BlastKind::Standard,
                    unknown: UnknownBombFacts::ALL,
                }]);
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(3, 3),
                    remaining_ticks: 10,
                });
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                parts.pickups.push(ObservedPickup {
                    cell: CellId::new(3, 3),
                    kind: PickupKind::Teleporter,
                });
            })
            .unwrap(),
        basis
            .map_parts(|parts| parts.bombs = BombKnowledge::Unavailable)
            .unwrap(),
    ];
    for variant in variants {
        assert_eq!(plan(&variant, true).0.survival.control.intent.manual, None);
    }
}

#[test]
fn admitted_v27_route_rejects_new_oil_bombs_and_pickups() {
    let (state, room_settings) = state(false, Some(REMOTE_OIL), true, false);
    let basis = observe(&state, room_settings);
    let reliable = plan(&basis, true).0;
    let movement = reliable.survival.control.intent.movement;
    let execution = reliable.survival.execution.unwrap();
    let admitted = basis
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(6);
            parts.rules.running_updates += 6;
            parts.rules.remaining_ticks -= 6;
        })
        .unwrap();
    let variants = [
        basis
            .clone()
            .map_parts(|parts| {
                advance(parts);
                parts.oil.push(ObservedOil {
                    cell: CellId::new(3, 3),
                    state: OilBarrelState::Burning,
                });
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                advance(parts);
                parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                    id: EntityId(90),
                    placed_at: Tick(7),
                    cell: CellId::new(3, 3),
                    owner: EntityId(1),
                    range: 1,
                    trigger: ObservedBombTrigger::Remote,
                    blast: BlastKind::Standard,
                    unknown: UnknownBombFacts::ALL,
                }]);
            })
            .unwrap(),
        basis
            .clone()
            .map_parts(|parts| {
                advance(parts);
                parts.pickups.push(ObservedPickup {
                    cell: CellId::new(3, 3),
                    kind: PickupKind::MovementSpeed,
                });
            })
            .unwrap(),
    ];
    for current in variants {
        let mut route = CommittedRoute::admit_execution(
            execution.clone(),
            basis.clone(),
            &admitted,
            Tick(6),
            movement,
        )
        .unwrap();
        assert_eq!(route.observe(&current), RouteStatus::Invalidated);
    }
}

fn advance(parts: &mut crate::cpu::ObservationParts) {
    parts.tick = Tick(7);
    parts.rules.running_updates += 7;
    parts.rules.remaining_ticks -= 7;
}
