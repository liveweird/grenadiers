use crate::cpu::{BombKnowledge, ObservedActor, WorldId};
use crate::game::{
    ActorInit, ArenaGenerationConfig, Bomb, BombTrigger, CellId, Direction, EffectKind,
    EnabledItems, EntityId, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId,
    SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World, generate_multiplayer_arena,
};
use crate::protocol::v14::{EventIndex, EventKind};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings, RespawnLairTuning};

use super::server_test_support::WorldCase;
use super::*;

const OWNER: EntityId = EntityId(20);

fn enabled_items() -> EnabledItems {
    EnabledItems::from_bits(
        EnabledItems::LANDMINE.bits()
            | EnabledItems::INVISIBLE_BOMBS.bits()
            | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap()
}

pub(super) fn dense_mine_case(cloaked: bool) -> WorldCase {
    let enabled = enabled_items();
    let assignment = RoundAssignment::first();
    let generated = generate_multiplayer_arena(
        [91; 32],
        enabled,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: BoardSize::FifteenByThirteen,
            assignment,
        },
    )
    .unwrap();
    let generation = generated.record();
    let (generated_world, _) = World::from_generated(generated).unwrap();
    let mut arena = generated_world.arena().clone();
    let blocks = arena.blocks().clone();
    arena.remove_blocks(&blocks);
    let mut world = World::new(arena);
    let spawns = world.arena().spawns().to_vec();
    let occupied = spawns
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let cells = (0..world.arena().height())
        .flat_map(|row| (0..world.arena().width()).map(move |column| CellId::new(column, row)))
        .filter(|cell| {
            world.arena().terrain(*cell) == Some(crate::game::Terrain::Floor)
                && !world.arena().blocks().contains(cell)
                && !occupied.contains(cell)
        })
        .take(34)
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 34);

    let lifecycle = super::server_conformance_lifecycle::lifecycle(
        PlayerCount::Two,
        BoardSize::FifteenByThirteen,
        enabled,
        true,
    );
    let tick = lifecycle.tick();
    let ordinary = Bomb::new(
        EntityId(100),
        cells[0],
        OWNER,
        2,
        tick,
        BombTrigger::Timed {
            detonates_at: Tick(tick.0 + 180),
        },
    )
    .unwrap();
    world.bombs.insert(ordinary.id(), ordinary);
    for (offset, &cell) in cells[1..].iter().enumerate() {
        let id = EntityId(101 + u32::try_from(offset).unwrap());
        let mine = Bomb::new(id, cell, OWNER, 2, tick, BombTrigger::Contact).unwrap();
        world.bombs.insert(id, mine);
    }
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + u32::try_from(seat.index()).unwrap() * 10),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v9(
        crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
    );
    let mut state = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        tick,
        assignment,
        SimulationSeed::new([92; 32]),
    )
    .unwrap();
    state.apply_test_effect(SeatId::One, EffectKind::InvisibleBombs, tick);
    if cloaked {
        state.apply_test_effect(SeatId::Two, EffectKind::InvisibilityCloak, tick);
    }
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = super::server_conformance_lifecycle::room_view_with_tuning(
        &lifecycle,
        StormCloudConfig::Disabled,
        RespawnLairTuning::new(false, RespawnLairSettings::DEFAULT),
        true,
    );
    WorldCase {
        room,
        capture,
        generation,
        state,
        lifecycle,
    }
}

fn publication(case: &WorldCase) -> BoundPublication {
    bind_expanded(
        &case.room,
        case.room.round_id,
        &case.capture,
        case.generation,
    )
    .unwrap()
}

#[test]
fn dense_mines_capture_bind_and_observe_with_one_ordinary_slot() {
    let case = dense_mine_case(false);
    assert_eq!(case.state.world().bombs().len(), 34);
    assert_eq!(case.state.live_bomb_count(SeatId::Two), 1);
    let publication = publication(&case);
    let world = WorldId {
        room_instance: 17,
        physical_round: u64::from(case.room.round_id.get()),
    };
    let owner = observe_cpu_recipient(&publication, Seat::Two, world).unwrap();
    assert!(matches!(&owner.parts().bombs, BombKnowledge::Visible(bombs) if bombs.len() == 34));
    let owner_actor = owner
        .parts()
        .actors
        .iter()
        .find(|actor| actor.id() == OWNER)
        .unwrap();
    assert!(matches!(owner_actor, ObservedActor::Visible(actor) if actor.live_bomb_count == 1));

    let blind = observe_cpu_recipient(&publication, Seat::One, world).unwrap();
    assert!(matches!(blind.parts().bombs, BombKnowledge::Unavailable));
}

#[test]
fn binding_rejects_ordinary_count_and_duplicate_cell_tampering() {
    let case = dense_mine_case(false);
    let mut wrong_count = case.capture.clone();
    wrong_count.base.world.actors[1].view.live_bomb_count = 0;
    assert!(matches!(
        bind_expanded(
            &case.room,
            case.room.round_id,
            &wrong_count,
            case.generation
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    ));

    let mut duplicate_cell = case.capture.clone();
    duplicate_cell.base.world.bombs[1].cell = duplicate_cell.base.world.bombs[0].cell;
    assert!(matches!(
        bind_expanded(
            &case.room,
            case.room.round_id,
            &duplicate_cell,
            case.generation
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    ));
}

#[test]
fn old_capture_stays_closed_and_rules_nine_event_bounds_are_checked() {
    let case = dense_mine_case(false);
    assert!(matches!(
        crate::protocol::v16::capture_expanded(&case.state, &[]),
        Err(ProjectionError::InvalidRules)
    ));

    assert!(EventIndex::new(192).is_err());
    assert_eq!(EventIndex::new_for_rules(215, 9).unwrap().get(), 215);
    assert!(EventIndex::new_for_rules(216, 9).is_err());
    let event = WireEvent {
        tick: crate::protocol::Tick::new(1).unwrap(),
        index: EventIndex::new_for_rules(200, 9).unwrap(),
        kind: EventKind::ActorEliminated {
            id: crate::protocol::EntityId::new(20),
            seat: Seat::Two,
        },
    };
    assert!(SnapshotEvents::new(vec![event.clone(); 193]).is_err());
    assert!(SnapshotEvents::new_for_rules(vec![event.clone(); 256], 9).is_ok());
    assert!(SnapshotEvents::new_for_rules(vec![event; 257], 9).is_err());
    assert_eq!(
        crate::protocol::v14::projection_limits::report_events(9),
        216
    );
}
