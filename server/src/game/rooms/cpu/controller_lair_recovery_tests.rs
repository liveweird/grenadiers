use std::sync::{Arc, atomic::AtomicBool, mpsc::sync_channel};

use grenadiers_core::cpu::forecast::ObservedHazardMemory;
use grenadiers_core::cpu::policy::UtilityMemory;
use grenadiers_core::cpu::survival::{ExecutionCertificate, WORK_ALLOWANCE, plan_lairs};
use grenadiers_core::cpu::{
    BombKnowledge, CpuRng, DecisionId, Intent, MemoryKey, MovementIntent, Observation,
    ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedEnvironment, ObservedLairs,
    UnknownBombFacts, WorkBudget,
};
use grenadiers_core::game::{
    BlastKind, CellId, Direction, EntityId, LairView, MobView, Position, SeatId, StormCloudConfig,
    Tick, lair_layout_v1,
};
use grenadiers_core::room::{BoardSize, CpuProfile, LairConfig, LairSettings};

use super::contact::ContactStatus;
use super::lair_recovery_test_support::{visible_cpu, visible_cpu_mut};
use super::{CpuController, Pending};
use crate::game::rooms::cpu::worker::{CpuPlanner, PlanResult};

const EPOCH: u64 = 4;
const OWN_BOMB: EntityId = EntityId(700);
#[test]
fn lair_contact_invalidation_retains_birth_origin_and_allows_repeated_recovery() {
    let (mut controller, stopped) = stopped_origin_controller();
    assert!(controller.submit(stopped.clone(), EPOCH));
    assert_eq!(controller.planner.pump_for_test(1).delivered, 1);
    let accepted = controller
        .poll(stopped.parts().tick, EPOCH)
        .expect("origin-aware lair recovery");
    assert_ne!(accepted.intent.movement, MovementIntent::Stop);
    assert!(accepted.intent.manual.is_none());
    assert!(accepted.committed_pulse.is_some());
    assert!(accepted.contact.is_some());
    let lease = accepted.committed_pulse.as_ref().unwrap().lease().clone();
    let intent = accepted.intent;
    controller.on_movement_admitted();
    controller.commit(accepted, intent);
    assert_eq!(controller.recovery_state(), (true, true));

    let tick = Tick(stopped.parts().tick.0 + 1);
    let pose = lease.pose_after(1);
    let changed = advance_scene(
        &stopped,
        tick,
        pose.min(),
        match lease.movement() {
            MovementIntent::Direction(direction) => direction,
            MovementIntent::Stop => Direction::East,
        },
        901,
    );
    assert!(controller.accept_observation(&changed));
    assert_eq!(
        controller.validate_contact(&changed, tick),
        Some(ContactStatus::Invalidated)
    );
    assert_eq!(controller.recovery_state(), (true, false));

    controller.on_stop_admitted(tick);
    let stopped_again = advance_scene(
        &changed,
        Tick(tick.0 + 1),
        pose.min(),
        match lease.movement() {
            MovementIntent::Direction(direction) => direction,
            MovementIntent::Stop => Direction::East,
        },
        901,
    );
    assert!(controller.accept_observation(&stopped_again));
    assert!(controller.submit(stopped_again.clone(), EPOCH));
    assert_eq!(controller.planner.pump_for_test(1).delivered, 1);
    let recovered = controller
        .poll(stopped_again.parts().tick, EPOCH)
        .expect("fresh recovery after contact invalidation");
    assert_ne!(recovered.intent.movement, MovementIntent::Stop);
    assert!(recovered.committed_pulse.is_some());
    assert!(recovered.contact.is_some());
}
#[test]
fn lair_recovery_keeps_pose_and_epoch_fences() {
    let (mut displaced, stopped) = stopped_origin_controller();
    let actor = visible_cpu(&stopped);
    let wrong_pose = advance_scene(
        &stopped,
        Tick(stopped.parts().tick.0 + 1),
        Position {
            x: actor.position.x + 1,
            ..actor.position
        },
        actor.heading,
        900,
    );
    assert!(displaced.accept_observation(&wrong_pose));
    assert_eq!(displaced.recovery_state(), (false, false));

    let (mut wrong_epoch, _) = stopped_origin_controller();
    wrong_epoch.validate_escape_epoch(EPOCH + 1);
    assert_eq!(wrong_epoch.recovery_state(), (false, false));
}

fn stopped_origin_controller() -> (CpuController, Observation) {
    let basis = lair_plant_basis();
    let mut controller = prepared(&basis);
    let (plan, contact, rng) = certified_plant(&basis);
    let Some(ExecutionCertificate::Plant(certificate)) = plan.execution.as_ref() else {
        panic!("rules-v8 fixture must certify planting")
    };
    let placement = *certificate.placement();
    let movement = plan.control.intent.movement;
    let route = certificate.route().clone();
    inject(&mut controller, basis.clone(), plan, contact, rng);

    let admission = advance_scene(
        &basis,
        Tick(placement.placed_at.0 - 1),
        visible_cpu(&basis).position,
        visible_cpu(&basis).heading,
        0,
    );
    assert!(controller.accept_observation(&admission));
    let accepted = controller.poll(admission.parts().tick, EPOCH).unwrap();
    controller.on_movement_admitted();
    controller.commit(
        accepted,
        Intent {
            movement,
            manual: Some(grenadiers_core::cpu::ManualIntent::Bomb),
        },
    );

    let (position, heading) = route
        .lane(placement.placed_at)
        .and_then(|lane| lane.pose_at(placement.placed_at))
        .unwrap();
    let birth = planted_scene(&basis, placement, position, heading, 900);
    assert!(controller.accept_observation(&birth));
    assert_eq!(
        controller.validate_contact(&birth, birth.parts().tick),
        Some(ContactStatus::Invalidated)
    );
    assert_eq!(controller.recovery_state(), (true, false));

    controller.on_stop_admitted(birth.parts().tick);
    let stopped = advance_scene(
        &birth,
        Tick(birth.parts().tick.0 + 1),
        position,
        heading,
        900,
    );
    assert!(controller.accept_observation(&stopped));
    assert_eq!(controller.recovery_state(), (true, false));
    (controller, stopped)
}

fn prepared(basis: &Observation) -> CpuController {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [5; 32],
        CpuProfile::LAIRS_STANDARD,
    );
    controller.reset_world(3);
    assert!(controller.accept_observation(basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    controller
}

fn certified_plant(
    basis: &Observation,
) -> (
    grenadiers_core::cpu::survival::SurvivalPlan,
    Option<grenadiers_core::cpu::MobContactLease>,
    CpuRng,
) {
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(basis).unwrap();
    let mut rng = CpuRng::from_seed(key, [8; 32]);
    let plan = plan_lairs(
        basis,
        &UtilityMemory::new(key),
        &history,
        basis.parts().tick,
        &mut rng,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(matches!(
        plan.survival.execution,
        Some(ExecutionCertificate::Plant(_))
    ));
    (plan.survival, plan.contact, rng)
}

fn inject(
    controller: &mut CpuController,
    basis: Observation,
    plan: grenadiers_core::cpu::survival::SurvivalPlan,
    contact: Option<grenadiers_core::cpu::MobContactLease>,
    rng: CpuRng,
) {
    let decision = DecisionId {
        world: basis.parts().world,
        command_epoch: EPOCH,
        observation_tick: basis.parts().tick,
        checked_request_serial: 1,
    };
    let (sender, receiver) = sync_channel(1);
    sender
        .send(PlanResult {
            outcome: super::super::worker::WorkerPlan::Ordinary {
                plan: Box::new(plan),
                contact,
            },
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
        continuation: None,
    });
    controller.request_serial = 1;
}

fn lair_plant_basis() -> Observation {
    let base = crate::game::rooms::cpu::tests::observation();
    let mut parts = base.into_parts();
    parts.rules.version = 8;
    parts.arena.blocks = vec![CellId::new(4, 3)];
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.position = Position::center(CellId::new(3, 3));
    cpu.heading = Direction::East;
    let cells = lair_layout_v1(BoardSize::FifteenByThirteen);
    for cell in cells {
        parts.arena.terrain
            [usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column)] =
            grenadiers_core::game::Terrain::Wall;
    }
    new_lair_observation(parts, Vec::new())
}

fn planted_scene(
    basis: &Observation,
    placement: grenadiers_core::cpu::routing::PlannedBomb,
    position: Position,
    heading: Direction,
    mob_id: u32,
) -> Observation {
    let mut parts = basis.clone().into_parts();
    advance_parts(&mut parts, placement.placed_at, position, heading);
    visible_cpu_mut(&mut parts).live_bomb_count = 1;
    parts.bombs = BombKnowledge::Visible(vec![
        observed_bomb(
            OWN_BOMB,
            placement.owner,
            placement.cell,
            placement,
            placement.range,
        ),
        observed_bomb(
            EntityId(701),
            EntityId(1),
            CellId::new(13, 11),
            placement,
            2,
        ),
    ]);
    new_lair_observation(parts, vec![observed_mob(mob_id)])
}

fn advance_scene(
    basis: &Observation,
    tick: Tick,
    position: Position,
    heading: Direction,
    mob_id: u32,
) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
    advance_parts(&mut parts, tick, position, heading);
    if let BombKnowledge::Visible(bombs) = &mut parts.bombs {
        for bomb in bombs {
            if let ObservedBombTrigger::Timed {
                remaining_ticks, ..
            } = &mut bomb.trigger
            {
                *remaining_ticks -= elapsed;
            }
        }
    }
    let mobs = (mob_id != 0)
        .then(|| observed_mob(mob_id))
        .into_iter()
        .collect();
    new_lair_observation(parts, mobs)
}

fn advance_parts(
    parts: &mut grenadiers_core::cpu::ObservationParts,
    tick: Tick,
    position: Position,
    heading: Direction,
) {
    let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let cpu = visible_cpu_mut(parts);
    cpu.position = position;
    cpu.heading = heading;
}

fn new_lair_observation(
    parts: grenadiers_core::cpu::ObservationParts,
    mobs: Vec<MobView>,
) -> Observation {
    let cells = lair_layout_v1(BoardSize::FifteenByThirteen);
    let lairs = cells
        .into_iter()
        .enumerate()
        .map(|(index, cell)| LairView {
            cell,
            next_spawn_at: Tick(300),
            living_mobs: mobs
                .iter()
                .filter(|mob| usize::from(mob.lair) == index)
                .count() as u8,
        })
        .collect();
    Observation::new_rules_v8(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![None, Some(0)]),
        ObservedLairs::new(
            LairConfig::Enabled(LairSettings::new(5, 3).unwrap()),
            lairs,
            mobs,
        ),
    )
    .unwrap()
}

fn observed_bomb(
    id: EntityId,
    owner: EntityId,
    cell: CellId,
    placement: grenadiers_core::cpu::routing::PlannedBomb,
    range: u16,
) -> ObservedBomb {
    ObservedBomb {
        id,
        placed_at: placement.placed_at,
        cell,
        owner,
        range,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: u16::try_from(placement.detonates_at.0 - placement.placed_at.0)
                .unwrap(),
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }
}

fn observed_mob(id: u32) -> MobView {
    MobView {
        id: EntityId(id),
        lair: 0,
        position: Position::center(CellId::new(1, 11)),
        heading: Direction::East,
        frozen: false,
    }
}
