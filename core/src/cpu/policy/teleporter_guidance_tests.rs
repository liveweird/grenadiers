use super::*;

fn captured() -> (Observation, UtilityMemory) {
    let observation = crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121();
    let key = crate::cpu::MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    (observation, UtilityMemory::new(key))
}

fn at_tick(observation: Observation, tick: Tick) -> Observation {
    observation
        .map_parts(|parts| {
            let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
            parts.tick = tick;
            parts.rules.running_updates += elapsed;
            parts.rules.remaining_ticks -= elapsed;
        })
        .unwrap()
}

fn move_cpu_toward(observation: Observation, waypoint: CellId, units: i32) -> Observation {
    observation
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            let target = crate::game::Position::center(waypoint);
            if actor.position.x != target.x {
                actor.position.x += units * (target.x - actor.position.x).signum();
            } else {
                actor.position.y += units * (target.y - actor.position.y).signum();
            }
        })
        .unwrap()
}

#[test]
fn retained_approach_requires_the_named_teleporter() {
    let key = crate::cpu::MemoryKey {
        world: crate::cpu::WorldId {
            room_instance: 1,
            physical_round: 1,
        },
        cpu_seat: crate::game::SeatId::One,
    };
    let target = CellId::new(3, 9);
    let approach = CellId::new(3, 10);
    let intent = StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell: target,
            kind: PickupKind::Teleporter,
        },
        approach_cell: Some(approach),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(1),
        score: 10,
        committed_until: Tick(2),
        progress_deadline: Tick(3),
        last_distance: 1,
        last_cpu_position: crate::game::Position::center(CellId::new(1, 1)),
        deferred_until: None,
    };
    let memory = UtilityMemory::new(key).with_strategic_intent(intent);
    assert_eq!(retained_approach(memory, target), Some(approach));
    assert_eq!(retained_approach(memory, CellId::new(4, 9)), None);
}

#[test]
fn captured_scene_retains_one_legal_neighbor_approach() {
    let (observation, memory) = captured();
    let first = teleporter_guidance(&observation, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let retained = teleporter_guidance(
        &observation,
        &first.next_memory,
        &mut WorkBudget::new(32_768),
    )
    .unwrap();
    assert_eq!(retained.target, CellId::new(3, 9));
    assert_eq!(retained.approach, first.approach);
    assert!(retained.candidates.iter().flatten().count() <= 2);
}

#[test]
fn adjacent_center_guides_directly_toward_the_named_pickup() {
    let (basis, memory) = captured();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let observation = basis
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = crate::game::Position::center(first.approach);
        })
        .unwrap();
    let guidance = teleporter_guidance(
        &observation,
        &first.next_memory,
        &mut WorkBudget::new(32_768),
    )
    .unwrap();
    assert_eq!(guidance.approach, first.approach);
    assert_eq!(
        guidance.candidates[0].unwrap().movement,
        MovementIntent::Direction(direction(first.approach, guidance.target).unwrap())
    );
    assert_eq!(guidance.candidates[0].unwrap().waypoint, guidance.target);
}

#[test]
fn identical_pose_expires_without_renewing_the_only_approach() {
    let (basis, memory) = captured();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let deadline = first
        .next_memory
        .strategic_intent()
        .unwrap()
        .progress_deadline;
    let expired = at_tick(basis, deadline);
    assert!(
        teleporter_guidance(&expired, &first.next_memory, &mut WorkBudget::new(32_768)).is_none()
    );
}

#[test]
fn expired_only_approach_returns_the_supplied_selector_fallback_unchanged() {
    let (basis, memory) = captured();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let deadline = first
        .next_memory
        .strategic_intent()
        .unwrap()
        .progress_deadline;
    let expired = at_tick(basis, deadline);
    let mut history = crate::cpu::forecast::ObservedHazardMemory::new(memory.key());
    history.observe(&expired).unwrap();
    let control = crate::cpu::policy::ControlPlan::stop(memory);
    let fallback = crate::cpu::survival::LairSurvivalPlan {
        survival: crate::cpu::survival::SurvivalPlan {
            control,
            lease: None,
            execution: None,
        },
        contact: None,
    };
    let selected = crate::cpu::survival::select_teleporter_recovery(
        &expired,
        &history,
        expired.parts().tick,
        &first.next_memory,
        None,
        fallback,
        &mut WorkBudget::new(65_536),
    );
    let crate::cpu::survival::TeleporterRecoveryChoice::Ordinary(plan) = selected else {
        panic!("expired sole approach must decline to the supplied fallback")
    };
    assert_eq!(plan.survival.control, control);
    assert!(plan.contact.is_none());
}

#[test]
fn expired_approach_uses_one_alternate_then_declines_instead_of_oscillating() {
    let (basis, memory) = captured();
    let basis = basis
        .map_parts(|parts| {
            parts.arena.blocks.retain(|cell| *cell != CellId::new(2, 9));
        })
        .unwrap();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let first_intent = first.next_memory.strategic_intent().unwrap();
    let expired = at_tick(basis.clone(), first_intent.progress_deadline);
    let alternate =
        teleporter_guidance(&expired, &first.next_memory, &mut WorkBudget::new(32_768)).unwrap();
    let alternate_intent = alternate.next_memory.strategic_intent().unwrap();
    assert_ne!(alternate.approach, first.approach);
    assert_eq!(
        alternate_intent.last_switch_reason,
        StrategicSwitchReason::Stalled
    );

    let occupied_alternate = at_tick(
        expired
            .clone()
            .map_parts(|parts| {
                let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                    unreachable!()
                };
                rival.position = crate::game::Position::center(alternate.approach);
            })
            .unwrap(),
        Tick(expired.parts().tick.0 + 1),
    );
    assert!(
        teleporter_guidance(
            &occupied_alternate,
            &alternate.next_memory,
            &mut WorkBudget::new(32_768)
        )
        .is_none(),
        "an invalidated Stalled alternate must not rotate back to the first approach"
    );

    let expired_again = at_tick(expired, alternate_intent.progress_deadline);
    assert!(
        teleporter_guidance(
            &expired_again,
            &alternate.next_memory,
            &mut WorkBudget::new(32_768)
        )
        .is_none()
    );
}

#[test]
fn real_route_subcell_progress_renews_deadline_but_not_commitment() {
    let (basis, memory) = captured();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let first_intent = first.next_memory.strategic_intent().unwrap();
    let progressed = at_tick(
        move_cpu_toward(basis, first.candidates[0].unwrap().waypoint, 20),
        first_intent.progress_deadline,
    );
    let renewed = teleporter_guidance(
        &progressed,
        &first.next_memory,
        &mut WorkBudget::new(32_768),
    )
    .unwrap();
    let renewed_intent = renewed.next_memory.strategic_intent().unwrap();
    assert_eq!(renewed.approach, first.approach);
    assert!(renewed_intent.progress_deadline > first_intent.progress_deadline);
    assert_eq!(renewed_intent.committed_until, first_intent.committed_until);
}

#[test]
fn progress_from_neighbor_center_is_measured_toward_the_teleporter() {
    let (basis, memory) = captured();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let adjacent = basis
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = crate::game::Position::center(first.approach);
        })
        .unwrap();
    let at_neighbor =
        teleporter_guidance(&adjacent, &first.next_memory, &mut WorkBudget::new(32_768)).unwrap();
    let neighbor_intent = at_neighbor.next_memory.strategic_intent().unwrap();
    let progressed = at_tick(
        move_cpu_toward(adjacent, at_neighbor.target, 20),
        neighbor_intent.progress_deadline,
    );
    let renewed = teleporter_guidance(
        &progressed,
        &at_neighbor.next_memory,
        &mut WorkBudget::new(32_768),
    )
    .unwrap();
    assert!(
        renewed
            .next_memory
            .strategic_intent()
            .unwrap()
            .progress_deadline
            > neighbor_intent.progress_deadline
    );
}

#[test]
fn target_disappearance_declines_and_fresh_memory_resets_guidance() {
    let (basis, memory) = captured();
    let first = teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).unwrap();
    let missing = basis
        .clone()
        .map_parts(|parts| {
            parts
                .pickups
                .retain(|pickup| pickup.kind != PickupKind::Teleporter);
        })
        .unwrap();
    assert!(
        teleporter_guidance(&missing, &first.next_memory, &mut WorkBudget::new(32_768)).is_none()
    );
    assert!(teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768)).is_some());
}
