use std::collections::BTreeMap;

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::lair_contact::tests::observation as lair_observation;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    CpuRng, MemoryKey, MovementIntent, Observation, ObservedActor, ObservedEffect, ObservedFlame,
    ObservedLairs, ObservedPickup, WorkBudget, WorldId,
};
use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, InputAction,
    PickupKind, Position, RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed,
    SpeedLevel, State, StormCloudConfig, Tick, World, fixed_arena,
};
use crate::room::{LairConfig, LairSettings, RoomSettings};

use super::plan_lairs;

const WORLD: WorldId = WorldId {
    room_instance: 91,
    physical_round: 1,
};
const CPU_CELL: CellId = CellId::new(1, 1);
const TRAP_CELL: CellId = CellId::new(3, 3);
const PICKUP_CELL: CellId = CPU_CELL;

fn state(kind: PickupKind) -> State {
    let enabled = EnabledItems::from_bits(
        EnabledItems::SLOW_FOOT.bits() | EnabledItems::REVERSE_CONTROLS.bits(),
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let mut arena = fixed_arena();
    let blocks = arena.blocks().clone();
    arena.remove_blocks(&blocks);
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(PICKUP_CELL, kind)]),
    )
    .unwrap();
    State::new_at_tick(
        RulesConfig::environment_v8(
            settings,
            StormCloudConfig::Disabled,
            LairConfig::Enabled(LairSettings::new(5, 3).unwrap()),
        ),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(CPU_CELL),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(100),
    )
    .unwrap()
}

fn observe(state: &State) -> Observation {
    let lairs = state.lairs().unwrap();
    let mobs = state.mobs();
    let fixture_mobs = mobs
        .iter()
        .cloned()
        .map(|mut mob| {
            mob.id = EntityId(mob.id.0 + 1_000);
            mob
        })
        .collect();
    let basis = lair_observation(fixture_mobs, Tick(400));
    let environment = basis.environment().unwrap().clone();
    let mut parts = basis.into_parts();
    parts.world = WORLD;
    parts.tick = state.tick();
    parts.rules.running_updates = u16::try_from(state.running_updates()).unwrap();
    parts.rules.remaining_ticks =
        u16::try_from(state.rules().round_ticks().saturating_sub(state.tick().0)).unwrap();
    parts.rules.settings = state.rules().room_settings().unwrap();
    let rival = state.actor(SeatId::One).unwrap();
    parts.actors[0] = ObservedActor::Hidden {
        id: rival.id(),
        seat: SeatId::One,
        alive: rival.alive(),
    };
    let actor = state.actor(SeatId::Two).unwrap();
    let ObservedActor::Visible(observed) = &mut parts.actors[1] else {
        unreachable!()
    };
    observed.position = actor.position();
    observed.heading = actor.heading();
    observed.speed = actor.speed();
    observed.alive = actor.alive();
    observed.bomb_capacity = actor.bomb_capacity();
    observed.bomb_range = actor.bomb_range();
    observed.live_bomb_count = u16::try_from(state.live_bomb_count(SeatId::Two)).unwrap();
    observed.equipment = actor.inventory().equipment();
    observed.weapon = actor.inventory().weapon();
    observed.effects = actor
        .effect_observations()
        .map(|(kind, expires_at)| ObservedEffect {
            kind,
            remaining_ticks: u16::try_from(expires_at.0 - state.tick().0).unwrap(),
        })
        .collect();
    observed.voluntary_actions_frozen = state.voluntary_actions_frozen()[1];
    observed.last_teleport_tick = state.last_teleport_tick();
    parts.pickups = state
        .world()
        .exposed_pickups()
        .iter()
        .map(|(&cell, pickup)| ObservedPickup {
            cell,
            kind: pickup.kind(),
        })
        .collect();
    parts.flames = state
        .world()
        .hazards()
        .iter()
        .map(|(&cell, &expires_at)| ObservedFlame {
            cell,
            remaining_ticks: u16::try_from(expires_at.0 - state.tick().0).unwrap(),
        })
        .collect();
    Observation::new_rules_v8(
        parts,
        environment,
        ObservedLairs::new(state.rules().lair_config(), lairs.to_vec(), mobs),
    )
    .unwrap()
}

fn pending_teleporter_trap() -> State {
    let enabled = EnabledItems::from_bits(
        EnabledItems::TELEPORTER.bits()
            | EnabledItems::WALL_PASS.bits()
            | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let mut arena = fixed_arena();
    let blocks = arena.blocks().clone();
    arena.remove_blocks(&blocks);
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::from([(TRAP_CELL, PickupKind::Teleporter)]),
        BTreeMap::new(),
    )
    .unwrap();
    let mut state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v8(
            settings,
            StormCloudConfig::Disabled,
            LairConfig::Enabled(LairSettings::new(5, 3).unwrap()),
        ),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(CPU_CELL),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([91; 32]),
    )
    .unwrap();
    state.insert_test_flame(TRAP_CELL, Tick(301));
    while state.tick() < Tick(300) {
        state.step(&[]).unwrap();
    }
    let mob = state.mobs()[0].position;
    state.set_test_actor_pose(SeatId::One, mob, Direction::West);
    state.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(300));
    state.apply_test_effect(SeatId::One, EffectKind::InvisibilityCloak, Tick(300));
    let center = Position::center(TRAP_CELL);
    state.set_test_actor_pose(
        SeatId::Two,
        Position {
            x: center.x + 100,
            y: center.y,
        },
        Direction::East,
    );
    state.replace_test_actor_equipment(SeatId::Two, Equipment::WallPass);
    state.apply_test_effect(SeatId::Two, EffectKind::Shield, Tick(1));
    state
}

fn key() -> MemoryKey {
    MemoryKey {
        world: WORLD,
        cpu_seat: SeatId::Two,
    }
}

#[test]
fn physics_changing_pickups_execute_one_update_then_replan_from_real_state() {
    for (kind, effect) in [
        (PickupKind::MovementSpeed, None),
        (
            PickupKind::ReverseControls,
            Some(EffectKind::ReverseControls),
        ),
        (PickupKind::SlowFoot, Some(EffectKind::SlowFoot)),
    ] {
        let mut state = state(kind);
        let before = observe(&state);
        let key = key();
        let first = plan_lairs(
            &before,
            &UtilityMemory::new(key),
            &ObservedHazardMemory::new(key),
            Tick(100),
            &mut CpuRng::from_seed(key, [41; 32]),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(first.survival.control.hold_ticks, 1, "{kind:?}");
        let MovementIntent::Direction(direction) = first.survival.control.intent.movement else {
            panic!("{kind:?} did not advance toward the pickup")
        };

        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(direction),
            }])
            .unwrap();
        assert!(
            !state.world().exposed_pickups().contains_key(&PICKUP_CELL),
            "{kind:?} remained at {:?}",
            state.actor(SeatId::Two).unwrap().position()
        );
        let actor = state.actor(SeatId::Two).unwrap();
        if let Some(effect) = effect {
            assert!(actor.effect_observations().any(|(kind, _)| kind == effect));
        } else {
            assert_eq!(actor.speed(), SpeedLevel::One);
        }

        let changed = observe(&state);
        let second = plan_lairs(
            &changed,
            &first.survival.control.next_memory,
            &ObservedHazardMemory::new(key),
            changed.parts().tick,
            &mut CpuRng::from_seed(key, [42; 32]),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(second.survival.control.intent.manual, None, "{kind:?}");
        assert!(
            second.contact.is_some() || second.survival.control.intent == crate::cpu::Intent::STOP
        );
    }
}

#[test]
fn pending_teleporter_collection_is_rejected_before_real_npc_endpoint_death() {
    let mut state = pending_teleporter_trap();
    let observed = observe(&state);
    assert!(matches!(
        observed.parts().actors[0],
        ObservedActor::Hidden { .. }
    ));
    assert_eq!(observed.parts().flames[0].remaining_ticks, 1);
    let ObservedActor::Visible(cpu) = &observed.parts().actors[1] else {
        unreachable!()
    };
    assert!(
        cpu.effects
            .iter()
            .any(|effect| { effect.kind == EffectKind::Shield && effect.remaining_ticks == 1 })
    );
    let key = key();
    let plan = plan_lairs(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        observed.parts().tick,
        &mut CpuRng::from_seed(key, [51; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(plan.survival.control.intent, crate::cpu::Intent::STOP);
    assert!(plan.contact.is_none());

    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.last_teleport_tick(), Some(Tick(301)));
    assert!(!state.actor(SeatId::Two).unwrap().alive());
}
