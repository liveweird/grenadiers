use std::collections::BTreeSet;

use crate::game::{
    ActorInit, ArenaGenerationConfig, Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId,
    PlayerCount, Position, RoundAssignment, RulesConfig, SeatId, SpeedLevel, StormCloudConfig,
    Tick as GameTick, World, generate_multiplayer_arena, spike_pit_layout_v1,
};
use crate::protocol::{Counter, RoomCode, RoomId, Tick};
use crate::room::{
    BoardSize as DomainBoard, LairRoomConfiguration, LairTuning, RespawnLairConfig, SeatControl,
    SpikePitConfig, SpikePitSettings, SpikePitTuning, SpikeRoomConfiguration,
};

use super::*;

fn room_view(tick: u64) -> RoomView {
    let base = crate::room::RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Human],
        None,
        DomainBoard::FifteenByThirteen,
    )
    .unwrap();
    let lairs = LairTuning::new(false, crate::room::LairSettings::new(10, 1).unwrap());
    let spike_pits = SpikePitTuning::new(true, SpikePitSettings::DEFAULT);
    let environment = Environment {
        storm_cloud: false,
        lairs: Lairs {
            enabled: false,
            spawn_seconds: lairs.settings().spawn_seconds(),
            max_living_mobs: 1,
        },
        spike_pits: SpikePits {
            enabled: true,
            warning_ticks: SpikePitSettings::DEFAULT.warning_ticks() as u16,
            raised_ticks: SpikePitSettings::DEFAULT.raised_ticks() as u16,
        },
    };
    RoomView {
        settings: RoomSettings::new(
            GameSettings::new(
                0,
                256,
                3,
                180,
                ItemSet::from_mask(ItemMask::new(0).unwrap()),
            )
            .unwrap(),
            environment,
        ),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: ItemSet::from_mask(ItemMask::new(EnabledItems::ALL.bits()).unwrap()),
        configuration: RoomConfiguration::from_domain(SpikeRoomConfiguration::new(
            LairRoomConfiguration::new(base, StormCloudConfig::Disabled, lairs),
            spike_pits,
        ))
        .unwrap(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(tick).unwrap(),
        rules_version: RulesVersion::ARENA_V10,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: [Seat::One, Seat::Two]
            .map(|seat| Score {
                seat,
                wins: Wins::new(0).unwrap(),
            })
            .to_vec(),
        last_round: None,
        seats: [Seat::One, Seat::Two]
            .map(|seat| SeatStatus {
                seat,
                occupied: true,
                ready: false,
                rematch_ready: false,
                controller: Controller::Human,
                connection: ConnectionStatus::Connected,
            })
            .to_vec(),
        phase: RoomPhase::Running,
    }
}

fn mutable_layout_world() -> (World, CellId, CellId, Vec<CellId>) {
    let board = DomainBoard::FifteenByThirteen;
    let preferred = [
        CellId::new(6, 5),
        CellId::new(8, 5),
        CellId::new(6, 7),
        CellId::new(8, 7),
    ];
    let initial = [
        CellId::new(3, 4),
        CellId::new(11, 4),
        CellId::new(3, 8),
        CellId::new(11, 8),
    ];
    let destroyed = preferred[0];
    let bomb_cell = CellId::new(5, 5);
    let potential_spawns = [
        CellId::new(1, 1),
        CellId::new(13, 1),
        CellId::new(1, 11),
        CellId::new(13, 11),
    ];
    let mut terrain = vec![crate::game::Terrain::Floor; board.cell_count()];
    for row in 0..board.height() {
        for column in 0..board.width() {
            if row == 0 || row + 1 == board.height() || column == 0 || column + 1 == board.width() {
                terrain[usize::from(row * board.width() + column)] = crate::game::Terrain::Wall;
            }
        }
    }
    let clear = initial
        .into_iter()
        .chain(preferred.into_iter().filter(|cell| *cell != destroyed))
        .chain([bomb_cell])
        .collect::<BTreeSet<_>>();
    let blocks = (1..board.height() - 1)
        .flat_map(|row| (1..board.width() - 1).map(move |column| CellId::new(column, row)))
        .filter(|cell| {
            !clear.contains(cell)
                && potential_spawns.iter().all(|spawn| {
                    spawn.column.abs_diff(cell.column) + spawn.row.abs_diff(cell.row) > 1
                })
        })
        .collect();
    let arena = crate::game::Arena::new_for_board(
        board,
        terrain,
        blocks,
        [potential_spawns[0], potential_spawns[3]],
    )
    .unwrap();
    assert_eq!(spike_pit_layout_v1(&arena, []), initial);
    (World::new(arena), destroyed, bomb_cell, initial.to_vec())
}

#[test]
fn binding_keeps_construction_time_pits_after_blocks_open_a_preferred_orbit() {
    let generated = generate_multiplayer_arena(
        [73; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: DomainBoard::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap();
    let generation = generated.record();
    let (mut world, destroyed, bomb_cell, initial_layout) = mutable_layout_world();
    let spawns = world.arena().spawns().to_vec();
    let bomb = Bomb::new(
        EntityId(90),
        bomb_cell,
        EntityId(20),
        1,
        GameTick(0),
        BombTrigger::Timed {
            detonates_at: GameTick(1),
        },
    )
    .unwrap();
    world.bombs.insert(bomb.id(), bomb);
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32 * 10),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v10(
        crate::room::RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
    );
    let mut state = crate::game::State::new(rules, world, actors).unwrap();
    state.step(&[]).unwrap();
    assert!(!state.world().arena().blocks().contains(&destroyed));
    assert_ne!(
        spike_pit_layout_v1(state.world().arena(), []),
        initial_layout
    );

    let capture = capture_expanded(&state, &[]).unwrap();
    let captured_cells = capture
        .spike_pits
        .iter()
        .map(|pit| CellId::new(pit.cell.column().into(), pit.cell.row().into()))
        .collect::<Vec<_>>();
    assert_eq!(captured_cells, initial_layout);
    assert!(super::projection_validation::valid_spike_pits(
        &room_view(1),
        &capture,
    ));
    let mut omitted = capture.clone();
    omitted.spike_pits.pop();
    assert!(!super::projection_validation::valid_spike_pits(
        &room_view(1),
        &omitted,
    ));
    let mut empty = capture.clone();
    empty.spike_pits.clear();
    assert!(super::projection_validation::valid_spike_pits(
        &room_view(1),
        &empty,
    ));
    let mut hidden_oil = capture.clone();
    hidden_oil
        .base
        .world
        .oil
        .push(crate::protocol::v14::projection_facts::OilFacts {
            cell: capture.spike_pits[0].cell,
            phase: crate::protocol::v14::projection_facts::OilPhase::Concealed,
        });
    assert!(super::projection_validation::valid_spike_pits(
        &room_view(1),
        &hidden_oil,
    ));
    hidden_oil.base.world.oil[0].phase = crate::protocol::v14::projection_facts::OilPhase::Unlit;
    assert!(!super::projection_validation::valid_spike_pits(
        &room_view(1),
        &hidden_oil,
    ));
    assert!(
        bind_expanded(
            &room_view(1),
            Counter::new(1).unwrap(),
            &capture,
            generation
        )
        .is_ok()
    );
}
