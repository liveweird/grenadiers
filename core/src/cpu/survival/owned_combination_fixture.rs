use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::{
    BombKnowledge, MemoryKey, Observation, ObservationParts, ObservedActor, ObservedArena,
    ObservedBomb, ObservedBombTrigger, ObservedEnvironment, ObservedRespawnLairs, ObservedRules,
    UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use crate::game::{
    BlastKind, CellId, Direction, EntityId, LairStatusView, Position, SeatId, SpeedLevel,
    StormCloudConfig, Terrain, Tick, lair_layout_v1,
};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings, RoomSettings};

use super::TerminalRefugeHandoff;

pub(super) struct CombinationBasis {
    pub(super) observation: Observation,
    pub(super) history: ObservedHazardMemory,
    pub(super) memory: UtilityMemory,
    pub(super) handoff: TerminalRefugeHandoff,
    pub(super) settled: Tick,
}

pub(super) fn basis() -> CombinationBasis {
    let observation = observation();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let previous = observation
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(parts.tick.0 - 1);
            parts.rules.running_updates -= 1;
            parts.rules.remaining_ticks += 1;
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            let ObservedBombTrigger::Timed {
                remaining_ticks, ..
            } = &mut bombs[0].trigger
            else {
                unreachable!()
            };
            *remaining_ticks += 1;
        })
        .unwrap();
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&previous).unwrap();
    history.observe(&observation).unwrap();
    let settled = Tick(previous.parts().tick.0 - 1);
    let memory = UtilityMemory::new(key).with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block {
            cell: CellId::new(3, 2),
        },
        approach_cell: Some(CellId::new(5, 2)),
        phase: StrategicPhase::Reengage,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: observation.parts().tick,
        score: 2,
        committed_until: Tick(observation.parts().tick.0 + 60),
        progress_deadline: Tick(observation.parts().tick.0 + 120),
        last_distance: 2,
        last_cpu_position: Position::center(CellId::new(5, 2)),
        deferred_until: None,
    });
    let actor = super::compatibility::actor(&observation).unwrap();
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        unreachable!()
    };
    let handoff = TerminalRefugeHandoff::new(
        key,
        (
            Tick(observation.parts().tick.0 - 20),
            Tick(observation.parts().tick.0 - 19),
        ),
        bombs[0].id,
        actor.position,
        observation.clone(),
        Tick(observation.parts().tick.0 + 10),
        settled,
    );
    CombinationBasis {
        observation,
        history,
        memory,
        handoff,
        settled,
    }
}

pub(super) fn replace_observation(observation: Observation) -> CombinationBasis {
    let mut changed = basis();
    let (route_tick, first_update) = changed.handoff.route_identity();
    changed.observation = observation.clone();
    changed.handoff = TerminalRefugeHandoff::new(
        changed.handoff.key(),
        (route_tick, first_update),
        changed.handoff.bomb_id(),
        changed.handoff.refuge(),
        observation,
        changed.handoff.old_survive_until(),
        changed.handoff.settled_stop_tick(),
    );
    changed
}

fn observation() -> Observation {
    let width = 11_u16;
    let height = 9_u16;
    let mut terrain = vec![Terrain::Wall; usize::from(width * height)];
    for cell in [
        CellId::new(1, 1),
        CellId::new(3, 2),
        CellId::new(4, 2),
        CellId::new(5, 1),
        CellId::new(5, 2),
        CellId::new(5, 3),
        CellId::new(6, 2),
        CellId::new(7, 2),
        CellId::new(8, 2),
    ] {
        terrain[usize::from(cell.row * width + cell.column)] = Terrain::Floor;
    }
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 6_701,
            physical_round: 1,
        },
        tick: Tick(100),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ContactV2,
            tick_rate: 60,
            running_updates: 20,
            remaining_ticks: 3_000,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width,
            height,
            terrain,
            blocks: vec![CellId::new(3, 2), CellId::new(5, 1)],
        },
        actors: vec![
            actor(1, SeatId::One, CellId::new(1, 1), 0, 1),
            actor(2, SeatId::Two, CellId::new(5, 2), 1, 2),
        ],
        bombs: BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(70),
            placed_at: Tick(80),
            cell: CellId::new(5, 3),
            owner: EntityId(2),
            range: 2,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: 100,
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 2]),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::new(10).unwrap()),
            lair_layout_v1(BoardSize::ElevenByNine)
                .into_iter()
                .map(|cell| LairStatusView {
                    cell,
                    next_spawn_at: Some(Tick(699)),
                    living_mobs: 0,
                })
                .collect(),
            Vec::new(),
        )),
        None,
    )
    .unwrap()
}

fn actor(
    id: u32,
    seat: SeatId,
    cell: CellId,
    live_bomb_count: u16,
    bomb_capacity: u16,
) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity,
        bomb_range: 2,
        live_bomb_count,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}
