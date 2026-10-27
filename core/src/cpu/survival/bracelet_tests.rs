use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::policy::{
    ControlPlan, UtilityMemory,
    tests::{key, observation},
};
use crate::cpu::{
    BombKnowledge, CpuRng, Intent, Observation, ObservedActor, ObservedFlame, UnknownActorFacts,
    VisibleActor, WorkBudget,
};
use crate::game::types::HALF_FOOTPRINT;
use crate::game::{
    ActorInit, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, OilState,
    PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, World,
};
use crate::room::RoomSettings;

use super::{SurvivalPlan, WORK_ALLOWANCE};

fn bracelet_observation(flame_ticks: u16) -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::BRACELET);
    parts.actors[0] = ObservedActor::Visible(VisibleActor {
        id: EntityId(1),
        seat: SeatId::One,
        position: Position::center(CellId::new(8, 7)),
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
    });
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.heading = Direction::East;
    cpu.equipment = Some(Equipment::Bracelet);
    parts.flames.push(ObservedFlame {
        cell: CellId::new(9, 7),
        remaining_ticks: flame_ticks,
    });
    Observation::new(parts).unwrap()
}

fn tactic(observation: &Observation) -> Option<SurvivalPlan> {
    let memory = UtilityMemory::new(key());
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let hazards = forecast(
        observation,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .ok()?;
    let ordinary = ControlPlan {
        intent: Intent::moving(Direction::East),
        hold_ticks: 12,
        next_memory: memory,
    };
    super::bracelet::plan(
        observation,
        &hazards,
        observation.parts().tick,
        &memory,
        &ordinary,
        &mut work,
    )
}

#[test]
fn certified_straight_approach_targets_fire_with_a_guarded_lease() {
    let observed = bracelet_observation(40);
    let planned = tactic(&observed).expect("the adjacent rival can be shoved into fire");
    assert_eq!(planned.control.intent, Intent::moving(Direction::East));
    assert_eq!(planned.control.hold_ticks, 12);
    assert!(planned.execution.is_none());
    let lease = planned.lease.expect("Bracelet movement remains a pulse");
    assert_eq!(
        lease.assurance(),
        &crate::cpu::routing::RouteAssurance::Certified
    );
    assert!(lease.bracelet_guard_allows(&observed));
}

#[test]
fn expiring_fire_or_a_blocked_victim_sweep_withholds_the_tactic() {
    assert!(tactic(&bracelet_observation(14)).is_none());
    assert!(
        tactic(&bracelet_observation(15)).is_some(),
        "the flame expiring one tick after latest contact is still useful"
    );

    let mut blocked = bracelet_observation(40).into_parts();
    blocked.arena.blocks.push(CellId::new(9, 7));
    assert!(tactic(&Observation::new(blocked).unwrap()).is_none());
}

#[test]
fn tangent_rival_or_unknown_bombs_withholds_the_tactic() {
    let mut tangent = bracelet_observation(40).into_parts();
    let ObservedActor::Visible(rival) = &mut tangent.actors[0] else {
        unreachable!()
    };
    rival.position.y += 2 * HALF_FOOTPRINT;
    assert!(tactic(&Observation::new(tangent).unwrap()).is_none());

    let mut unknown = bracelet_observation(40).into_parts();
    unknown.bombs = BombKnowledge::Unavailable;
    assert!(tactic(&Observation::new(unknown).unwrap()).is_none());
}

#[test]
fn exhausted_tactic_work_fails_without_an_action() {
    let observed = bracelet_observation(40);
    let memory = UtilityMemory::new(key());
    let hazards = forecast(
        &observed,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(
        super::bracelet::plan(
            &observed,
            &hazards,
            observed.parts().tick,
            &memory,
            &ControlPlan::stop(memory),
            &mut WorkBudget::new(0),
        )
        .is_none()
    );
}

#[test]
fn failed_tactic_preserves_the_normal_survival_fallback() {
    let mut blocked = bracelet_observation(40).into_parts();
    blocked.arena.blocks.push(CellId::new(9, 7));
    let observed = Observation::new(blocked).unwrap();
    assert!(tactic(&observed).is_none());

    let planned = super::plan(
        &observed,
        &UtilityMemory::new(key()),
        &ObservedHazardMemory::new(key()),
        observed.parts().tick,
        &mut CpuRng::from_seed(key(), [8; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(planned.lease.is_some() || planned.execution.is_some());
    assert_ne!(
        planned.control.intent.manual,
        Some(crate::cpu::ManualIntent::Equipment)
    );
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn cooldown_state() -> (State, RoomSettings) {
    let enabled =
        EnabledItems::from_bits(EnabledItems::BRACELET.bits() | EnabledItems::OIL_BARREL.bits())
            .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let arena = crate::game::test_support::open_border_arena(BTreeSet::new());
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(4, 4), PickupKind::Bracelet)]),
        BTreeMap::from([(CellId::new(7, 4), OilState::Burning)]),
    )
    .unwrap();
    let mut state = State::new(
        RulesConfig::expanded_v5(settings),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position { x: 5_248, y: 4_608 },
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position { x: 4_608, y: 4_608 },
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    state.step(&[]).unwrap();
    assert_eq!(
        state.actors()[1].inventory().equipment(),
        Some(Equipment::Bracelet)
    );
    let shove = state
        .step(&[input(SeatId::Two, InputAction::Press(Direction::East))])
        .unwrap();
    assert!(shove.forced_movement.is_some());
    state
        .step(&[input(SeatId::Two, InputAction::Clear)])
        .unwrap();
    state
        .step(&[input(SeatId::Two, InputAction::Press(Direction::East))])
        .unwrap();
    for _ in 0..12 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[input(SeatId::Two, InputAction::Clear)])
        .unwrap();
    (state, settings)
}

fn ready_copy(cooldown: &State, settings: RoomSettings) -> State {
    let arena = crate::game::test_support::open_border_arena(BTreeSet::new());
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(
            cooldown.actors()[1].position().cell().unwrap(),
            PickupKind::Bracelet,
        )]),
        BTreeMap::from([(CellId::new(7, 4), OilState::Burning)]),
    )
    .unwrap();
    let actors = SeatId::ALL.map(|seat| {
        let actor = &cooldown.actors()[seat.index()];
        ActorInit {
            id: actor.id(),
            seat,
            position: actor.position(),
            heading: actor.heading(),
            speed: actor.speed(),
        }
    });
    let mut ready = State::new(RulesConfig::expanded_v5(settings), world, actors).unwrap();
    ready.step(&[]).unwrap();
    for _ in 1..cooldown.tick().0 {
        ready.step(&[]).unwrap();
    }
    ready
}

#[test]
fn private_cooldown_is_publicly_inert_and_no_shove_keeps_the_cpu_safe() {
    let (mut cooldown, settings) = cooldown_state();
    let mut ready = ready_copy(&cooldown, settings);
    assert_eq!(ready.public_view(), cooldown.public_view());

    let mut ready_shoved = false;
    let mut cooldown_shoved = false;
    for update in 0..12 {
        let inputs = (update == 0)
            .then_some(input(SeatId::Two, InputAction::Press(Direction::East)))
            .into_iter()
            .collect::<Vec<_>>();
        ready_shoved |= ready.step(&inputs).unwrap().forced_movement.is_some();
        cooldown_shoved |= cooldown.step(&inputs).unwrap().forced_movement.is_some();
    }
    assert!(ready_shoved);
    assert!(!cooldown_shoved);
    assert!(cooldown.actors()[SeatId::Two.index()].alive());
    assert!(cooldown.actors()[SeatId::One.index()].alive());
}
