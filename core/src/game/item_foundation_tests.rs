use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::effects::{EffectKind, TimedEffects};
use crate::game::input::Direction;
use crate::game::inventory::{Equipment, SpecialWeapon};
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::{StepError, World, fixed_arena};

fn state_at(tick: Tick) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    State::new_at_tick(
        RulesConfig::classic_v1(),
        World::new(arena),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 1),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        tick,
    )
    .unwrap()
}

fn expose_one_to_fire(state: &mut State, expires_at: Tick) {
    let cell = state.actors[0].position.cell().unwrap();
    state.world.hazards.insert(cell, expires_at);
}

#[test]
fn heart_absorbs_both_fire_passes_and_shield_preserves_heart() {
    let mut heart = state_at(Tick(0));
    heart.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    expose_one_to_fire(&mut heart, Tick(100));
    let report = heart.step(&[]).unwrap();
    assert!(heart.actors[0].alive());
    assert_eq!(heart.actors[0].inventory().equipment(), None);
    let grace = heart.actors[0].damage_grace.unwrap();
    assert!(grace.is_active(Tick(1)));
    assert_eq!(grace.remaining_ticks(Tick(1)), 60);
    assert!(report.events.is_empty());

    let mut shield = state_at(Tick(0));
    shield.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    shield.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    expose_one_to_fire(&mut shield, Tick(100));
    shield.step(&[]).unwrap();
    assert!(shield.actors[0].alive());
    assert_eq!(
        shield.actors[0].inventory().equipment(),
        Some(Equipment::Heart)
    );
    assert_eq!(shield.actors[0].damage_grace, None);
}

#[test]
fn effects_expire_before_damage_and_death_clears_all_transient_state() {
    let mut state = state_at(Tick(299));
    state.actors[0].inventory.replace_equipment(Equipment::Kick);
    state.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Remote);
    state.actors[0]
        .effects
        .apply(EffectKind::Shield, Tick(0))
        .unwrap();
    state.actors[0]
        .effects
        .apply(EffectKind::InvisibilityCloak, Tick(0))
        .unwrap();
    expose_one_to_fire(&mut state, Tick(400));

    state.step(&[]).unwrap();

    assert!(!state.actors[0].alive());
    assert_eq!(state.actors[0].inventory(), &Default::default());
    assert_eq!(state.actors[0].effects(), &TimedEffects::default());
    assert_eq!(state.actors[0].damage_grace, None);
}

#[test]
fn derived_bomb_stats_are_captured_without_mutating_base_upgrades() {
    let mut state = state_at(Tick(0));
    for kind in [
        EffectKind::GoldenFlame,
        EffectKind::ShortFlame,
        EffectKind::ShortFuse,
        EffectKind::LongFuse,
    ] {
        state.actors[0].effects.apply(kind, Tick(0)).unwrap();
    }
    let base_range = state.actors[0].bomb_range();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let bomb = state.world.bombs.values().next().unwrap();
    assert_eq!(bomb.range(), 1);
    assert_eq!(bomb.detonates_at(), Tick(181));
    assert_eq!(state.actors[0].bomb_range(), base_range);
}

#[test]
fn slow_foot_exposes_exact_rate_but_does_not_change_classic_movement_before_activation() {
    let mut control = state_at(Tick(0));
    let mut slowed = control.clone();
    slowed.actors[0]
        .effects
        .apply(EffectKind::SlowFoot, Tick(0))
        .unwrap();
    let input = [SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(Direction::East),
    }];

    control.step(&input).unwrap();
    slowed.step(&input).unwrap();

    assert_eq!(slowed.actors[0].position(), control.actors[0].position());
    let effective = slowed.actors[0].effective_speed(Tick(1));
    assert_eq!(effective.numerator(), SpeedLevel::Base.units_per_second());
    assert_eq!(effective.denominator(), 2);
}

#[test]
fn a_new_round_starts_with_empty_foundation_state() {
    let state = state_at(Tick(9_000));
    for actor in state.actors() {
        assert_eq!(actor.inventory(), &Default::default());
        assert_eq!(actor.effects(), &TimedEffects::default());
        assert_eq!(actor.damage_grace, None);
    }
}

#[test]
fn heart_absorbs_damage_on_the_final_tick_and_overflow_step_is_atomic() {
    let mut state = state_at(Tick(u64::MAX - 1));
    state.actors[0]
        .inventory
        .replace_equipment(Equipment::Heart);
    let cell = state.actors[0].position.cell().unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(90),
        cell,
        EntityId(2),
        1,
        Tick(u64::MAX - 1),
        Tick(u64::MAX),
    )
    .unwrap();

    state.step(&[]).unwrap();
    assert!(state.actors[0].alive());
    assert_eq!(state.actors[0].inventory().equipment(), None);
    assert!(
        state.actors[0]
            .damage_grace
            .is_some_and(|grace| grace.is_active(Tick(u64::MAX)))
    );

    let terminal = state.clone();
    assert_eq!(state.step(&[]), Err(StepError::TickOverflow));
    assert_eq!(state, terminal);
}
