use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::{CpuRng, MemoryKey, MovementIntent, Observation, WorkBudget, WorldId};
use crate::game::{
    ActorInit, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, State, World,
};
use crate::protocol::v11::{
    ConnectionStatus, GameSettings, ItemMask, ItemSet, MapId, RoomPhase, RoomView, RoundCount,
    RulesVersion, Scores, SeatStatus, Wins,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::{UtilityMemory, plan};

const CPU_START: CellId = CellId::new(13, 11);
const WORLD: WorldId = WorldId {
    room_instance: 85,
    physical_round: 1,
};

fn state_with_pickups(
    enabled: EnabledItems,
    pickups: impl IntoIterator<Item = (CellId, PickupKind)>,
) -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 3, 180)
        .unwrap()
        .with_enabled_items(enabled);
    let arena = crate::game::test_support::open_border_arena(BTreeSet::new());
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(arena.spawns()[seat.index()]),
        heading: Direction::West,
        speed: settings.starting_speed(),
    });
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        pickups.into_iter().collect(),
    )
    .unwrap();
    (
        State::new(RulesConfig::expanded_v5(settings), world, actors).unwrap(),
        settings,
    )
}

fn item_set(items: EnabledItems) -> ItemSet {
    ItemSet::from_mask(ItemMask::new(items.bits()).unwrap())
}

fn observation(state: &State, settings: RoomSettings) -> Observation {
    let capture = crate::protocol::v11::capture_expanded(state, &[]).unwrap();
    let room = RoomView {
        settings: GameSettings::new(
            0,
            settings.turn_assist_units(),
            settings.wins_to_finish(),
            u16::try_from(settings.bomb_fuse_ticks()).unwrap(),
            item_set(settings.enabled_items()),
        )
        .unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: item_set(EnabledItems::ALL),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: capture.world.0.physical_tick,
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: true,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Connected,
        }),
        phase: RoomPhase::Running,
    };
    let publication = crate::protocol::v11::bind_expanded(
        &room,
        room.round_id,
        &capture,
        Some(crate::game::generate_expanded_arena([31; 32], settings.enabled_items()).record()),
    )
    .unwrap();
    crate::protocol::v11::observe_cpu_recipient(&publication, Seat::Two, WORLD).unwrap()
}

fn execute_plan(state: &mut State, movement: MovementIntent, hold_ticks: u8) {
    if let MovementIntent::Direction(direction) = movement {
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(direction),
            }])
            .unwrap();
        for _ in 1..hold_ticks {
            state.step(&[]).unwrap();
        }
    }
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Clear,
        }])
        .unwrap();
}

fn drive_until_collected(
    state: &mut State,
    settings: RoomSettings,
    collected: PickupKind,
) -> Vec<CellId> {
    let key = MemoryKey {
        world: WORLD,
        cpu_seat: SeatId::Two,
    };
    let mut memory = UtilityMemory::new(key);
    let mut rng = CpuRng::from_seed(key, [85; 32]);
    let mut visited = vec![CPU_START];
    for _ in 0..80 {
        let observed = observation(state, settings);
        let mut work = WorkBudget::new(1_000);
        let choice = plan(&observed, &memory, &mut rng, &mut work).unwrap();
        assert_eq!(choice.intent.manual, None);
        memory = choice.next_memory;
        execute_plan(state, choice.intent.movement, choice.hold_ticks);
        let cell = state.actors()[SeatId::Two.index()]
            .position()
            .cell()
            .unwrap();
        if visited.last() != Some(&cell) {
            visited.push(cell);
        }
        if !state
            .world()
            .exposed_pickups()
            .values()
            .any(|pickup| pickup.kind() == collected)
        {
            return visited;
        }
    }
    panic!("CPU did not collect {collected:?} within the bounded pulse budget");
}

#[test]
fn real_state_detours_around_nearer_curse_to_collect_useful_upgrade() {
    let curse = CellId::new(12, 11);
    let useful = CellId::new(11, 11);
    let (mut state, settings) = state_with_pickups(
        EnabledItems::SLOW_FOOT,
        [
            (curse, PickupKind::SlowFoot),
            (useful, PickupKind::BombCapacity),
        ],
    );

    let visited = drive_until_collected(&mut state, settings, PickupKind::BombCapacity);

    assert_eq!(state.actors()[SeatId::Two.index()].bomb_capacity(), 2);
    assert_eq!(
        state.world().exposed_pickups().get(&curse).unwrap().kind(),
        PickupKind::SlowFoot
    );
    assert!(
        !visited.contains(&curse),
        "curse was used as transit: {visited:?}"
    );
    assert!(visited.iter().any(|cell| cell.row < CPU_START.row));
    assert_eq!(
        visited.iter().copied().collect::<BTreeSet<_>>().len(),
        visited.len()
    );
}

#[test]
fn real_state_ignores_nearer_unsupported_weapon_for_reachable_speed() {
    let weapon = CellId::new(12, 11);
    let useful = CellId::new(13, 9);
    let (mut state, settings) = state_with_pickups(
        EnabledItems::REMOTE_BOMB,
        [
            (weapon, PickupKind::RemoteBomb),
            (useful, PickupKind::MovementSpeed),
        ],
    );

    let visited = drive_until_collected(&mut state, settings, PickupKind::MovementSpeed);

    assert_eq!(state.actors()[SeatId::Two.index()].speed(), SpeedLevel::One);
    assert_eq!(
        state.world().exposed_pickups().get(&weapon).unwrap().kind(),
        PickupKind::RemoteBomb
    );
    assert!(!visited.contains(&weapon));
}
