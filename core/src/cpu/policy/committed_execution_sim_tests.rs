use std::collections::BTreeSet;

use crate::cpu::{
    BombKnowledge, ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedFlame,
    UnknownBombFacts,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    ActorInit, Arena, BlastKind, CellId, Direction, EntityId, InputAction, Position, RulesConfig,
    SeatId, SeatInput, SpeedLevel, State, Terrain, World,
};
use crate::room::RoomSettings;

use super::tests::{key, observation};
use super::{UtilityMemory, follow_committed_escape};

fn open_arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || column == ARENA_WIDTH - 1 || row == 0 || row == ARENA_HEIGHT - 1 {
                let index = usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column);
                terrain[index] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn observed(state: &State, settings: RoomSettings) -> crate::cpu::Observation {
    let actor = &state.actors()[1];
    let mut parts = observation(actor.position().cell().unwrap()).into_parts();
    parts.tick = state.tick();
    parts.rules.running_updates = u16::try_from(state.running_updates()).unwrap();
    parts.rules.remaining_ticks = 10_800 - parts.rules.running_updates;
    parts.rules.settings = settings;
    let ObservedActor::Visible(visible) = &mut parts.actors[1] else {
        unreachable!()
    };
    visible.position = actor.position();
    visible.heading = actor.heading();
    visible.speed = actor.speed();
    visible.live_bomb_count = state.world().bombs().len() as u16;
    parts.bombs = BombKnowledge::Visible(
        state
            .world()
            .bombs()
            .values()
            .map(|bomb| ObservedBomb {
                id: bomb.id(),
                placed_at: bomb.placed_at(),
                cell: bomb.cell(),
                owner: bomb.owner(),
                range: bomb.range(),
                trigger: ObservedBombTrigger::Timed {
                    remaining_ticks: u16::try_from(
                        bomb.detonates_at()
                            .unwrap()
                            .0
                            .saturating_sub(state.tick().0),
                    )
                    .unwrap(),
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            })
            .collect(),
    );
    parts.flames = state
        .world()
        .hazards()
        .iter()
        .map(|(&cell, &expires)| ObservedFlame {
            cell,
            remaining_ticks: u16::try_from(expires.0.saturating_sub(state.tick().0)).unwrap(),
        })
        .collect();
    crate::cpu::Observation::new(parts).unwrap()
}

#[test]
fn committed_escape_finishes_without_another_worker_result_before_shortest_fuse() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 3, 60).unwrap();
    let arena = open_arena();
    let spawns = arena.spawns();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::West,
        speed: SpeedLevel::Four,
    });
    let mut state = State::new(
        RulesConfig::configurable_v4(settings),
        World::new(arena),
        actors,
    )
    .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let origin = spawns[1];
    let refuge = CellId::new(origin.column - 1, origin.row - 1);
    let mut memory = UtilityMemory::from_route(
        key(),
        &[origin, CellId::new(origin.column - 1, origin.row), refuge],
        true,
    );
    let mut refuge_tick = None;
    let mut completed = false;
    for _ in 0..20 {
        let observation = observed(&state, settings);
        let Some(plan) = follow_committed_escape(&observation, &memory).unwrap() else {
            completed = true;
            break;
        };
        assert_eq!(plan.intent.manual, None);
        memory = plan.next_memory;
        match plan.intent.movement {
            crate::cpu::MovementIntent::Direction(direction) => {
                state
                    .step(&[SeatInput {
                        seat: SeatId::Two,
                        action: InputAction::Press(direction),
                    }])
                    .unwrap();
                for _ in 1..plan.hold_ticks {
                    state.step(&[]).unwrap();
                }
            }
            crate::cpu::MovementIntent::Stop => {}
        }
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Clear,
            }])
            .unwrap();
        for _ in 0..4 {
            state.step(&[]).unwrap();
        }
        let position = state.actors()[1].position();
        if position.x.abs_diff(Position::center(refuge).x) <= 64
            && position.y.abs_diff(Position::center(refuge).y) <= 64
            && refuge_tick.is_none()
        {
            refuge_tick = Some(state.tick().0);
        }
    }
    let reached_at = refuge_tick.expect("committed servo reaches its refuge");
    assert!(reached_at < 60, "reached refuge at tick {reached_at}");
    assert!(
        completed,
        "escape remains committed after visible hazards clear"
    );
    assert!(state.actors()[1].alive());
}
