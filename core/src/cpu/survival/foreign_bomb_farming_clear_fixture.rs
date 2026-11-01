use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::{BombKnowledge, MemoryKey, Observation, ObservedActor, ObservedEnvironment};
use crate::game::{CellId, Direction, EntityId, Position, StormCloudConfig, StormCloudView, Tick};

use super::foreign_bomb_farming_fixture::{CPU, captured_basis};

pub(super) const SETTLED_STOP: Tick = Tick(432);
pub(super) const FARM_TARGET: CellId = CellId::new(11, 2);

pub(super) struct ClearBasis {
    pub observation: Observation,
    pub hazard_memory: ObservedHazardMemory,
    pub memory: UtilityMemory,
}

pub(super) fn captured_clear_basis() -> ClearBasis {
    let original = captured_basis().observation;
    let respawn_lairs = original.respawn_lairs().unwrap().clone();
    let spike_pits = original.spike_pits().unwrap().clone();
    let mut parts = original.into_parts();
    parts.tick = SETTLED_STOP;
    parts.rules.running_updates = 252;
    parts.rules.remaining_ticks = 10_548;
    parts
        .arena
        .blocks
        .retain(|cell| !matches!((cell.column, cell.row), (1 | 13, 8)));
    set_actor(
        &mut parts.actors,
        EntityId(2),
        11_327,
        1_536,
        Direction::East,
    );
    set_actor(
        &mut parts.actors,
        EntityId(3),
        3_479,
        11_776,
        Direction::East,
    );
    set_actor(
        &mut parts.actors,
        EntityId(4),
        11_881,
        11_776,
        Direction::West,
    );
    for actor in &mut parts.actors {
        let ObservedActor::Visible(actor) = actor else {
            unreachable!()
        };
        actor.live_bomb_count = 0;
    }
    parts.bombs = BombKnowledge::Visible(Vec::new());
    parts.flames.clear();
    let observation = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 7_648, y: 6_656 },
                direction: Direction::East,
                leg_remaining_ticks: 4,
            }),
            vec![Some(0); 4],
        ),
        Some(respawn_lairs),
        Some(spike_pits),
    )
    .unwrap();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut hazard_memory = ObservedHazardMemory::new(key);
    hazard_memory.observe(&observation).unwrap();
    let memory = UtilityMemory::from_route(key, &[CellId::new(11, 1)], false)
        .with_strategic_intent(StrategicIntent {
            objective: StrategicObjectiveKind::Farm,
            target: StrategicTarget::Block { cell: FARM_TARGET },
            approach_cell: Some(CellId::new(11, 1)),
            phase: StrategicPhase::Approach,
            last_switch_reason: StrategicSwitchReason::Stalled,
            observation_tick: Tick(417),
            score: 2,
            committed_until: Tick(504),
            progress_deadline: Tick(570),
            last_distance: 1,
            last_cpu_position: Position {
                x: 10_752,
                y: 1_497,
            },
            deferred_until: Some(Tick(504)),
        })
        .with_failed_target(Some((
            StrategicTarget::Block {
                cell: CellId::new(9, 1),
            },
            Tick(504),
        )));
    ClearBasis {
        observation,
        hazard_memory,
        memory,
    }
}

fn set_actor(actors: &mut [ObservedActor], id: EntityId, x: i32, y: i32, heading: Direction) {
    let actor = actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.id == id => Some(actor),
            _ => None,
        })
        .unwrap();
    actor.position = Position { x, y };
    actor.heading = heading;
}
