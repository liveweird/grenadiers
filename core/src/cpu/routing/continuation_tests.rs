use super::RouteFailure;
use super::continuation::validate_legs;
use super::continuation_types::{MAX_CONTINUATION_LEGS, MAX_CONTINUATION_MOVES, NominalLeg};
use super::motion::{Motion, advance};
use super::tests::{actual_state_with_pickup, forecast_for, observation};
use super::validation::validate;
use crate::cpu::forecast::TickWindow;
use crate::cpu::{MovementIntent, ObservedActor, ObservedEffect};
use crate::game::{
    Direction, EffectKind, EnabledItems, InputAction, PickupKind, SeatId, SeatInput, Tick,
};
use crate::room::RoomSettings;

#[test]
fn nominal_guidance_has_distinct_bounded_geometry() {
    let legs = [
        NominalLeg::Move {
            direction: Direction::South,
            updates: 40,
        },
        NominalLeg::Stop { updates: 3 },
        NominalLeg::Move {
            direction: Direction::East,
            updates: 17,
        },
    ];
    assert_eq!(validate_legs(&legs), Ok(()));
    assert_eq!(MAX_CONTINUATION_LEGS, 8);
    assert_eq!(MAX_CONTINUATION_MOVES, 96);

    let too_long = [NominalLeg::Move {
        direction: Direction::East,
        updates: 97,
    }];
    assert_eq!(validate_legs(&too_long), Err(RouteFailure::Capacity));
    assert_eq!(
        validate_legs(&[NominalLeg::Stop { updates: 1 }]),
        Err(RouteFailure::Capacity)
    );
}

#[test]
fn cloned_checkpoint_preserves_residual_and_reverse_expiry_boundary() {
    let initial = observation(Vec::new(), Vec::new());
    let ObservedActor::Visible(initial_actor) = &initial.parts().actors[1] else {
        unreachable!()
    };
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let mut state = actual_state_with_pickup(
        initial_actor.position,
        initial_actor.heading,
        initial_actor.speed,
        settings,
        Some(PickupKind::ReverseControls),
    );
    state.step(&[]).unwrap();
    let expiry = state
        .actor(SeatId::Two)
        .unwrap()
        .effects()
        .observations()
        .find_map(|(kind, expiry)| (kind == EffectKind::ReverseControls).then_some(expiry))
        .unwrap();
    while state.tick().0 + 2 < expiry.0 {
        state.step(&[]).unwrap();
    }
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.settings = settings;
    parts.tick = state.tick();
    parts.rules.running_updates = u16::try_from(state.tick().0).unwrap();
    parts.rules.remaining_ticks -= u16::try_from(state.tick().0).unwrap();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    let actual = state.actor(SeatId::Two).unwrap();
    actor.position = actual.position();
    actor.heading = actual.heading();
    actor.effects.push(ObservedEffect {
        kind: EffectKind::ReverseControls,
        remaining_ticks: 2,
    });
    let observed = crate::cpu::Observation::new(parts).unwrap();
    let forecast = forecast_for(&observed);
    let context = validate(
        &observed,
        &forecast,
        TickWindow {
            start: Tick(state.tick().0 + 1),
            end: Tick(state.tick().0 + 2),
        },
        Tick(state.tick().0 + 4),
    )
    .unwrap();
    let mut continuous = Motion::stopped(&context, state.tick());
    let first = Tick(state.tick().0 + 1);
    advance(
        &context,
        &mut continuous,
        MovementIntent::Direction(Direction::East),
        first,
    )
    .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    let at_checkpoint = continuous.snapshot();
    assert!(at_checkpoint.reversed);
    assert_eq!(at_checkpoint.heading, Direction::West);
    assert_eq!(
        at_checkpoint.position,
        state.actor(SeatId::Two).unwrap().position()
    );
    let mut resumed = continuous.clone();
    let second = Tick(first.0 + 1);
    for motion in [&mut continuous, &mut resumed] {
        advance(
            &context,
            motion,
            MovementIntent::Direction(Direction::East),
            second,
        )
        .unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(resumed.snapshot(), continuous.snapshot());
    assert!(!resumed.snapshot().reversed);
    assert_eq!(resumed.snapshot().heading, Direction::East);
    assert_eq!(
        resumed.position,
        state.actor(SeatId::Two).unwrap().position()
    );
}
