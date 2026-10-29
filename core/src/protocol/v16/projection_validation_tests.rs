use crate::game::{PlayerCount, StormCloudConfig};
use crate::room::{BoardSize, LairConfig, LairSettings};

use super::*;

fn populated(board: BoardSize) -> super::server_conformance_tests::WorldCase {
    super::server_conformance_tests::world_case(
        board,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap()),
        300,
        (false, false),
        [29; 32],
    )
}

#[test]
fn checked_publication_rejects_mob_footprints_outside_every_board() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let original = populated(board);
        let (width, height) = board.dimensions();
        for (x, y) in [
            (0, 320),
            (319, 320),
            (u16::from(width) * 1_024 - 319, 320),
            (320, 0),
            (320, 319),
            (320, u16::from(height) * 1_024 - 319),
        ] {
            let mut capture = original.capture.clone();
            capture.mobs[0].position = crate::protocol::Position::new(x, y).unwrap();
            assert!(
                matches!(
                    bind_expanded(
                        &original.room,
                        original.room.round_id,
                        &capture,
                        original.generation,
                    ),
                    Err(ProjectionError::InvalidExpandedBinding)
                ),
                "{board:?} accepted ({x}, {y})",
            );
        }
    }
}

#[test]
fn eleven_by_nine_rejects_reported_edge_centers() {
    let original = populated(BoardSize::ElevenByNine);
    for x in [10_945, 11_264] {
        let mut capture = original.capture.clone();
        capture.mobs[0].position = crate::protocol::Position::new(x, 320).unwrap();
        assert!(
            bind_expanded(
                &original.room,
                original.room.round_id,
                &capture,
                original.generation,
            )
            .is_err()
        );
    }
}

#[test]
fn checked_publication_binds_mob_freeze_to_a_living_stopwatch_holder() {
    let frozen = super::server_conformance_tests::world_case(
        BoardSize::ElevenByNine,
        PlayerCount::Two,
        StormCloudConfig::Disabled,
        LairConfig::Enabled(LairSettings::new(5, 2).unwrap()),
        300,
        (false, true),
        [30; 32],
    );
    assert!(
        bind_expanded(
            &frozen.room,
            frozen.room.round_id,
            &frozen.capture,
            frozen.generation,
        )
        .is_ok()
    );

    let mut expired = frozen.capture.clone();
    expired.base.world.actors[0].view.effects = EffectViews::default();
    assert!(
        bind_expanded(
            &frozen.room,
            frozen.room.round_id,
            &expired,
            frozen.generation,
        )
        .is_err()
    );

    let mut dead = frozen.capture.clone();
    dead.base.world.actors[0].view.alive = false;
    dead.base.world.actors[0].view.effects = EffectViews::default();
    assert!(bind_expanded(&frozen.room, frozen.room.round_id, &dead, frozen.generation,).is_err());

    let mut unfrozen = frozen.capture.clone();
    for mob in &mut unfrozen.mobs {
        mob.frozen = false;
    }
    assert!(
        bind_expanded(
            &frozen.room,
            frozen.room.round_id,
            &unfrozen,
            frozen.generation,
        )
        .is_err()
    );
}
