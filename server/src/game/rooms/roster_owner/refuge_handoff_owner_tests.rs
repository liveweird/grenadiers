use grenadiers_core::cpu::{BombKnowledge, Observation, ObservedActor};
use grenadiers_core::game::{SeatId, Tick};
use grenadiers_core::room::CpuProfile;

use super::refuge_handoff_owner_support::{EPOCH, accepted_refuge, driver, terminal_refuge};
use crate::game::rooms::cpu::CpuController;
use crate::game::rooms::roster_runtime::RosterAction;

#[test]
fn owner_nonfatal_stop_or_bomb_refusal_keeps_the_old_authority() {
    for admitted_before_apply in [8, 7] {
        let (fixture, accepted) = accepted_refuge();
        let world = fixture.observation.parts().world;
        let rng_draws = fixture.controller.rng_draws();
        let memory = fixture.controller.intent_debug_summary_for(world);
        let mut owner = driver(fixture.controller);
        let runtime_epoch = owner.runtime.lifecycle().round_attempt();
        for _ in 0..admitted_before_apply {
            owner
                .runtime
                .submit_cpu_intent(CPU, runtime_epoch, RosterAction::Movement(None))
                .unwrap();
        }

        owner.apply_refuge_handoff(0, accepted).unwrap();

        assert!(owner.cpus[0].has_committed_route());
        assert_eq!(owner.cpus[0].rng_draws(), rng_draws);
        assert_eq!(owner.cpus[0].intent_debug_summary_for(world), memory);
    }
}

#[test]
fn owner_successfully_admitted_stop_and_bomb_commit_the_waiting_replacement() {
    let (fixture, accepted) = accepted_refuge();
    assert!(matches!(
        fixture.controller.plant_presence(),
        Some(grenadiers_core::cpu::routing::PlantPresence::Present(_))
    ));
    let mut owner = driver(fixture.controller);

    owner.apply_refuge_handoff(0, accepted).unwrap();

    assert_eq!(
        owner.cpus[0].plant_presence(),
        Some(grenadiers_core::cpu::routing::PlantPresence::Awaiting)
    );
    assert_eq!(owner.runtime.next_cpu_manual_action(CPU), Some(2));
}

#[test]
fn third_seat_urgent_job_claims_the_last_queue_slot_before_optional_refuge_work() {
    let fixture = terminal_refuge();
    let planner = fixture.planner.clone();
    let urgent_observation = stopped_observation(&fixture.observation, SeatId::Three);
    let mut fillers = Vec::new();
    for seed in 0..31 {
        let mut filler =
            stopped_controller(planner.clone(), urgent_observation.clone(), [seed; 32]);
        assert!(filler.submit(urgent_observation.clone(), EPOCH));
        fillers.push(filler);
    }
    let urgent = stopped_controller(planner, urgent_observation.clone(), [99; 32]);
    let dummy = stopped_controller(
        fixture.planner.clone(),
        urgent_observation.clone(),
        [98; 32],
    );
    let mut owner = driver(fixture.controller);
    owner.cpus = vec![owner.cpus.remove(0), dummy, urgent];
    owner.submit_cpu_candidates(
        &[
            Some(&fixture.observation),
            Some(&urgent_observation),
            Some(&urgent_observation),
        ],
        &[false, false, true],
        &[true, false, false],
        EPOCH,
    );

    assert!(
        owner.cpus[2].has_pending(),
        "third-seat urgent job was queued"
    );
    assert!(
        !owner.cpus[0].has_pending(),
        "optional job saw the full queue"
    );
    assert!(owner.cpus[0].has_committed_route());
    assert_eq!(fillers.len(), 31);
}

fn stopped_controller(
    planner: crate::game::rooms::cpu::CpuPlanner,
    observation: Observation,
    seed: [u8; 32],
) -> CpuController {
    let mut controller = CpuController::new(
        planner,
        observation.parts().recipient,
        observation.parts().world.room_instance,
        seed,
        CpuProfile::HARD_V35,
    );
    let mut prior = observation.clone().into_parts();
    prior.tick = Tick(prior.tick.0 - 1);
    prior.rules.running_updates -= 1;
    prior.rules.remaining_ticks += 1;
    if let BombKnowledge::Visible(bombs) = &mut prior.bombs {
        for bomb in bombs {
            if let grenadiers_core::cpu::ObservedBombTrigger::Timed {
                remaining_ticks,
                paused: false,
            } = &mut bomb.trigger
            {
                *remaining_ticks += 1;
            }
        }
    }
    let prior = Observation::new_rules_v10(
        prior,
        observation.environment().unwrap().clone(),
        observation.respawn_lairs().cloned(),
        observation.spike_pits().cloned(),
    )
    .unwrap();
    assert!(controller.accept_observation(&prior));
    controller.on_stop_admitted(prior.parts().tick);
    assert!(controller.accept_observation(&observation));
    controller
}

fn stopped_observation(source: &Observation, recipient: SeatId) -> Observation {
    source
        .clone()
        .map_parts(|parts| {
            parts.recipient = recipient;
            let ObservedActor::Visible(actor) = &mut parts.actors[recipient.index()] else {
                panic!("fixture recipient must be visible")
            };
            actor.voluntary_actions_frozen = false;
        })
        .unwrap()
}

const CPU: SeatId = SeatId::Two;
