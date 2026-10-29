use std::sync::{Arc, atomic::AtomicBool, mpsc::sync_channel};

use grenadiers_core::cpu::policy::{ControlPlan, UtilityMemory};
use grenadiers_core::cpu::survival::SurvivalPlan;
use grenadiers_core::cpu::{
    CpuRng, DecisionId, Observation, ObservedActor, UnknownActorFacts, VisibleActor,
};
use grenadiers_core::game::{CellId, Direction, EntityId, Position, SeatId, SpeedLevel, Tick};
use grenadiers_core::room::CpuProfile;

use super::{CpuController, Pending};
use crate::game::rooms::cpu::worker::{CpuPlanner, PlanResult};

fn visible(seat: SeatId) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(CellId::new(5 + seat.index() as u16, 5)),
        heading: Direction::West,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

fn observation(seat: SeatId) -> Observation {
    let mut parts = crate::game::rooms::cpu::tests::observation_for(SeatId::Two).into_parts();
    parts.rules.version = 6;
    match seat {
        SeatId::Two => parts.actors.push(ObservedActor::Hidden {
            id: EntityId(3),
            seat: SeatId::Three,
            alive: true,
        }),
        SeatId::Three => {
            parts.recipient = SeatId::Three;
            parts.actors[1] = ObservedActor::Hidden {
                id: EntityId(2),
                seat: SeatId::Two,
                alive: true,
            };
            parts.actors.push(visible(SeatId::Three));
        }
        SeatId::Four => {
            parts.recipient = SeatId::Four;
            parts.actors[1] = ObservedActor::Hidden {
                id: EntityId(2),
                seat: SeatId::Two,
                alive: true,
            };
            parts.actors.push(ObservedActor::Hidden {
                id: EntityId(3),
                seat: SeatId::Three,
                alive: true,
            });
            parts.actors.push(visible(SeatId::Four));
        }
        SeatId::One => unreachable!("the creator seat is not CPU-controlled"),
    }
    Observation::new(parts).unwrap()
}

fn controller(planner: CpuPlanner, seat: SeatId, profile: CpuProfile) -> CpuController {
    let mut controller =
        CpuController::new(planner, seat, 7, [seat.index() as u8 + 1; 32], profile);
    controller.reset_world(3);
    controller
}

#[test]
fn v17_easy_and_standard_accept_rules_six_and_run_the_real_policy() {
    for profile in [
        CpuProfile::MULTIPLAYER_STANDARD,
        CpuProfile::MULTIPLAYER_EASY,
    ] {
        let mut controller = controller(CpuPlanner::without_workers(), SeatId::Two, profile);
        let observed = observation(SeatId::Two);
        assert!(controller.accept_observation(&observed));
        let accepted = controller.plan_now_for_test(&observed, 4);
        assert_eq!(accepted.decision.world, observed.parts().world);
        assert_eq!(accepted.decision.observation_tick, observed.parts().tick);
    }
}

#[test]
fn a_cross_seat_completion_is_rejected_without_committing_rng() {
    let mut controller = controller(
        CpuPlanner::without_workers(),
        SeatId::Three,
        CpuProfile::MULTIPLAYER_STANDARD,
    );
    let own = observation(SeatId::Three);
    assert!(controller.accept_observation(&own));
    controller.settled_stop_tick = Some(own.parts().tick);
    let foreign = observation(SeatId::Two);
    let decision = DecisionId {
        world: own.parts().world,
        command_epoch: 4,
        observation_tick: own.parts().tick,
        checked_request_serial: 1,
    };
    let foreign_key = grenadiers_core::cpu::MemoryKey {
        world: foreign.parts().world,
        cpu_seat: SeatId::Two,
    };
    let (sender, receiver) = sync_channel(1);
    sender
        .send(PlanResult {
            plan: SurvivalPlan {
                control: ControlPlan::stop(UtilityMemory::new(foreign_key)),
                lease: None,
                execution: None,
            },
            contact: None,
            basis: foreign,
            rng: CpuRng::from_seed(foreign_key, [8; 32]),
            escape_origin: None,
        })
        .unwrap();
    controller.pending = Some(Pending {
        decision,
        receiver,
        live: Arc::new(AtomicBool::new(true)),
        completed: None,
    });
    controller.request_serial = 1;
    assert!(controller.poll(Tick(20), 4).is_none());
    assert_eq!(controller.rng_draws(), 0);
}

#[test]
fn multiplayer_controllers_keep_rng_memory_and_pending_cancellation_independent() {
    let planner = CpuPlanner::without_workers();
    let mut two = controller(
        planner.clone(),
        SeatId::Two,
        CpuProfile::MULTIPLAYER_STANDARD,
    );
    let mut three = controller(
        planner.clone(),
        SeatId::Three,
        CpuProfile::MULTIPLAYER_STANDARD,
    );
    let two_observation = observation(SeatId::Two);
    let three_observation = observation(SeatId::Three);
    assert!(two.accept_observation(&two_observation));
    assert!(three.accept_observation(&three_observation));
    two.settled_stop_tick = Some(two_observation.parts().tick);
    three.settled_stop_tick = Some(three_observation.parts().tick);
    assert!(two.submit(two_observation, 4));
    assert!(three.submit(three_observation, 4));
    assert!(two.has_pending() && three.has_pending());
    two.cancel();
    assert!(!two.has_pending());
    assert!(three.has_pending());
    let report = planner.pump_for_test(2);
    assert_eq!(report.cancelled, 1);
    assert_eq!(report.delivered, 1);
    assert_eq!(two.rng_draws(), 0);
    assert_eq!(three.rng_draws(), 0);
    assert_eq!(two.memory.as_ref().unwrap().key().cpu_seat, SeatId::Two);
    assert_eq!(three.memory.as_ref().unwrap().key().cpu_seat, SeatId::Three);
}

#[test]
fn seat_four_accepts_both_v17_levels_and_rejects_two_and_three_results() {
    for profile in [
        CpuProfile::MULTIPLAYER_STANDARD,
        CpuProfile::MULTIPLAYER_EASY,
    ] {
        let mut controller = controller(CpuPlanner::without_workers(), SeatId::Four, profile);
        let own = observation(SeatId::Four);
        assert!(controller.accept_observation(&own));
        let accepted = controller.plan_now_for_test(&own, 4);
        assert_eq!(accepted.decision.world, own.parts().world);
        assert_eq!(accepted.decision.observation_tick, own.parts().tick);
    }

    for foreign_seat in [SeatId::Two, SeatId::Three] {
        let mut controller = controller(
            CpuPlanner::without_workers(),
            SeatId::Four,
            CpuProfile::MULTIPLAYER_STANDARD,
        );
        let own = observation(SeatId::Four);
        assert!(controller.accept_observation(&own));
        controller.settled_stop_tick = Some(own.parts().tick);
        let foreign = observation(foreign_seat);
        let decision = DecisionId {
            world: own.parts().world,
            command_epoch: 4,
            observation_tick: own.parts().tick,
            checked_request_serial: 1,
        };
        let foreign_key = grenadiers_core::cpu::MemoryKey {
            world: foreign.parts().world,
            cpu_seat: foreign_seat,
        };
        let (sender, receiver) = sync_channel(1);
        sender
            .send(PlanResult {
                plan: SurvivalPlan {
                    control: ControlPlan::stop(UtilityMemory::new(foreign_key)),
                    lease: None,
                    execution: None,
                },
                contact: None,
                basis: foreign,
                rng: CpuRng::from_seed(foreign_key, [8; 32]),
                escape_origin: None,
            })
            .unwrap();
        controller.pending = Some(Pending {
            decision,
            receiver,
            live: Arc::new(AtomicBool::new(true)),
            completed: None,
        });
        controller.request_serial = 1;
        assert!(controller.poll(Tick(20), 4).is_none());
        assert_eq!(controller.rng_draws(), 0);
    }
}
