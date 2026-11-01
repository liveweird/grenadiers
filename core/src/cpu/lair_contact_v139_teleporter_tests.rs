use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant};
use crate::cpu::{MemoryKey, MovementIntent, WorkBudget};
use crate::game::{CellId, Direction, PickupKind, Tick};

use super::PlantContactProof;
use super::v139_teleporter_fixture::{CPU, captured_tick_6121};

const ALLOWANCE: u32 = 65_536;
// The public sample contains the pose at tick 6_121, but not the controller's
// private settled-stop clock or subsequent inputs. Certification is conditional
// on these premises; neither timestamp is claimed as a captured private fact.
const ASSUMED_SETTLED_STOP_TICK: Tick = Tick(6_121);
const ASSUMED_EXECUTION_TICK: Tick = Tick(6_128);

#[test]
fn captured_public_teleporter_scene_is_reconstructed_without_private_facts() {
    let observation = captured_tick_6121();
    let parts = observation.parts();
    assert_eq!(parts.recipient, CPU);
    assert!(matches!(&parts.bombs, crate::cpu::BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(parts.flames.is_empty());
    assert!(parts.oil.is_empty());
    assert_eq!(
        parts
            .pickups
            .iter()
            .filter(|pickup| pickup.kind == PickupKind::Teleporter)
            .map(|pickup| pickup.cell)
            .collect::<Vec<_>>(),
        vec![CellId::new(3, 9)]
    );
}

#[test]
fn existing_plant_guard_rejects_captured_exposed_teleporter() {
    let observation = captured_tick_6121();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    let mut route_work = WorkBudget::new(ALLOWANCE);
    let plant = plan_lair_ordinary_plant(
        &observation,
        &history,
        PlantRequest {
            execution_tick: ASSUMED_EXECUTION_TICK,
            settled_stop_tick: ASSUMED_SETTLED_STOP_TICK,
            first_movement: MovementIntent::Direction(Direction::West),
            limits: RouteLimits::default(),
        },
        &mut route_work,
    )
    .expect("the captured pose should have an ordinary plant candidate");

    let before = route_work.spent();
    let mut proof_work = WorkBudget::new(ALLOWANCE);
    let rejected = PlantContactProof::certify(&observation, &plant, &mut proof_work);
    assert!(rejected.is_none());
    assert_eq!(proof_work.spent(), 8);

    // Counterfactual control: remove only the public Teleporter. This does not
    // claim that the captured match followed the resulting ordinary route.
    let without_teleporter = captured_tick_6121()
        .map_parts(|parts| {
            parts
                .pickups
                .retain(|pickup| pickup.kind != PickupKind::Teleporter);
        })
        .unwrap();
    let mut control_work = WorkBudget::new(ALLOWANCE);
    let control = PlantContactProof::certify(&without_teleporter, &plant, &mut control_work);
    assert!(control.is_some());
    assert!(control_work.spent() > proof_work.spent());
    assert!(before <= ALLOWANCE);
}
