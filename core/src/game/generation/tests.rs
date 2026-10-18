use std::collections::VecDeque;

use super::*;
use crate::game::{
    ActorInit, Direction, EntityId, InputAction, Position, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, World,
};

struct Sequence {
    words: VecDeque<u32>,
    default: Option<u32>,
    draws: usize,
}

impl Sequence {
    fn new(words: impl IntoIterator<Item = u32>, default: Option<u32>) -> Self {
        Self {
            words: words.into_iter().collect(),
            default,
            draws: 0,
        }
    }
}

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.draws += 1;
        self.words.pop_front().or(self.default)
    }
}

#[test]
fn seeded_generation_is_reproducible_and_records_redacted_replay_metadata() {
    let seed = [37; 32];
    let first = generate_arena(seed);
    let second = generate_arena(seed);

    assert_eq!(first, second);
    assert_eq!(first.record().version(), GENERATOR_VERSION);
    assert_eq!(first.record().seed(), seed);
    assert!(!first.record().fallback_used());
    let debug = format!("{first:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("37, 37"));
    assert!(!debug.contains("concealed_pickups"));
}

#[test]
fn seeded_outputs_match_version_one_goldens() {
    assert_eq!(
        signature(&generate_arena([0; 32])),
        "..##.##C.#.#./.X#X#X#X#X.X./.#.#.##R#.R##/RX#XCX#X#X.X#/####.##C..#SR/CXCX.X.X.XCXC/RS#..C##.####/#X.X#X#XCX#XR/##R.#R##.#.#./.X.X#X#X#X#X./.#.#.C##.##.."
    );
    assert_eq!(
        signature(&generate_arena([1; 32])),
        "..#..#.#.#.CS/.X#XSX#X.X#X#/S##S.#.##..#R/.X.X.X#X#X.X./.######.#..../.XSX.X#X.XSX./....#.######./.X.X#X#X.X.X./R#..##.#.S##S/#X#X.X#XSX#X./SC.#.#.#..#.."
    );
}

#[test]
fn thresholds_and_draw_order_are_exact() {
    let words = [
        64, 29, 39, // occupied, present, capacity
        64, 29, 40, // occupied, present, range
        64, 29, 79, // occupied, present, range
        64, 29, 80, // occupied, present, speed
        65, // unoccupied, with no further draws for the pair
    ];
    let mut source = Sequence::new(words, Some(99));
    let generated = generate_with_source([0; 32], &mut source, classic_terrain());
    let concealed = generated.concealed_pickups();

    assert_eq!(
        concealed.get(&CellId::new(3, 1)),
        Some(&PickupKind::BombCapacity)
    );
    assert_eq!(
        concealed.get(&CellId::new(4, 1)),
        Some(&PickupKind::BlastRange)
    );
    assert_eq!(
        concealed.get(&CellId::new(5, 1)),
        Some(&PickupKind::BlastRange)
    );
    assert_eq!(
        concealed.get(&CellId::new(6, 1)),
        Some(&PickupKind::MovementSpeed)
    );
    assert!(!generated.arena().blocks().contains(&CellId::new(7, 1)));
    assert_eq!(source.draws, 62);
}

#[test]
fn the_unpaired_center_is_processed_once_and_last() {
    let pairs = eligible_pairs();
    assert_eq!(pairs.len(), 54);
    assert_eq!(pairs.last(), Some(&(CellId::new(7, 6), CellId::new(7, 6))));

    let mut words = vec![99; 53];
    words.extend([64, 29, 39]);
    let mut source = Sequence::new(words, None);
    let generated = generate_with_source([0; 32], &mut source, classic_terrain());

    assert_eq!(generated.arena().blocks().len(), 1);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(7, 6)),
        Some(&PickupKind::BombCapacity)
    );
    assert_eq!(source.draws, 56);
}

#[test]
fn rejection_sampling_accepts_on_the_sixteenth_draw() {
    let mut words = vec![u32::MAX; 15];
    words.push(42);
    let mut source = Sequence::new(words, None);

    assert_eq!(sample_percent(&mut source), Some(42));
    assert_eq!(source.draws, 16);
}

#[test]
fn rejection_exhaustion_uses_the_empty_symmetric_fallback() {
    let mut source = Sequence::new([u32::MAX; 16], None);
    let generated = generate_with_source([9; 32], &mut source, classic_terrain());

    assert!(generated.record().fallback_used());
    assert!(generated.arena().blocks().is_empty());
    assert!(generated.concealed_pickups().is_empty());
    assert_eq!(source.draws, 16);
}

#[test]
fn invalid_generated_terrain_uses_the_canonical_empty_fallback() {
    let mut invalid = classic_terrain();
    invalid[index(CellId::new(7, 0))] = Terrain::Floor;
    let mut source = Sequence::new([], Some(99));
    let generated = generate_with_source([3; 32], &mut source, invalid);

    assert!(generated.record().fallback_used());
    assert_eq!(
        generated.arena().terrain(CellId::new(7, 0)),
        Some(Terrain::Wall)
    );
    assert!(generated.arena().blocks().is_empty());
}

#[test]
fn generated_layers_are_symmetric_reserved_and_concealed_only_under_blocks() {
    let generated = generate_arena([7; 32]);
    let arena = generated.arena();

    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            let cell = CellId::new(column, row);
            let mirror = rotate(cell);
            assert_eq!(arena.terrain(cell), arena.terrain(mirror));
            assert_eq!(
                arena.blocks().contains(&cell),
                arena.blocks().contains(&mirror)
            );
            assert_eq!(
                generated.concealed_pickups().get(&cell),
                generated.concealed_pickups().get(&mirror)
            );
        }
    }
    for cell in reserved_cells() {
        assert_eq!(arena.terrain(cell), Some(Terrain::Floor));
        assert!(!arena.blocks().contains(&cell));
        assert!(!generated.concealed_pickups().contains_key(&cell));
    }
    assert!(
        generated
            .concealed_pickups()
            .keys()
            .all(|cell| arena.blocks().contains(cell))
    );
}

#[test]
fn empty_expanded_generation_matches_v1_except_for_metadata() {
    for seed in [[0; 32], [1; 32], [37; 32]] {
        let legacy = generate_arena(seed);
        let expanded = generate_expanded_arena(seed, EnabledItems::EMPTY);
        assert_eq!(legacy.arena(), expanded.arena());
        assert_eq!(legacy.concealed_pickups(), expanded.concealed_pickups());
        assert_eq!(expanded.record().version(), EXPANDED_GENERATOR_VERSION);
        assert_eq!(
            legacy.record().fallback_used(),
            expanded.record().fallback_used()
        );
    }
}

#[test]
fn seeded_outputs_match_version_two_heart_goldens() {
    assert_eq!(
        signature(&generate_expanded_arena([0; 32], EnabledItems::HEART)),
        "..##.##C.#.#./.X#X#X#X#X.X./.#.#.##S#.H##/CX#XRX#X#X.X#/####.##R..#CR/RXCX.X.X.XCXR/RC#..R##.####/#X.X#X#XRX#XC/##H.#S##.#.#./.X.X#X#X#X#X./.#.#.C##.##.."
    );
    assert_eq!(
        signature(&generate_expanded_arena([1; 32], EnabledItems::HEART)),
        "..#..#.#.#.SH/.X#XCX#X.X#X#/H##S.#.##..#R/.X.X.X#X#X.X./.######.#..../.XCX.X#X.XCX./....#.######./.X.X#X#X.X.X./R#..##.#.S##H/#X#X.X#XCX#X./HS.#.#.#..#.."
    );
}

#[test]
fn enabled_heart_has_fixed_weight_boundaries_and_symmetric_placement() {
    let words = [64, 29, 99, 64, 29, 100, 64, 29, 124];
    let mut source = Sequence::new(words, Some(99));
    let generated =
        generate_expanded_with_source([4; 32], &mut source, classic_terrain(), EnabledItems::HEART);

    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::MovementSpeed)
    );
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(4, 1)),
        Some(&PickupKind::Heart)
    );
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(5, 1)),
        Some(&PickupKind::Heart)
    );
}

#[test]
fn dense_generated_layout_survives_first_bomb_via_the_reserved_escape_route() {
    let mut source = Sequence::new([], Some(0));
    let generated = generate_with_source([0; 32], &mut source, classic_terrain());
    assert!(!generated.record().fallback_used());
    let mut state = State::new(
        RulesConfig::classic_v1(),
        World::new(generated.into_parts().0),
        [
            actor(1, SeatId::One, CellId::new(1, 1), Direction::East),
            actor(2, SeatId::Two, CellId::new(13, 11), Direction::West),
        ],
    )
    .unwrap();

    step_with(&mut state, SeatId::One, InputAction::Press(Direction::East));
    step_empty(&mut state, 19);
    assert_eq!(
        state.actors()[0].position(),
        Position::center(CellId::new(2, 1))
    );
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::West),
            },
        ])
        .unwrap();
    step_empty(&mut state, 19);
    assert_eq!(
        state.actors()[0].position(),
        Position::center(CellId::new(1, 1))
    );
    step_with(
        &mut state,
        SeatId::One,
        InputAction::Press(Direction::South),
    );
    step_empty(&mut state, 19);
    step_with(&mut state, SeatId::One, InputAction::Clear);
    assert_eq!(
        state.actors()[0].position(),
        Position::center(CellId::new(1, 2))
    );

    while !state.world().bombs().is_empty() {
        state.step(&[]).unwrap();
    }
    assert!(state.actors()[0].alive());
    assert!(state.world().hazards().contains_key(&CellId::new(2, 1)));
    assert!(!state.world().hazards().contains_key(&CellId::new(1, 2)));
}

fn actor(id: u32, seat: SeatId, cell: CellId, heading: Direction) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading,
        speed: SpeedLevel::Base,
    }
}

fn step_with(state: &mut State, seat: SeatId, action: InputAction) {
    state.step(&[SeatInput { seat, action }]).unwrap();
}

fn step_empty(state: &mut State, count: usize) {
    for _ in 0..count {
        state.step(&[]).unwrap();
    }
}

fn signature(generated: &GeneratedArena) -> String {
    let mut rows = Vec::new();
    for row in 1..ARENA_HEIGHT - 1 {
        let mut encoded = String::new();
        for column in 1..ARENA_WIDTH - 1 {
            let cell = CellId::new(column, row);
            encoded.push(match generated.concealed_pickups().get(&cell) {
                Some(PickupKind::BombCapacity) => 'C',
                Some(PickupKind::BlastRange) => 'R',
                Some(PickupKind::MovementSpeed) => 'S',
                Some(PickupKind::Heart) => 'H',
                Some(PickupKind::Shield) => 'D',
                Some(PickupKind::GoldenFlame) => 'G',
                Some(PickupKind::ShortFuse) => 'F',
                Some(PickupKind::ShortFlame) => 'L',
                Some(PickupKind::LongFuse) => 'U',
                Some(PickupKind::ReverseControls) => 'V',
                None if generated.arena().blocks().contains(&cell) => '#',
                None if generated.arena().terrain(cell) == Some(Terrain::Wall) => 'X',
                None => '.',
            });
        }
        rows.push(encoded);
    }
    rows.join("/")
}
