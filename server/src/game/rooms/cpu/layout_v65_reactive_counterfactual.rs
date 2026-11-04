use grenadiers_core::{
    cpu::{
        BombKnowledge, CpuRng, MovementIntent, ObservedBomb, ObservedBombTrigger, ObservedPickup,
        UnknownBombFacts, WorkBudget,
        survival::{
            CommittedRoute, RouteStatus, WORK_ALLOWANCE, plan_hard_reactive_v65_with_origin,
        },
    },
    game::{BlastKind, CellId, EntityId, Equipment, PickupKind, Position, SeatId, Terrain, Tick},
};

use super::super::trace::witness::PlanSnapshot;

pub(super) fn assert_unsafe_pose_has_directional_escape(recovery: &PlanSnapshot) {
    let counterfactual = recovery
        .observation
        .clone()
        .map_parts(|parts| {
            for actor in &mut parts.actors {
                if let grenadiers_core::cpu::ObservedActor::Visible(actor) = actor
                    && actor.seat == SeatId::Two
                {
                    actor.position = Position {
                        x: 11_478,
                        y: 1_536,
                    };
                }
            }
        })
        .unwrap();
    let mut rng = CpuRng::from_seed(recovery.rng_key, [65; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_reactive_v65_with_origin(
        &counterfactual,
        &recovery.memory,
        &recovery.hazard_memory,
        recovery.settled_stop_tick,
        None,
        &mut rng,
        &mut work,
    )
    .expect("the neutral unsafe-pose counterfactual must certify an own-bomb escape");
    let plan = planned.survival;
    let execution = plan.execution.unwrap();
    assert!(work.spent() <= WORK_ALLOWANCE);
    assert!(execution.requires_reactive_support());
    assert!(execution.plant().is_none());
    assert_eq!(execution.manual_intent(), None);
    assert_eq!(execution.manual_execution_tick(), None);
    assert_ne!(plan.control.intent.movement, MovementIntent::Stop);
    let route = execution.route();
    assert_eq!(route.start_window().start, Tick(445));
    assert_eq!(route.start_window().end, Tick(452));
    assert_eq!(route.survive_until(), Tick(503));
    assert!((445..452).all(|tick| route.lane(Tick(tick)).is_some()));
    assert!(has_foreign_bomb_after_horizon(&counterfactual, Tick(503)));
    assert!(
        CommittedRoute::admit_execution(
            execution.clone(),
            counterfactual.clone(),
            &counterfactual,
            Tick(444),
            plan.control.intent.movement,
        )
        .is_err()
    );
    let mut committed = CommittedRoute::admit_reactive_execution(
        execution,
        counterfactual.clone(),
        &counterfactual,
        Tick(444),
        plan.control.intent.movement,
    )
    .expect("Hard65 explicit admission must retain the directional counterfactual route");
    assert_publication_mutations_revoke(&committed, &counterfactual);
    let mut committed_transition = false;
    for tick in 445..472 {
        if let RouteStatus::Submit(command) =
            committed.poll_reactive_command(Tick(tick), &counterfactual, None)
        {
            assert!(committed.commit_reactive_command(command, &counterfactual));
            committed_transition = true;
        }
    }
    assert!(committed_transition);
    eprintln!(
        "HARD65_UNSAFE_COUNTERFACTUAL work={} admission=444 movement={:?} surviveUntil=503",
        work.spent(),
        plan.control.intent.movement
    );
}

fn has_foreign_bomb_after_horizon(
    observation: &grenadiers_core::cpu::Observation,
    horizon: Tick,
) -> bool {
    let actor = observation.parts().actors[SeatId::Two.index()].id();
    matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.iter().any(|bomb| {
        bomb.owner != actor
            && matches!(bomb.trigger, ObservedBombTrigger::Timed { remaining_ticks, .. }
                if Tick(observation.parts().tick.0 + u64::from(remaining_ticks)) > horizon)
    }))
}

fn assert_publication_mutations_revoke(
    committed: &CommittedRoute,
    observation: &grenadiers_core::cpu::Observation,
) {
    let mutations = [
        with_new_public_bomb(observation),
        observation
            .clone()
            .map_parts(|parts| {
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    return;
                };
                bombs
                    .iter_mut()
                    .find(|bomb| bomb.id == EntityId(7))
                    .unwrap()
                    .cell = CellId::new(5, 7);
            })
            .unwrap(),
        observation
            .clone()
            .map_parts(|parts| {
                let owner = parts.actors[SeatId::Two.index()].id();
                let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                    return;
                };
                let bomb = bombs.iter_mut().find(|bomb| bomb.owner == owner).unwrap();
                let ObservedBombTrigger::Timed { paused, .. } = &mut bomb.trigger else {
                    panic!("the retained own bomb must be timed")
                };
                *paused = true;
            })
            .unwrap(),
        observation
            .clone()
            .map_parts(|parts| {
                parts.pickups.push(ObservedPickup {
                    cell: CellId::new(5, 7),
                    kind: PickupKind::BombCapacity,
                });
                parts.pickups.sort_unstable_by_key(|pickup| pickup.cell);
            })
            .unwrap(),
        observation
            .clone()
            .map_parts(|parts| {
                let grenadiers_core::cpu::ObservedActor::Visible(actor) =
                    &mut parts.actors[SeatId::Two.index()]
                else {
                    return;
                };
                actor.equipment = Some(Equipment::Kick);
            })
            .unwrap(),
        observation
            .clone()
            .map_parts(|parts| {
                let grenadiers_core::cpu::ObservedActor::Visible(actor) =
                    &mut parts.actors[SeatId::Two.index()]
                else {
                    return;
                };
                actor.last_teleport_tick = Some(parts.tick);
            })
            .unwrap(),
    ];
    for mutation in mutations {
        assert_eq!(
            committed.poll_reactive_command(Tick(445), &mutation, None),
            RouteStatus::Invalidated
        );
    }
}

fn with_new_public_bomb(
    observation: &grenadiers_core::cpu::Observation,
) -> grenadiers_core::cpu::Observation {
    observation
        .clone()
        .map_parts(|parts| {
            let cell = open_cell(parts);
            let owner = parts
                .actors
                .iter_mut()
                .find_map(|actor| match actor {
                    grenadiers_core::cpu::ObservedActor::Visible(actor)
                        if actor.alive && actor.live_bomb_count < actor.bomb_capacity =>
                    {
                        actor.live_bomb_count += 1;
                        Some((actor.id, actor.bomb_range))
                    }
                    _ => None,
                })
                .unwrap();
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                panic!("the retained counterfactual must expose all bombs")
            };
            bombs.push(ObservedBomb {
                id: EntityId(90),
                placed_at: parts.tick,
                cell,
                owner: owner.0,
                range: owner.1,
                trigger: ObservedBombTrigger::Timed {
                    remaining_ticks: 180,
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            });
        })
        .unwrap()
}

fn open_cell(parts: &grenadiers_core::cpu::ObservationParts) -> CellId {
    (1..parts.arena.height - 1)
        .flat_map(|row| (1..parts.arena.width - 1).map(move |column| CellId::new(column, row)))
        .find(|cell| {
            parts
                .arena
                .index(*cell)
                .is_some_and(|index| parts.arena.terrain[index] == Terrain::Floor)
                && !parts.arena.blocks.contains(cell)
                && !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.iter().any(|bomb| bomb.cell == *cell))
                && !parts.flames.iter().any(|flame| flame.cell == *cell)
                && !parts.pickups.iter().any(|pickup| pickup.cell == *cell)
                && !parts.oil.iter().any(|oil| oil.cell == *cell)
                && !parts.actors.iter().any(|actor| {
                    matches!(actor, grenadiers_core::cpu::ObservedActor::Visible(actor)
                        if actor.position.cell() == Some(*cell))
                })
        })
        .unwrap()
}
