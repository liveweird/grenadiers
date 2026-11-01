use crate::cpu::{
    DecisionId, Observation, ObservedActor, ObservedEnvironment, WorkBudget, WorldId,
};
use crate::game::{
    Direction, Position, STORM_CLOUD_SPEED_PER_TICK, SeatId, SpecialWeapon, SpeedLevel,
    StormCloudConfig, StormCloudView, Tick,
};

use super::teleporter_execution_tests::{after_move, at, basis, decision, plan, step};
use super::{PreparedTeleporterStep, TELEPORTER_ADMISSION_ALLOWANCE, TeleporterStepStatus};

#[test]
fn admission_rejects_world_seat_root_rival_pickup_and_capability_changes() {
    let original = basis();
    let mut cases = Vec::new();
    cases.push(
        original
            .clone()
            .map_parts(|parts| {
                parts.world = WorldId {
                    room_instance: parts.world.room_instance + 1,
                    ..parts.world
                };
            })
            .unwrap(),
    );
    cases.push(
        original
            .clone()
            .map_parts(|parts| parts.recipient = SeatId::One)
            .unwrap(),
    );
    cases.push(
        original
            .clone()
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                    unreachable!()
                };
                actor.position.x += 1;
            })
            .unwrap(),
    );
    cases.push(
        original
            .clone()
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[0] else {
                    unreachable!()
                };
                actor.position = Position {
                    x: 13_312,
                    y: 1_536,
                };
            })
            .unwrap(),
    );
    cases.push(
        original
            .clone()
            .map_parts(|parts| parts.pickups.clear())
            .unwrap(),
    );
    cases.push(
        original
            .clone()
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                    unreachable!()
                };
                actor.speed = SpeedLevel::One;
            })
            .unwrap(),
    );
    for changed in cases {
        assert_rejected(&original, &changed, decision(&original));
    }
}

#[test]
fn admission_rejects_hidden_rival_wrong_decision_and_changed_job_basis() {
    let original = basis();
    let mut parts = original.clone().into_parts();
    let actor = match &parts.actors[0] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => unreachable!(),
    };
    parts.actors[0] = ObservedActor::Hidden {
        id: actor.id,
        seat: actor.seat,
        alive: true,
    };
    let hidden = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Disabled,
            None,
            vec![None, Some(0), Some(0), Some(0)],
        ),
        None,
        None,
    )
    .unwrap();
    assert_rejected(&original, &hidden, decision(&original));

    let wrong = DecisionId {
        world: WorldId {
            physical_round: 99,
            ..original.parts().world
        },
        ..decision(&original)
    };
    assert_rejected(&original, &original, wrong);

    let changed_basis = at(&original, 1);
    let current = changed_basis.clone();
    assert!(
        PreparedTeleporterStep::admit(
            plan(step(&original, Tick(6_111)).unwrap()),
            changed_basis,
            &current,
            Tick(6_111),
            decision(&original),
            current.parts().tick,
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        )
        .is_err()
    );
}

#[test]
fn stopped_fence_accepts_bounded_rival_motion_but_rejects_new_actor_facts() {
    let basis = basis();
    let mut moving_rival = after_move(&basis, 2, Direction::West)
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.position.x += 16;
        })
        .unwrap();
    assert_completed(&basis, &moving_rival);

    let dead_rival = after_move(&basis, 2, Direction::West)
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.alive = false;
        })
        .unwrap();
    assert_completed(&basis, &dead_rival);

    let cases = [
        Box::new(|parts: &mut crate::cpu::ObservationParts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.bomb_capacity += 1;
        }) as Box<dyn Fn(&mut crate::cpu::ObservationParts)>,
        Box::new(|parts: &mut crate::cpu::ObservationParts| {
            let ObservedActor::Visible(root) = &mut parts.actors[2] else {
                unreachable!()
            };
            root.live_bomb_count += 1;
        }),
        Box::new(|parts: &mut crate::cpu::ObservationParts| {
            let ObservedActor::Visible(root) = &mut parts.actors[2] else {
                unreachable!()
            };
            root.last_teleport_tick = Some(Tick(6_999));
        }),
    ];
    for change in cases {
        moving_rival = after_move(&basis, 2, Direction::West)
            .map_parts(change)
            .unwrap();
        assert_invalidated(&basis, &moving_rival);
    }
}

#[test]
fn real_equipped_rival_death_clears_public_inventory_and_grace() {
    let basis = equipped_storm_basis();
    let cleared = death_publication(&basis, false);
    assert_completed(&basis, &cleared);

    let uncleared_fields = [
        Box::new(|actor: &mut crate::cpu::VisibleActor| {
            actor.equipment = Some(crate::game::Equipment::Punch);
        }) as Box<dyn Fn(&mut crate::cpu::VisibleActor)>,
        Box::new(|actor: &mut crate::cpu::VisibleActor| {
            actor.weapon = Some(SpecialWeapon::Power);
        }),
        Box::new(|actor: &mut crate::cpu::VisibleActor| {
            actor.damage_grace_remaining_ticks = 1;
        }),
    ];
    for change in uncleared_fields {
        let uncleared = cleared
            .clone()
            .map_parts(|parts| {
                let ObservedActor::Visible(partner) = &mut parts.actors[1] else {
                    unreachable!()
                };
                change(partner);
            })
            .unwrap();
        assert_invalidated(&basis, &uncleared);
    }
    assert_invalidated(&basis, &with_storm_timer(cleared, 1, 1));

    let root_dead = death_publication(&basis, true);
    assert_invalidated(&basis, &root_dead);
}

fn with_storm_timer(observation: Observation, index: usize, remaining: u16) -> Observation {
    let old = observation.environment().unwrap();
    let mut timers = old.actor_storm_slow_remaining_ticks().to_vec();
    timers[index] = Some(remaining);
    let environment = ObservedEnvironment::new(old.storm_cloud_config(), old.storm_cloud(), timers);
    let respawn = observation.respawn_lairs().cloned();
    let pits = observation.spike_pits().cloned();
    Observation::new_rules_v10(observation.into_parts(), environment, respawn, pits).unwrap()
}

fn equipped_storm_basis() -> Observation {
    let captured = crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121();
    let mut parts = captured.clone().into_parts();
    let ObservedActor::Visible(partner) = &mut parts.actors[1] else {
        unreachable!()
    };
    partner.weapon = Some(SpecialWeapon::Power);
    partner.damage_grace_remaining_ticks = 4;
    let old = captured.environment().unwrap();
    let mut timers = old.actor_storm_slow_remaining_ticks().to_vec();
    timers[1] = Some(5);
    let environment = ObservedEnvironment::new(old.storm_cloud_config(), old.storm_cloud(), timers);
    let respawn = captured.respawn_lairs().cloned();
    let pits = captured.spike_pits().cloned();
    Observation::new_rules_v10(parts, environment, respawn, pits).unwrap()
}

fn death_publication(basis: &Observation, root_dead: bool) -> Observation {
    let mut parts = basis.clone().into_parts();
    parts.tick = Tick(parts.tick.0 + 2);
    parts.rules.running_updates += 2;
    parts.rules.remaining_ticks -= 2;
    let ObservedActor::Visible(root) = &mut parts.actors[2] else {
        unreachable!()
    };
    root.heading = Direction::West;
    let index = if root_dead { 2 } else { 1 };
    let ObservedActor::Visible(dead) = &mut parts.actors[index] else {
        unreachable!()
    };
    dead.alive = false;
    dead.equipment = None;
    dead.weapon = None;
    dead.effects.clear();
    dead.damage_grace_remaining_ticks = 0;
    let old = basis.environment().unwrap();
    let cloud = old.storm_cloud().unwrap();
    let mut timers = old.actor_storm_slow_remaining_ticks().to_vec();
    timers[index] = Some(0);
    let environment = ObservedEnvironment::new(
        StormCloudConfig::Single,
        Some(StormCloudView {
            position: Position {
                y: cloud.position.y - 2 * STORM_CLOUD_SPEED_PER_TICK,
                ..cloud.position
            },
            leg_remaining_ticks: cloud.leg_remaining_ticks - 2,
            ..cloud
        }),
        timers,
    );
    Observation::new_rules_v10(
        parts,
        environment,
        basis.respawn_lairs().cloned(),
        basis.spike_pits().cloned(),
    )
    .unwrap()
}

fn assert_completed(basis: &Observation, stopped: &Observation) {
    assert!(matches!(
        committed(basis).observe_stopped(
            stopped,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        ),
        TeleporterStepStatus::Completed(_)
    ));
}

fn assert_invalidated(basis: &Observation, stopped: &Observation) {
    assert!(matches!(
        committed(basis).observe_stopped(
            stopped,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        ),
        TeleporterStepStatus::Invalidated
    ));
}

fn committed(basis: &Observation) -> super::CommittedTeleporterStep {
    let memory = crate::cpu::policy::UtilityMemory::new(crate::cpu::MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    });
    let mut prepared = PreparedTeleporterStep::admit(
        plan(step(basis, Tick(6_111)).unwrap()),
        basis.clone(),
        basis,
        Tick(6_111),
        decision(basis),
        basis.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    )
    .unwrap();
    let movement = prepared.command().unwrap();
    let mut committed = prepared.commit(movement).unwrap();
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), memory.key(), 7)
    else {
        unreachable!()
    };
    assert!(committed.record_stop(stop));
    committed
}

fn assert_rejected(basis: &Observation, current: &Observation, decision: DecisionId) {
    assert!(
        PreparedTeleporterStep::admit(
            plan(step(basis, Tick(6_111)).unwrap()),
            basis.clone(),
            current,
            Tick(6_111),
            decision,
            current.parts().tick,
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        )
        .is_err()
    );
}
