use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::routing::{PlantCertificate, PlantPresence, PlantRequest, RouteLimits};
use crate::cpu::survival::{
    CommittedRoute, ExecutionCertificate, RouteStatus, WORK_ALLOWANCE,
    contact_scene_candidate_for_test, plan_contact_topology_v40_with_origin,
};
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, MovementIntent, Observation, ObservedBomb,
    ObservedBombTrigger, ObservedRespawnLairs, UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, Direction, EntityId, PickupKind, Tick};
use crate::room::RespawnLairSettings;

use super::PlantContactProof;
use super::physical_fixture::{CPU, controlled_state, input, memory};

fn fixture() -> (Observation, PlantCertificate) {
    let (mut state, settings) = controlled_state();
    for _ in 0..RespawnLairSettings::DEFAULT.spawn_ticks() {
        state.step(&[]).unwrap();
    }
    let observation =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let certificate = crate::cpu::routing::plan_lair_ordinary_plant(
        &observation,
        &history,
        PlantRequest {
            execution_tick: Tick(607),
            settled_stop_tick: Tick(600),
            first_movement: crate::cpu::MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        &mut work,
    )
    .unwrap();
    (observation, certificate)
}

fn rebuilt(
    source: &Observation,
    mutate: impl FnOnce(&mut crate::cpu::ObservationParts),
) -> Observation {
    let mut parts = source.parts().clone();
    mutate(&mut parts);
    Observation::new_rules_v10(
        parts,
        source.environment().unwrap().clone(),
        source.respawn_lairs().cloned(),
        source.spike_pits().cloned(),
    )
    .unwrap()
}

#[test]
fn topology_construction_and_renewal_fail_closed_one_unit_short() {
    let (basis, plant) = fixture();
    let mut full = WorkBudget::new(WORK_ALLOWANCE);
    let proof = PlantContactProof::certify(&basis, &plant, &mut full).unwrap();
    let construction = full.spent();
    assert_eq!(construction, 1_975);
    assert!(
        PlantContactProof::certify(&basis, &plant, &mut WorkBudget::new(construction)).is_some()
    );
    assert!(
        PlantContactProof::certify(&basis, &plant, &mut WorkBudget::new(construction - 1))
            .is_none()
    );

    let (candidate, renewal) = contact_scene_candidate_for_test(
        &basis,
        &basis,
        &plant,
        &proof,
        PlantPresence::Awaiting,
        8_192,
    );
    assert_eq!(candidate, Some(PlantPresence::Awaiting));
    assert_eq!(renewal, 526);
    assert_eq!(
        contact_scene_candidate_for_test(
            &basis,
            &basis,
            &plant,
            &proof,
            PlantPresence::Awaiting,
            renewal,
        )
        .0,
        Some(PlantPresence::Awaiting)
    );
    assert_eq!(
        contact_scene_candidate_for_test(
            &basis,
            &basis,
            &plant,
            &proof,
            PlantPresence::Awaiting,
            renewal - 1,
        )
        .0,
        None
    );
}

#[test]
fn topology_construction_rejects_unsupported_initial_public_facts() {
    let (basis, plant) = fixture();
    let unavailable = rebuilt(&basis, |parts| parts.bombs = BombKnowledge::Unavailable);
    assert!(
        PlantContactProof::certify(&unavailable, &plant, &mut WorkBudget::new(WORK_ALLOWANCE))
            .is_none()
    );

    let teleporter = rebuilt(&basis, |parts| {
        parts.pickups.push(crate::cpu::ObservedPickup {
            cell: crate::game::CellId::new(11, 1),
            kind: PickupKind::Teleporter,
        });
    });
    assert!(
        PlantContactProof::certify(&teleporter, &plant, &mut WorkBudget::new(WORK_ALLOWANCE))
            .is_none()
    );

    let no_update = rebuilt(&basis, |parts| parts.rules.running_updates = 0);
    assert!(
        PlantContactProof::certify(&no_update, &plant, &mut WorkBudget::new(WORK_ALLOWANCE))
            .is_none()
    );
}

#[test]
fn renewal_rejects_lost_barriers_and_unexpected_bombs() {
    let (basis, plant) = fixture();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let proof = PlantContactProof::certify(&basis, &plant, &mut work).unwrap();

    let opened_separator = rebuilt(&basis, |parts| {
        parts.arena.blocks.retain(|cell| *cell != CellId::new(7, 5));
    });
    assert!(!proof.observation_fits(&opened_separator, &mut WorkBudget::new(8_192)));

    let foreign_bomb = rebuilt(&basis, |parts| {
        parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(99),
            placed_at: Tick(590),
            cell: CellId::new(11, 1),
            owner: EntityId(1),
            range: 2,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: 170,
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]);
    });
    assert_eq!(
        contact_scene_candidate_for_test(
            &basis,
            &foreign_bomb,
            &plant,
            &proof,
            PlantPresence::Awaiting,
            8_192,
        )
        .0,
        None
    );
}

#[test]
fn established_presence_and_bound_source_never_downgrade_on_mutation() {
    let (mut state, settings) = controlled_state();
    for _ in 0..RespawnLairSettings::DEFAULT.spawn_ticks() {
        state.step(&[]).unwrap();
    }
    let basis =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_contact_topology_v40_with_origin(
        &basis,
        &memory(&basis),
        &history,
        basis.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [40; 32]),
        &mut work,
    )
    .unwrap();
    let execution = planned.survival.execution.unwrap();
    assert!(matches!(execution, ExecutionCertificate::ContactPlant(_)));
    while state.tick() < Tick(606) {
        state.step(&[]).unwrap();
    }
    let admission =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    let mut committed = CommittedRoute::admit_execution(
        execution,
        basis,
        &admission,
        Tick(606),
        MovementIntent::Stop,
    )
    .unwrap();
    state.step(&input(None, true)).unwrap();
    let present =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    assert_ne!(
        committed.observe_with_stop(&present, Some(Tick(607))),
        RouteStatus::Invalidated
    );
    let PlantPresence::Present(id) = committed.plant_presence_for_test().unwrap() else {
        panic!("physical placement must establish one actual bomb identity");
    };
    state.step(&input(None, false)).unwrap();
    let current =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    for changed in [
        current
            .clone()
            .map_parts(|parts| parts.bombs = BombKnowledge::Visible(Vec::new()))
            .unwrap(),
        current
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                bombs[0].cell = CellId::new(11, 7);
            })
            .unwrap(),
        current
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    unreachable!()
                };
                bombs[0].id = EntityId(100);
            })
            .unwrap(),
    ] {
        let mut route = committed.clone();
        assert_eq!(route.observe(&changed), RouteStatus::Invalidated);
        assert_eq!(
            route.plant_presence_for_test(),
            Some(PlantPresence::Present(id))
        );
    }

    let original = current.respawn_lairs().unwrap();
    let mut mobs = original.mobs().to_vec();
    mobs[0].position = crate::game::Position::center(CellId::new(10, 6));
    let moved_source = Observation::new_rules_v10(
        current.parts().clone(),
        current.environment().unwrap().clone(),
        Some(ObservedRespawnLairs::new(
            original.config(),
            original.lairs().to_vec(),
            mobs,
        )),
        current.spike_pits().cloned(),
    )
    .unwrap();
    let mut route = committed.clone();
    assert_eq!(route.observe(&moved_source), RouteStatus::Invalidated);
    assert_eq!(
        route.plant_presence_for_test(),
        Some(PlantPresence::Present(id))
    );
}
