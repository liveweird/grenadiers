use super::*;
use crate::room::{CpuPolicy, CpuProfile};

#[test]
fn accepts_every_valid_two_to_four_player_lineup() {
    for player_count in 2..=4 {
        for cpu_mask in 0..(1 << (player_count - 1)) {
            let mut controls = vec![SeatControl::Human];
            controls.extend((0..player_count - 1).map(|index| {
                if cpu_mask & (1 << index) == 0 {
                    SeatControl::Human
                } else {
                    SeatControl::Cpu
                }
            }));
            let difficulty = (cpu_mask != 0).then_some(CpuDifficulty::Standard);
            let configuration =
                RoomConfiguration::new(&controls, difficulty, BoardSize::ThirteenByEleven).unwrap();
            assert_eq!(configuration.player_count().get(), player_count);
            assert_eq!(configuration.controllers(), controls);
            assert_eq!(configuration.board(), BoardSize::ThirteenByEleven);
        }
    }
}

#[test]
fn rejects_invalid_player_counts_and_nonhuman_creator() {
    assert_eq!(
        RoomConfiguration::new(&[SeatControl::Human], None, BoardSize::default()),
        Err(RoomConfigurationError::InvalidPlayerCount { actual: 1 })
    );
    assert_eq!(
        RoomConfiguration::new(&[SeatControl::Human; 5], None, BoardSize::default()),
        Err(RoomConfigurationError::InvalidPlayerCount { actual: 5 })
    );
    assert_eq!(
        RoomConfiguration::new(
            &[SeatControl::Cpu, SeatControl::Cpu],
            Some(CpuDifficulty::Easy),
            BoardSize::default(),
        ),
        Err(RoomConfigurationError::CreatorMustBeHuman)
    );
}

#[test]
fn cpu_difficulty_presence_matches_cpu_seats() {
    assert_eq!(
        RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            None,
            BoardSize::default(),
        ),
        Err(RoomConfigurationError::MissingCpuDifficulty)
    );
    assert_eq!(
        RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Human],
            Some(CpuDifficulty::Easy),
            BoardSize::default(),
        ),
        Err(RoomConfigurationError::UnexpectedCpuDifficulty)
    );
}

#[test]
fn exposes_only_actual_seats_in_canonical_order() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Easy),
        BoardSize::default(),
    )
    .unwrap();
    assert_eq!(configuration.controller(SeatId::Four), None);
    assert_eq!(
        configuration.human_seats().collect::<Vec<_>>(),
        vec![SeatId::One, SeatId::Three]
    );
    assert_eq!(
        configuration.cpu_seats().collect::<Vec<_>>(),
        vec![SeatId::Two]
    );
}

#[test]
fn unused_capacity_is_canonical_for_value_equality() {
    let constructed = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::default(),
    )
    .unwrap();
    assert_eq!(
        constructed,
        RoomConfiguration::solo(CpuDifficulty::Standard)
    );
}

#[test]
fn board_sizes_are_a_closed_set_of_dimension_pairs() {
    for (board, dimensions) in [
        (BoardSize::ElevenByNine, (11, 9)),
        (BoardSize::ThirteenByEleven, (13, 11)),
        (BoardSize::FifteenByThirteen, (15, 13)),
    ] {
        assert_eq!(board.dimensions(), dimensions);
        assert_eq!(BoardSize::try_from(dimensions), Ok(board));
    }
    assert_eq!(BoardSize::default(), BoardSize::FifteenByThirteen);
    assert!(BoardSize::try_from((11, 11)).is_err());
    assert!(BoardSize::try_from((15, 9)).is_err());
}

#[test]
fn legacy_modes_bridge_to_default_board_and_profile_difficulty_only() {
    assert_eq!(
        RoomConfiguration::from_legacy(RoomMode::HumanDuel),
        RoomConfiguration::human_duel()
    );
    let legacy = RoomMode::Solo(CpuProfile {
        policy: CpuPolicy::UtilityV1,
        difficulty: CpuDifficulty::Easy,
    });
    assert_eq!(
        RoomConfiguration::from_legacy(legacy),
        RoomConfiguration::solo(CpuDifficulty::Easy)
    );
}
