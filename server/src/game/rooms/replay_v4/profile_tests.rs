use grenadiers_core::{
    game::SeatId,
    room::{BoardSize, CpuDifficulty, RoomConfiguration, RoomSettings, SeatControl},
};

use super::{
    CpuProvenance, Journal, ReplayError, ReplayV4Header, ReplayV5Header, replay, replay_v5,
};

fn configuration(board: BoardSize, controls: &[SeatControl]) -> RoomConfiguration {
    RoomConfiguration::new(
        controls,
        controls
            .contains(&SeatControl::Cpu)
            .then_some(CpuDifficulty::Standard),
        board,
    )
    .unwrap()
}

fn provenance(configuration: RoomConfiguration) -> Vec<CpuProvenance> {
    configuration
        .cpu_seats()
        .map(|seat| CpuProvenance::new(seat, CpuDifficulty::Standard, [seat.index() as u8; 32]))
        .collect()
}

#[test]
fn constructors_bind_each_format_to_its_exact_generator_and_board_profile() {
    let large = configuration(
        BoardSize::FifteenByThirteen,
        &[SeatControl::Human, SeatControl::Cpu],
    );
    let v4 =
        ReplayV4Header::new(large, RoomSettings::DEFAULT, [false; 4], provenance(large)).unwrap();
    assert_eq!(v4.format_version, 4);
    assert_eq!(v4.generator_version, 3);
    assert_eq!(v4.simulation_randomness_version, 1);
    assert_eq!(v4.simulation_seed_derivation_version, 1);

    for board in [BoardSize::ElevenByNine, BoardSize::ThirteenByEleven] {
        let small = configuration(board, &[SeatControl::Human, SeatControl::Human]);
        let v5 =
            ReplayV5Header::new_v5(small, RoomSettings::DEFAULT, [false; 4], Vec::new()).unwrap();
        assert_eq!(v5.format_version, 5);
        assert_eq!(v5.generator_version, 4);
        assert_eq!(v5.simulation_randomness_version, 1);
        assert_eq!(v5.simulation_seed_derivation_version, 1);
        assert!(super::validation::validate_header_v5(&v5).is_ok());
        assert!(ReplayV4Header::new(small, RoomSettings::DEFAULT, [false; 4], vec![]).is_err());
    }
    assert!(
        ReplayV5Header::new_v5(large, RoomSettings::DEFAULT, [false; 4], provenance(large))
            .is_err()
    );
}

#[test]
fn cross_profile_and_version_mutations_are_rejected_atomically() {
    let large = configuration(
        BoardSize::FifteenByThirteen,
        &[SeatControl::Human, SeatControl::Cpu],
    );
    let v4 =
        ReplayV4Header::new(large, RoomSettings::DEFAULT, [false; 4], provenance(large)).unwrap();
    let small = configuration(
        BoardSize::ElevenByNine,
        &[SeatControl::Human, SeatControl::Cpu],
    );
    let v5 = ReplayV5Header::new_v5(small, RoomSettings::DEFAULT, [false; 4], provenance(small))
        .unwrap();

    let mut v5_as_v4 = v5.clone();
    v5_as_v4.format_version = 4;
    assert!(super::validation::validate_header(&v5_as_v4).is_err());
    assert!(super::validation::validate_header_v5(&v5_as_v4).is_err());
    let mut v4_as_v5 = v4.clone();
    v4_as_v5.format_version = 5;
    assert!(super::validation::validate_header(&v4_as_v5).is_err());
    assert!(super::validation::validate_header_v5(&v4_as_v5).is_err());

    for invalid in [0, 3, 5] {
        let mut wrong_generator = v5.clone();
        wrong_generator.generator_version = invalid;
        assert!(super::validation::validate_header_v5(&wrong_generator).is_err());
    }
    let mut wrong_rules = v5.clone();
    wrong_rules.rules_version = 5;
    assert!(super::validation::validate_header_v5(&wrong_rules).is_err());
    let mut wrong_board = v5.clone();
    wrong_board.configuration = large;
    assert!(super::validation::validate_header_v5(&wrong_board).is_err());
    for (mut header, validate) in [
        (
            v4.clone(),
            super::validation::validate_header as fn(&ReplayV4Header) -> Result<(), ()>,
        ),
        (v5.clone(), super::validation::validate_header_v5),
    ] {
        header.simulation_randomness_version += 1;
        assert!(validate(&header).is_err());
    }
    for (mut header, validate) in [
        (
            v4.clone(),
            super::validation::validate_header as fn(&ReplayV4Header) -> Result<(), ()>,
        ),
        (v5.clone(), super::validation::validate_header_v5),
    ] {
        header.simulation_seed_derivation_version += 1;
        assert!(validate(&header).is_err());
    }
    let mut wrong_policy = v5.clone();
    wrong_policy.cpus[0].policy_version = 16;
    assert_eq!(wrong_policy.cpus[0].seat, SeatId::Two);
    assert!(super::validation::validate_header_v5(&wrong_policy).is_err());

    assert_eq!(replay(&Journal::new(v5)), Err(ReplayError::InvalidHeader));
    assert_eq!(
        replay_v5(&Journal::new(v4)),
        Err(ReplayError::InvalidHeader)
    );
}
