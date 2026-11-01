use super::*;
use crate::protocol::Cell;

fn create_room(spike_pits: &str) -> String {
    r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":false,"lairs":{"enabled":false,"spawnSeconds":10,"maxLivingMobs":1},"spikePits":$SPIKE_PITS}}}"#
        .replace("$SPIKE_PITS", spike_pits)
}

#[test]
fn parses_strict_bounded_spike_pit_configuration_and_retains_disabled_values() {
    let message = parse_client_message(&create_room(
        r#"{"enabled":false,"warningTicks":12,"raisedTicks":240}"#,
    ))
    .unwrap();
    let GameClientMessage::CreateRoom { configuration, .. } = message else {
        panic!("expected create-room message")
    };
    let tuning = configuration.domain().spike_pits();
    assert!(!tuning.enabled());
    assert_eq!(tuning.settings().warning_ticks(), 12);
    assert_eq!(tuning.settings().raised_ticks(), 240);

    assert!(
        parse_client_message(&create_room(
            r#"{"enabled":true,"warningTicks":5,"raisedTicks":180}"#,
        ))
        .is_err()
    );
    assert!(
        parse_client_message(&create_room(
            r#"{"enabled":true,"warningTicks":36,"raisedTicks":601}"#,
        ))
        .is_err()
    );
    assert!(
        parse_client_message(&create_room(
            r#"{"enabled":true,"warningTicks":36,"raisedTicks":180,"seed":1}"#,
        ))
        .is_err()
    );
}

#[test]
fn serializes_v19_and_rules10_as_exact_identities() {
    let authenticated = GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    };
    assert_eq!(
        encode_server_message(&authenticated).unwrap(),
        r#"{"type":"authenticated","protocolVersion":19}"#,
    );
    assert_eq!(RulesVersion::new(10), Ok(RulesVersion::ARENA_V10));
    assert!(RulesVersion::new(9).is_err());
}

#[test]
fn spike_state_wire_phases_are_closed_and_idle_has_zero_remaining() {
    let idle = SpikePitView {
        cell: Cell::new(3, 3).unwrap(),
        phase: SpikePitPhase::Idle,
        remaining_ticks: 0,
    };
    assert_eq!(
        serde_json::to_string(&idle).unwrap(),
        r#"{"cell":{"column":3,"row":3},"phase":"idle","remainingTicks":0}"#,
    );
}

#[test]
fn captures_rules10_public_spike_state_without_private_activation_facts() {
    use crate::game::{
        ActorInit, ArenaGenerationConfig, EnabledItems, PlayerCount, Position, RoundAssignment,
        RulesConfig, SeatId, SpeedLevel, StormCloudConfig, World, generate_multiplayer_arena,
    };
    use crate::room::{BoardSize, RespawnLairConfig, SpikePitConfig, SpikePitSettings};

    let generated = generate_multiplayer_arena(
        [18; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: BoardSize::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap();
    let (world, _) = World::from_generated(generated).unwrap();
    let spawns = world.arena().spawns();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: crate::game::EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: crate::game::Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v10(
        crate::room::RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
    );
    let state = crate::game::State::new(rules, world, actors).unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();
    assert_eq!(capture.base.world.rules_version, 10);
    assert!(capture.spike_pits.len() <= crate::game::MAX_SPIKE_PITS);
    assert!(
        capture
            .spike_pits
            .iter()
            .all(|pit| { pit.phase == SpikePitPhase::Idle && pit.remaining_ticks == 0 })
    );
    assert_eq!(
        crate::protocol::v14::projection_limits::bombs(&capture.base.world),
        195
    );
    assert_eq!(
        crate::protocol::v14::projection_limits::report_events(10),
        216
    );
    assert_eq!(
        crate::protocol::v14::projection_limits::snapshot_events(10),
        256
    );
    assert_eq!(
        crate::protocol::v14::EventIndex::new_for_rules(215, 10)
            .unwrap()
            .get(),
        215
    );
    assert!(crate::protocol::v14::EventIndex::new_for_rules(216, 10).is_err());
    assert_eq!(
        crate::protocol::v14::projection_limits::report_events(8),
        49
    );
    assert_eq!(
        crate::protocol::v14::projection_limits::report_events(11),
        49
    );
}

#[test]
fn rules10_dense_mines_keep_contact_bombs_out_of_live_capacity() {
    use crate::game::{
        ActorInit, ArenaGenerationConfig, Bomb, BombTrigger, CellId, Direction, EnabledItems,
        EntityId, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId, SpeedLevel,
        StormCloudConfig, Tick, World, generate_multiplayer_arena,
    };
    use crate::room::{BoardSize, RespawnLairConfig, SpikePitConfig};

    let enabled = EnabledItems::from_bits(EnabledItems::LANDMINE.bits()).unwrap();
    let generated = generate_multiplayer_arena(
        [91; 32],
        enabled,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: BoardSize::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap();
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
                && !occupied.contains(cell)
        })
        .take(34)
        .collect::<Vec<_>>();
    let owner = EntityId(20);
    let ordinary = Bomb::new(
        EntityId(100),
        cells[0],
        owner,
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(180),
        },
    )
    .unwrap();
    world.bombs.insert(ordinary.id(), ordinary);
    for (offset, &cell) in cells[1..].iter().enumerate() {
        let id = EntityId(101 + offset as u32);
        world.bombs.insert(
            id,
            Bomb::new(id, cell, owner, 2, Tick(0), BombTrigger::Contact).unwrap(),
        );
    }
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32 * 10),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v10(
        crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        SpikePitConfig::Disabled,
    );
    let state = crate::game::State::new(rules, world, actors).unwrap();
    assert_eq!(state.live_bomb_count(SeatId::Two), 1);
    let capture = capture_expanded(&state, &[]).unwrap();
    assert_eq!(capture.base.world.bombs.len(), 34);
    assert!(
        crate::protocol::v14::projection_validation::validate(
            &capture.base.world,
            &capture.base.reports,
            &capture.base.rules,
        )
        .is_ok()
    );
    assert!(capture.spike_pits.is_empty());
}
