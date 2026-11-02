use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{StrategicTarget, UtilityMemory};
use crate::cpu::{
    BombKnowledge, CpuRng, ManualIntent, MemoryKey, MovementIntent, Observation, ObservationParts,
    ObservedActor, ObservedArena, ObservedBomb, ObservedBombTrigger, ObservedEnvironment,
    ObservedFlame, ObservedOil, ObservedPickup, ObservedRules, UnknownActorFacts, UnknownBombFacts,
    VisibleActor, WorkBudget, WorldId,
};
use crate::game::{
    ActorInit, Arena, BombTrigger, CellId, Direction, EnabledItems, EntityId, InputAction,
    OilState, PickupKind, Position, RoundAssignment, RulesConfig, SeatId, SeatInput,
    SimulationSeed, State, StormCloudConfig, Terrain, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

use super::{
    CommittedRoute, ExecutionCertificate, LairSurvivalPlan, RouteStatus, WORK_ALLOWANCE,
    plan_reliable_v27_with_origin, plan_spike_pits_v26_with_origin, strategic_plant_still_targets,
};

const CPU: SeatId = SeatId::Two;
const TARGET: CellId = CellId::new(8, 7);
const ORIGIN: CellId = CellId::new(7, 7);
pub(super) const SAFE_EXIT: CellId = CellId::new(6, 7);
pub(super) const REMOTE_OIL: CellId = CellId::new(11, 7);
const WORLD: WorldId = WorldId {
    room_instance: 27,
    physical_round: 1,
};

fn settings(teleporter: bool) -> RoomSettings {
    let bits = EnabledItems::OIL_BARREL.bits()
        | if teleporter {
            EnabledItems::TELEPORTER.bits()
        } else {
            0
        };
    RoomSettings::DEFAULT.with_enabled_items(EnabledItems::from_bits(bits).unwrap())
}

fn arena() -> Arena {
    let fixed = fixed_arena();
    let mut terrain = std::array::from_fn(|index| {
        let column = u16::try_from(index % 15).unwrap();
        let row = u16::try_from(index / 15).unwrap();
        fixed.terrain(CellId::new(column, row)).unwrap()
    });
    for cell in [CellId::new(7, 6), CellId::new(7, 8)] {
        terrain[usize::from(cell.row) * 15 + usize::from(cell.column)] = Terrain::Wall;
    }
    Arena::new(
        terrain,
        BTreeSet::from([TARGET]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

pub(super) fn state(
    lairs: bool,
    oil: Option<CellId>,
    teleporter_enabled: bool,
    concealed_teleporter: bool,
) -> (State, RoomSettings) {
    let room_settings = settings(teleporter_enabled);
    let concealed = concealed_teleporter
        .then_some((TARGET, PickupKind::Teleporter))
        .into_iter()
        .collect();
    let oil = oil
        .map(|cell| BTreeMap::from([(cell, OilState::Burning)]))
        .unwrap_or_default();
    let world =
        World::with_pickups_and_oil(arena(), concealed, BTreeMap::new(), BTreeMap::new(), oil)
            .unwrap();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: room_settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(ORIGIN),
            heading: Direction::East,
            speed: room_settings.starting_speed(),
        },
    ];
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            room_settings,
            StormCloudConfig::Disabled,
            lairs,
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([27; 32]),
    )
    .unwrap();
    (state, room_settings)
}

pub(super) fn observe(state: &State, room_settings: RoomSettings) -> Observation {
    observe_with_lairs(state, (room_settings, RespawnLairSettings::new(5).unwrap()))
}

pub(crate) fn observe_with_lairs(
    state: &State,
    config: (RoomSettings, RespawnLairSettings),
) -> Observation {
    let view = state.public_view();
    let actors = view
        .actors
        .iter()
        .map(|actor| {
            ObservedActor::Visible(VisibleActor {
                id: actor.id,
                seat: actor.seat,
                position: actor.position,
                heading: actor.heading,
                speed: actor.speed,
                alive: actor.alive,
                bomb_capacity: actor.bomb_capacity,
                bomb_range: actor.bomb_range,
                live_bomb_count: u16::try_from(actor.live_bomb_count).unwrap(),
                equipment: None,
                weapon: None,
                effects: Vec::new(),
                damage_grace_remaining_ticks: 0,
                voluntary_actions_frozen: false,
                last_teleport_tick: None,
                unknown: UnknownActorFacts::ALL,
            })
        })
        .collect();
    let bombs = view
        .bombs
        .iter()
        .map(|bomb| ObservedBomb {
            id: bomb.id,
            placed_at: bomb.placed_at,
            cell: bomb.cell,
            owner: bomb.owner,
            range: bomb.range,
            trigger: match bomb.trigger {
                BombTrigger::Timed { detonates_at } => ObservedBombTrigger::Timed {
                    remaining_ticks: u16::try_from(detonates_at.0 - view.tick.0).unwrap(),
                    paused: false,
                },
                BombTrigger::Remote => ObservedBombTrigger::Remote,
                BombTrigger::Contact => ObservedBombTrigger::Contact,
            },
            blast: bomb.blast,
            unknown: UnknownBombFacts::ALL,
        })
        .collect();
    let parts = ObservationParts {
        world: WORLD,
        tick: view.tick,
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: u16::try_from(view.running_updates).unwrap(),
            remaining_ticks: u16::try_from(view.remaining_ticks).unwrap(),
            settings: config.0,
        },
        arena: ObservedArena {
            width: view.arena.width,
            height: view.arena.height,
            terrain: view.arena.terrain,
            blocks: view.arena.blocks,
        },
        actors,
        bombs: BombKnowledge::Visible(bombs),
        flames: view
            .flames
            .into_iter()
            .filter_map(|flame| {
                let remaining = flame.expires_at.0.checked_sub(view.tick.0)?;
                (remaining > 0).then_some(ObservedFlame {
                    cell: flame.cell,
                    remaining_ticks: u16::try_from(remaining).ok()?,
                })
            })
            .collect(),
        oil: view
            .oil_barrels
            .into_iter()
            .map(|oil| ObservedOil {
                cell: oil.cell,
                state: oil.state,
            })
            .collect(),
        pickups: view
            .pickups
            .into_iter()
            .map(|pickup| ObservedPickup {
                cell: pickup.cell,
                kind: pickup.kind,
            })
            .collect(),
    };
    let respawn_lairs = super::observation_test_adapter::projected_respawn_lairs(state, config.1);
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Disabled,
            None,
            vec![Some(0); state.actors().len()],
        ),
        respawn_lairs,
        None,
    )
    .unwrap()
}

pub(super) fn plan(observation: &Observation, reliable: bool) -> (LairSurvivalPlan, u32) {
    let key = MemoryKey {
        world: WORLD,
        cpu_seat: CPU,
    };
    let mut hazards = ObservedHazardMemory::new(key);
    let _ = hazards.observe(observation);
    let mut rng = CpuRng::from_seed(key, [27; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = if reliable {
        plan_reliable_v27_with_origin(
            observation,
            &UtilityMemory::new(key),
            &hazards,
            observation.parts().tick,
            None,
            &mut rng,
            &mut work,
        )
    } else {
        plan_spike_pits_v26_with_origin(
            observation,
            &UtilityMemory::new(key),
            &hazards,
            observation.parts().tick,
            None,
            &mut rng,
            &mut work,
        )
    }
    .unwrap();
    (planned, work.spent())
}

fn movement_input(movement: MovementIntent) -> Vec<SeatInput> {
    match movement {
        MovementIntent::Direction(direction) => vec![SeatInput {
            seat: CPU,
            action: InputAction::Press(direction),
        }],
        MovementIntent::Stop => vec![SeatInput {
            seat: CPU,
            action: InputAction::Clear,
        }],
    }
}

pub(super) fn execute(
    state: &mut State,
    room_settings: RoomSettings,
    basis: Observation,
    planned: LairSurvivalPlan,
) {
    let control = planned.survival.control;
    let execution = planned.survival.execution.unwrap();
    let admission_tick = Tick(execution.manual_execution_tick().unwrap().0 - 1);
    while state.tick() < admission_tick {
        state.step(&[]).unwrap();
    }
    let current = observe(state, room_settings);
    let mut route = CommittedRoute::admit_execution(
        execution,
        basis,
        &current,
        admission_tick,
        control.intent.movement,
    )
    .unwrap();
    let mut inputs = movement_input(control.intent.movement);
    inputs.push(SeatInput {
        seat: CPU,
        action: InputAction::PlaceBomb,
    });
    state.step(&inputs).unwrap();
    let mut settled_stop =
        (control.intent.movement == MovementIntent::Stop).then_some(state.tick());
    for _ in 0..300 {
        let current = observe(state, room_settings);
        if route.observe_with_stop(&current, settled_stop) == RouteStatus::Invalidated {
            assert!(!state.world().arena().blocks().contains(&TARGET));
            assert!(current.parts().flames.is_empty());
            assert!(
                current.parts().pickups.iter().any(|pickup| {
                    pickup.cell == TARGET && pickup.kind == PickupKind::Teleporter
                })
            );
            return;
        }
        let status = route.poll_command(state.tick(), settled_stop);
        let (inputs, submitted_stop) = match status {
            RouteStatus::Submit(command) => {
                assert!(route.commit_command(command));
                (
                    movement_input(command.movement()),
                    command.movement() == MovementIntent::Stop,
                )
            }
            RouteStatus::Completed => return,
            RouteStatus::Invalidated => panic!(
                "certified oil route invalidated at {:?}: {:?}",
                state.tick(),
                current.parts()
            ),
            RouteStatus::Continue | RouteStatus::Wait => (Vec::new(), false),
        };
        state.step(&inputs).unwrap();
        if submitted_stop {
            settled_stop = Some(state.tick());
        } else if matches!(status, RouteStatus::Submit(_)) {
            settled_stop = None;
        }
    }
    panic!("certified oil route did not complete")
}

#[test]
fn v27_alone_plants_past_remote_persistent_oil_and_executes_authoritatively() {
    for lairs in [false, true] {
        let (mut state, room_settings) = state(lairs, Some(REMOTE_OIL), true, false);
        let basis = observe(&state, room_settings);
        assert!(matches!(&basis.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()));
        assert!(basis.parts().flames.is_empty());
        assert!(basis.parts().pickups.is_empty());

        let (archived, archived_work) = plan(&basis, false);
        assert_eq!(
            archived.survival.control.intent.manual, None,
            "lairs={lairs}"
        );
        assert!(archived_work <= WORK_ALLOWANCE);

        let (reliable, reliable_work) = plan(&basis, true);
        assert_eq!(
            reliable.survival.control.intent.manual,
            Some(ManualIntent::Bomb),
            "lairs={lairs}"
        );
        assert!(reliable_work <= WORK_ALLOWANCE);
        assert_eq!(reliable.contact.is_some(), lairs);
        let intent = reliable
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap();
        assert_eq!(intent.target, StrategicTarget::Block { cell: TARGET });
        let certificate = reliable
            .survival
            .execution
            .as_ref()
            .and_then(ExecutionCertificate::plant)
            .unwrap();
        assert!(strategic_plant_still_targets(
            &basis,
            intent,
            certificate,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ));

        execute(&mut state, room_settings, basis, reliable);
        assert!(state.actor(CPU).unwrap().alive(), "lairs={lairs}");
        assert!(!state.world().arena().blocks().contains(&TARGET));
    }
}
