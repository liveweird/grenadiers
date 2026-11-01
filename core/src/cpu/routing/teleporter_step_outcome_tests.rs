use crate::cpu::{ObservedActor, WorkBudget};
use crate::game::{Direction, Position, Tick};

use super::teleporter_step_tests::{ALLOWANCE, collect_observation, forecast_for, request};
use super::{TeleporterStepPhase, certify_teleporter_step};

#[test]
fn collect_retains_clipped_no_swap_and_every_living_partner_landing() {
    let observation = collect_observation();
    let forecast = forecast_for(&observation);
    let proof = certify_teleporter_step(
        &observation,
        &forecast,
        request(&observation, Direction::North),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap();
    assert_eq!(proof.phase(), TeleporterStepPhase::Collect);

    // A future blocker may refuse all motion or clip any prefix. Collection
    // may consume the pickup yet refuse its swap, leaving the full endpoint.
    for position in [
        Position {
            x: 3_584,
            y: 10_250,
        },
        Position {
            x: 3_584,
            y: 10_225,
        },
        Position {
            x: 3_584,
            y: 10_199,
        },
    ] {
        assert!(proof.allows_stopped_position(position), "{position:?}");
    }

    // Every other living actor is a possible seeded partner, including human
    // seats. Each current pose belongs to its arbitrary-input landing region.
    for actor in observation
        .parts()
        .actors
        .iter()
        .filter_map(|actor| match actor {
            ObservedActor::Visible(actor)
                if actor.alive && actor.seat != observation.parts().recipient =>
            {
                Some(actor)
            }
            ObservedActor::Visible(_) | ObservedActor::Hidden { .. } => None,
        })
    {
        assert!(
            proof.allows_stopped_position(actor.position),
            "partner {:?}",
            actor.seat
        );
    }
    assert!(!proof.allows_stopped_position(Position {
        x: 14_336,
        y: 12_288,
    }));
    assert_eq!(proof.start_window().start, Tick(6_122));
    assert_eq!(proof.start_window().end, Tick(6_129));
}

#[test]
fn approach_does_not_gain_teleport_landing_authority() {
    let observation = crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121();
    let forecast = forecast_for(&observation);
    let proof = certify_teleporter_step(
        &observation,
        &forecast,
        request(&observation, Direction::West),
        &mut WorkBudget::new(ALLOWANCE),
    )
    .unwrap();
    let partner = match &observation.parts().actors[0] {
        ObservedActor::Visible(actor) => actor.position,
        ObservedActor::Hidden { .. } => unreachable!(),
    };
    assert_eq!(proof.phase(), TeleporterStepPhase::Approach);
    assert!(!proof.allows_stopped_position(partner));
}
