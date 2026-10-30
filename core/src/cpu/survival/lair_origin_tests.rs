use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::lair_contact::tests::mob;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    BombKnowledge, CpuRng, MovementIntent, ObservedActor, ObservedBomb, ObservedBombTrigger,
    UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId};

use super::lair_planner_tests::{key, planned, plant_scene};
use super::{ExecutionCertificate, PlantEscapeOrigin, plan_lairs, plan_lairs_with_origin};

fn interrupted_plant_origin() -> (crate::cpu::Observation, PlantEscapeOrigin) {
    let basis = plant_scene(None);
    let planned = planned(&basis);
    let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
        panic!("fixture must produce a certified plant")
    };
    let placement = *certificate.placement();
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(placement.placed_at.0 - parts.tick.0).unwrap();
    parts.tick = placement.placed_at;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let lane = certificate.route().lane(placement.placed_at).unwrap();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    (actor.position, actor.heading) = lane.pose_at(placement.placed_at).unwrap();
    actor.live_bomb_count = 1;
    let own_id = EntityId(700);
    parts.bombs = BombKnowledge::Visible(vec![
        ObservedBomb {
            id: own_id,
            placed_at: placement.placed_at,
            cell: placement.cell,
            owner: placement.owner,
            range: placement.range,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: u16::try_from(placement.detonates_at.0 - placement.placed_at.0)
                    .unwrap(),
                paused: false,
            },
            blast: placement.blast,
            unknown: UnknownBombFacts::ALL,
        },
        ObservedBomb {
            id: EntityId(701),
            placed_at: placement.placed_at,
            cell: CellId::new(13, 11),
            owner: EntityId(1),
            range: 2,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: u16::try_from(placement.detonates_at.0 - placement.placed_at.0)
                    .unwrap(),
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        },
    ]);
    let current = crate::cpu::Observation::new_rules_v8(
        parts,
        basis.environment().unwrap().clone(),
        basis.lairs().unwrap().clone(),
    )
    .unwrap();
    let origin = PlantEscapeOrigin::new(basis, *certificate, own_id, &current)
        .expect("exact own birth retains origin grace despite the rival bomb");
    (current, origin)
}

#[test]
fn interrupted_multiplayer_plant_uses_origin_grace_with_a_contact_lease() {
    let (observed, origin) = interrupted_plant_origin();
    let key = key(&observed);
    let memory = UtilityMemory::new(key);
    let history = ObservedHazardMemory::new(key);

    let without_origin = plan_lairs(
        &observed,
        &memory,
        &history,
        observed.parts().tick,
        &mut CpuRng::from_seed(key, [17; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        without_origin.survival.control.intent.movement,
        MovementIntent::Stop
    );

    let recovered = plan_lairs_with_origin(
        &observed,
        &memory,
        &history,
        observed.parts().tick,
        Some(&origin),
        &mut CpuRng::from_seed(key, [17; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_ne!(
        recovered.survival.control.intent.movement,
        MovementIntent::Stop
    );
    assert_eq!(recovered.survival.control.intent.manual, None);
    assert!(recovered.survival.lease.is_some());
    assert!(recovered.contact.is_some());
}

#[test]
fn origin_grace_does_not_bypass_lair_contact_or_work_limits() {
    let (observed, origin) = interrupted_plant_origin();
    let actor = super::compatibility::actor(&observed).unwrap();
    let mut lairs = observed.lairs().unwrap().lairs().to_vec();
    lairs[0].living_mobs = 1;
    let unsafe_overlay = crate::cpu::ObservedLairs::new(
        observed.lairs().unwrap().config(),
        lairs,
        vec![mob(10, actor.position)],
    );
    let unsafe_observation = crate::cpu::Observation::new_rules_v8(
        observed.clone().into_parts(),
        observed.environment().unwrap().clone(),
        unsafe_overlay,
    )
    .unwrap();
    let key = key(&unsafe_observation);
    for (observation, limit) in [(&unsafe_observation, 65_536), (&observed, 1)] {
        let plan = plan_lairs_with_origin(
            observation,
            &UtilityMemory::new(key),
            &ObservedHazardMemory::new(key),
            observation.parts().tick,
            Some(&origin),
            &mut CpuRng::from_seed(key, [19; 32]),
            &mut WorkBudget::new(limit),
        )
        .unwrap();
        assert_eq!(plan.survival.control.intent.movement, MovementIntent::Stop);
        assert!(plan.survival.lease.is_none());
        assert!(plan.contact.is_none());
    }
}
