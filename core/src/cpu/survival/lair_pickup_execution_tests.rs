use std::collections::BTreeMap;

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::lair_contact::tests::observation as lair_observation;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    CpuRng, MemoryKey, MovementIntent, Observation, ObservedActor, ObservedEffect, ObservedLairs,
    ObservedPickup, WorkBudget, WorldId,
};
use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction, PickupKind,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, StormCloudConfig, Tick, World,
    fixed_arena,
};
use crate::room::{LairConfig, LairSettings, RoomSettings};

use super::plan_lairs;

const WORLD: WorldId = WorldId {
    room_instance: 91,
    physical_round: 1,
};
const CPU_CELL: CellId = CellId::new(1, 1);
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
    let basis = lair_observation(mobs.clone(), lairs[0].next_spawn_at);
    let environment = basis.environment().unwrap().clone();
    let mut parts = basis.into_parts();
    parts.world = WORLD;
    parts.tick = state.tick();
    parts.rules.running_updates = u16::try_from(state.running_updates()).unwrap();
    parts.rules.remaining_ticks =
        u16::try_from(state.rules().round_ticks().saturating_sub(state.tick().0)).unwrap();
    parts.rules.settings = state.rules().room_settings().unwrap();
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
    Observation::new_rules_v8(
        parts,
        environment,
        ObservedLairs::new(state.rules().lair_config(), lairs.to_vec(), mobs),
    )
    .unwrap()
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
