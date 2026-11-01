use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant};
use crate::cpu::{MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, EntityId, InputAction, Position, SeatId, SeatInput, State, Tick};
use crate::room::RoomSettings;

use super::continuation_test_fixture::{CPU, observe, observe_for, state_with_lairs, step_until};
use super::{PlantEscapeOrigin, WORK_ALLOWANCE};

pub(super) struct ContinuationScene {
    pub(super) state: State,
    pub(super) settings: RoomSettings,
    pub(super) basis: Observation,
    pub(super) rival_basis: Observation,
    pub(super) origin: PlantEscapeOrigin,
    pub(super) rival_origin: PlantEscapeOrigin,
    pub(super) key: MemoryKey,
    pub(super) rival_key: MemoryKey,
}

pub(super) fn scene() -> ContinuationScene {
    scene_with_lairs(true)
}

pub(super) fn no_lair_scene() -> ContinuationScene {
    scene_with_lairs(false)
}

fn scene_with_lairs(enabled: bool) -> ContinuationScene {
    let (mut state, settings) = state_with_lairs(enabled);
    state.step(&[]).unwrap();
    step_until(&mut state, Tick(184));
    let plant_basis = observe(&state, settings);
    let key = MemoryKey {
        world: plant_basis.parts().world,
        cpu_seat: CPU,
    };
    let rival_plant_basis = observe_for(&state, settings, SeatId::One);
    let rival_key = MemoryKey {
        world: rival_plant_basis.parts().world,
        cpu_seat: SeatId::One,
    };
    let request = PlantRequest {
        execution_tick: Tick(191),
        settled_stop_tick: Tick(184),
        first_movement: MovementIntent::Direction(Direction::South),
        limits: RouteLimits::default(),
    };
    let plant = plan_lair_ordinary_plant(
        &plant_basis,
        &ObservedHazardMemory::new(key),
        request,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .expect("ordinary short-fuse plant proof");
    let rival_plant = plan_lair_ordinary_plant(
        &rival_plant_basis,
        &ObservedHazardMemory::new(rival_key),
        request,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .expect("mirrored ordinary short-fuse plant proof");
    assert_eq!(plant.placement().placed_at, Tick(191));
    assert_eq!(plant.placement().detonates_at, Tick(281));
    step_until(&mut state, Tick(190));
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: CPU,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    step_until(&mut state, Tick(196));
    assert_eq!(
        state.actor(CPU).unwrap().position(),
        Position { x: 1536, y: 9728 }
    );
    assert_eq!(
        state.actor(SeatId::One).unwrap().position(),
        Position { x: 13824, y: 9728 }
    );
    assert!(
        state.world().bombs().values().all(|bomb| {
            bomb.placed_at() == Tick(191) && bomb.detonates_at() == Some(Tick(281))
        })
    );
    let basis = observe(&state, settings);
    let rival_basis = observe_for(&state, settings, SeatId::One);
    let own = state
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.owner() == EntityId(2))
        .unwrap();
    let rival_bomb = state
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.owner() == EntityId(1))
        .unwrap();
    let origin = PlantEscapeOrigin::new(plant_basis, plant, own.id(), &basis)
        .expect("genuine stopped plant origin");
    let rival_origin = PlantEscapeOrigin::new(
        rival_plant_basis,
        rival_plant,
        rival_bomb.id(),
        &rival_basis,
    )
    .expect("genuine mirrored stopped plant origin");
    ContinuationScene {
        state,
        settings,
        basis,
        rival_basis,
        origin,
        rival_origin,
        key,
        rival_key,
    }
}
