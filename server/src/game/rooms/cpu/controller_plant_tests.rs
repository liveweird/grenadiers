use super::{CpuController, Pending};
use crate::game::rooms::cpu::worker::{CpuPlanner, PlanResult};
use grenadiers_core::cpu::forecast::ObservedHazardMemory;
use grenadiers_core::cpu::policy::UtilityMemory;
use grenadiers_core::cpu::routing::RouteCertificate;
use grenadiers_core::cpu::survival::{ExecutionCertificate, SurvivalPlan, WORK_ALLOWANCE};
use grenadiers_core::cpu::{
    BombKnowledge, CpuRng, DecisionId, Intent, ManualIntent, MemoryKey, Observation, ObservedActor,
    ObservedBomb, ObservedBombTrigger, UnknownBombFacts, WorkBudget,
};
use grenadiers_core::game::{BlastKind, EntityId, SeatId, Tick};
use grenadiers_core::game::{EffectKind, Equipment, SpecialWeapon, SpeedLevel};
use grenadiers_core::room::CpuProfile;
use std::sync::{Arc, atomic::AtomicBool, mpsc::sync_channel};
#[test]
fn completed_plant_result_waits_for_exact_admission() {
    let basis = crate::game::rooms::cpu::tests::observation();
    let mut controller = prepared(&basis);
    inject_certified(&mut controller, &basis);
    assert!(controller.poll(Tick(20), 4).is_none());
    assert!(controller.has_pending());
    assert_eq!(controller.rng_draws(), 0);

    let exact = advanced(&basis, Tick(26));
    assert!(controller.accept_observation(&exact));
    let accepted = controller.poll(Tick(26), 4).expect("exact admission");
    assert!(accepted.has_route());
    assert_eq!(accepted.intent.manual, Some(ManualIntent::Bomb));
    assert!(!controller.has_pending());
}

#[test]
fn completed_plant_result_cancels_when_late_or_epoch_changes() {
    let basis = crate::game::rooms::cpu::tests::observation();
    for profile in [CpuProfile::CURRENT, CpuProfile::CURRENT_EASY] {
        let make = || {
            let mut controller = prepared(&basis);
            controller.profile = profile;
            inject_certified(&mut controller, &basis);
            controller
        };
        for (tick, epoch) in [(Tick(27), 4), (Tick(20), 5)] {
            let mut rejected = make();
            assert!(rejected.poll(tick, epoch).is_none());
            assert!(!rejected.has_pending());
            assert_eq!(rejected.rng_draws(), 0);
        }
    }
}

#[test]
fn fresh_freeze_rejects_certified_plant_at_exact_admission() {
    let basis = crate::game::rooms::cpu::tests::observation();
    let mut controller = prepared(&basis);
    inject_certified(&mut controller, &basis);
    assert!(controller.poll(Tick(20), 4).is_none());
    let mut frozen = advanced(&basis, Tick(26)).into_parts();
    let ObservedActor::Visible(cpu) = &mut frozen.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.voluntary_actions_frozen = true;
    let frozen = Observation::new(frozen).unwrap();
    assert!(controller.accept_observation(&frozen));

    assert!(controller.poll(Tick(26), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.rng_draws(), 0);
}

#[test]
fn actor_fact_mutations_reject_plant_admission_and_midroute() {
    let mutations = [
        ActorMutation::Identity,
        ActorMutation::Dead,
        ActorMutation::Speed,
        ActorMutation::Equipment,
        ActorMutation::Weapon,
        ActorMutation::Capacity,
        ActorMutation::Range,
        ActorMutation::LiveCount,
        ActorMutation::Teleport,
        ActorMutation::DamageGrace,
        ActorMutation::Effects,
    ];
    for mutation in mutations {
        let basis = crate::game::rooms::cpu::tests::observation();
        let mut admission = prepared(&basis);
        inject_certified(&mut admission, &basis);
        assert!(admission.poll(Tick(20), 4).is_none());
        let changed = mutated(advanced(&basis, Tick(26)), mutation);
        assert!(admission.accept_observation(&changed));
        assert!(
            admission.poll(Tick(26), 4).is_none(),
            "admission {mutation:?}"
        );

        let mut committed = prepared(&basis);
        let fixture = inject_certified(&mut committed, &basis);
        let exact = advanced(&basis, Tick(26));
        assert!(committed.accept_observation(&exact));
        let accepted = committed.poll(Tick(26), 4).unwrap();
        committed.on_movement_admitted();
        committed.commit(
            accepted,
            Intent {
                movement: fixture.movement,
                manual: None,
            },
        );
        let changed = mutated(placement_observation(&basis, &fixture, None), mutation);
        assert!(committed.accept_observation(&changed));
        assert_eq!(
            committed.observe_route(&changed),
            Some(grenadiers_core::cpu::survival::RouteStatus::Invalidated),
            "midroute {mutation:?}"
        );
    }
}

#[test]
fn effect_expiry_before_admission_is_accepted() {
    let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.effects.push(grenadiers_core::cpu::ObservedEffect {
        kind: EffectKind::SlowFoot,
        remaining_ticks: 3,
    });
    let basis = Observation::new(parts).unwrap();
    let mut controller = prepared(&basis);
    inject_certified(&mut controller, &basis);
    let exact = advanced(&basis, Tick(26));
    assert!(controller.accept_observation(&exact));
    assert!(controller.poll(Tick(26), 4).is_some());
}

#[test]
fn rejected_manual_selects_absent_and_keeps_the_committed_route() {
    let basis = crate::game::rooms::cpu::tests::observation();
    let mut controller = prepared(&basis);
    let fixture = inject_certified(&mut controller, &basis);
    let exact = advanced(&basis, Tick(26));
    assert!(controller.accept_observation(&exact));
    let accepted = controller.poll(Tick(26), 4).unwrap();
    controller.on_movement_admitted();
    controller.commit(
        accepted,
        Intent {
            movement: fixture.movement,
            manual: None,
        },
    );

    assert!(!controller.is_escaping());
    assert_eq!(
        controller
            .committed_route
            .as_ref()
            .and_then(|route| route.plant_presence()),
        Some(grenadiers_core::cpu::routing::PlantPresence::Absent)
    );
    let absent = placement_observation(&basis, &fixture, None);
    assert!(controller.accept_observation(&absent));
    assert!(controller.observe_route(&absent).is_some());
}

#[test]
fn accepted_manual_binds_the_recipient_visible_bomb_id() {
    let basis = crate::game::rooms::cpu::tests::observation();
    let mut controller = prepared(&basis);
    let fixture = inject_certified(&mut controller, &basis);
    let exact = advanced(&basis, Tick(26));
    assert!(controller.accept_observation(&exact));
    let accepted = controller.poll(Tick(26), 4).unwrap();
    controller.on_movement_admitted();
    controller.commit(
        accepted,
        Intent {
            movement: fixture.movement,
            manual: Some(ManualIntent::Bomb),
        },
    );

    let placed = placement_observation(&basis, &fixture, Some(EntityId(91)));
    assert!(controller.accept_observation(&placed));
    assert!(controller.observe_route(&placed).is_some());
    assert_eq!(
        controller
            .committed_route
            .as_ref()
            .and_then(|route| route.plant_presence()),
        Some(grenadiers_core::cpu::routing::PlantPresence::Present(
            EntityId(91)
        ))
    );
}

#[test]
fn accepted_manual_without_physical_placement_selects_absent() {
    let basis = crate::game::rooms::cpu::tests::observation();
    let mut controller = prepared(&basis);
    let fixture = inject_certified(&mut controller, &basis);
    let exact = advanced(&basis, Tick(26));
    assert!(controller.accept_observation(&exact));
    let accepted = controller.poll(Tick(26), 4).unwrap();
    controller.on_movement_admitted();
    controller.commit(
        accepted,
        Intent {
            movement: fixture.movement,
            manual: Some(ManualIntent::Bomb),
        },
    );

    let absent = placement_observation(&basis, &fixture, None);
    assert!(controller.accept_observation(&absent));
    assert!(controller.observe_route(&absent).is_some());
    assert_eq!(
        controller.plant_presence(),
        Some(grenadiers_core::cpu::routing::PlantPresence::Absent)
    );
    assert!(controller.has_committed_route());
}

pub(super) fn prepared(basis: &Observation) -> CpuController {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        grenadiers_core::game::SeatId::Two,
        7,
        [5; 32],
        CpuProfile::CURRENT,
    );
    controller.reset_world(3);
    assert!(controller.accept_observation(basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    controller
}

pub(super) struct PlantFixture {
    pub(super) placement: grenadiers_core::cpu::routing::PlannedBomb,
    pub(super) movement: grenadiers_core::cpu::MovementIntent,
    pub(super) position: grenadiers_core::game::Position,
    pub(super) heading: grenadiers_core::game::Direction,
    pub(super) route: RouteCertificate,
}

pub(super) fn inject_certified(
    controller: &mut CpuController,
    basis: &Observation,
) -> PlantFixture {
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(basis).unwrap();
    let mut rng = CpuRng::from_seed(key, [8; 32]);
    let plan = grenadiers_core::cpu::survival::plan(
        basis,
        &UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        &mut rng,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let Some(ExecutionCertificate::Plant(certificate)) = &plan.execution else {
        panic!("expected certified plant")
    };
    let (position, heading) = certificate
        .route()
        .lane(Tick(27))
        .and_then(|lane| lane.pose_at(Tick(27)))
        .unwrap();
    let fixture = PlantFixture {
        placement: *certificate.placement(),
        movement: plan.control.intent.movement,
        position,
        heading,
        route: certificate.route().clone(),
    };
    inject(controller, basis.clone(), plan, rng);
    fixture
}

fn inject(controller: &mut CpuController, basis: Observation, plan: SurvivalPlan, rng: CpuRng) {
    let decision = DecisionId {
        world: basis.parts().world,
        command_epoch: 4,
        observation_tick: basis.parts().tick,
        checked_request_serial: 1,
    };
    let (sender, receiver) = sync_channel(1);
    sender
        .send(PlanResult {
            plan,
            contact: None,
            basis,
            rng,
            escape_origin: None,
        })
        .unwrap();
    controller.pending = Some(Pending {
        decision,
        receiver,
        live: Arc::new(AtomicBool::new(true)),
        completed: None,
        refuge_handoff: None,
    });
    controller.request_serial = 1;
}

pub(super) fn advanced(basis: &Observation, tick: Tick) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    for effect in &mut cpu.effects {
        effect.remaining_ticks = effect.remaining_ticks.saturating_sub(elapsed);
    }
    cpu.effects.retain(|effect| effect.remaining_ticks > 0);
    Observation::new(parts).unwrap()
}

#[derive(Clone, Copy, Debug)]
enum ActorMutation {
    Identity,
    Dead,
    Speed,
    Equipment,
    Weapon,
    Capacity,
    Range,
    LiveCount,
    Teleport,
    DamageGrace,
    Effects,
}

fn mutated(observation: Observation, mutation: ActorMutation) -> Observation {
    let mut parts = observation.into_parts();
    let tick = parts.tick;
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    match mutation {
        ActorMutation::Identity => cpu.id = EntityId(3),
        ActorMutation::Dead => cpu.alive = false,
        ActorMutation::Speed => cpu.speed = SpeedLevel::One,
        ActorMutation::Equipment => cpu.equipment = Some(Equipment::Kick),
        ActorMutation::Weapon => cpu.weapon = Some(SpecialWeapon::Remote),
        ActorMutation::Capacity => cpu.bomb_capacity += 1,
        ActorMutation::Range => cpu.bomb_range += 1,
        ActorMutation::LiveCount => cpu.live_bomb_count += 1,
        ActorMutation::Teleport => cpu.last_teleport_tick = Some(tick),
        ActorMutation::DamageGrace => cpu.damage_grace_remaining_ticks = 1,
        ActorMutation::Effects => cpu.effects.push(grenadiers_core::cpu::ObservedEffect {
            kind: EffectKind::SlowFoot,
            remaining_ticks: 10,
        }),
    }
    Observation::new(parts).unwrap()
}

pub(super) fn placement_observation(
    basis: &Observation,
    fixture: &PlantFixture,
    id: Option<EntityId>,
) -> Observation {
    let mut parts = advanced(basis, fixture.placement.placed_at).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.position = fixture.position;
    cpu.heading = fixture.heading;
    if let Some(id) = id {
        cpu.live_bomb_count = 1;
        let remaining_ticks =
            u16::try_from(fixture.placement.detonates_at.0 - fixture.placement.placed_at.0)
                .unwrap();
        parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id,
            placed_at: fixture.placement.placed_at,
            cell: fixture.placement.cell,
            owner: fixture.placement.owner,
            range: fixture.placement.range,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks,
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]);
    }
    Observation::new(parts).unwrap()
}
