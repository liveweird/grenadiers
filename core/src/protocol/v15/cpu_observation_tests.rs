use crate::cpu::{ObservedActor, WorldId};
use crate::game::{
    ArenaGenerationConfig, EffectKind as DomainEffect, EnabledItems, PlayerCount, RoundAssignment,
    SeatId, StormCloudConfig, Tick,
};
use crate::protocol::{Counter, RoomCode, RoomId};
use crate::room::{BoardSize, EnvironmentRoomConfiguration, RoomConfiguration, SeatControl};

use super::*;

const WORLD: WorldId = WorldId {
    room_instance: 41,
    physical_round: 1,
};

fn publication(
    state: &crate::game::State,
    environment: StormCloudConfig,
    items: ItemSet,
    enabled: EnabledItems,
) -> BoundPublication {
    let capture = capture_expanded(state, &[]).unwrap();
    let room = room(environment, items, capture.0.world.physical_tick);
    bind_expanded(&room, room.round_id, &capture, generation(enabled)).unwrap()
}

fn generation(enabled: EnabledItems) -> crate::game::GenerationRecord {
    crate::game::generate_multiplayer_arena(
        [22; 32],
        enabled,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: BoardSize::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
    )
    .unwrap()
    .record()
}

fn room(environment: StormCloudConfig, items: ItemSet, tick: crate::protocol::Tick) -> RoomView {
    let base = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Human],
        None,
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let seats = [Seat::One, Seat::Two];
    RoomView {
        settings: RoomSettings::new(
            GameSettings::new(0, 256, 3, 180, items).unwrap(),
            Environment {
                storm_cloud: environment == StormCloudConfig::Single,
            },
        ),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: Some(Seat::One),
        available_items: crate::protocol::v14::projection_supported_items::available_items_for(
            crate::protocol::v14::RulesVersion::MULTIPLAYER_V6,
        ),
        configuration: super::RoomConfiguration::from_domain(EnvironmentRoomConfiguration::new(
            base,
            environment,
        ))
        .unwrap(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick,
        rules_version: RulesVersion::ENVIRONMENT_V7,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: seats
            .into_iter()
            .map(|seat| Score {
                seat,
                wins: Wins::new(0).unwrap(),
            })
            .collect(),
        last_round: None,
        seats: seats
            .into_iter()
            .map(|seat| SeatStatus {
                seat,
                occupied: true,
                ready: false,
                rematch_ready: false,
                controller: Controller::Human,
                connection: ConnectionStatus::Connected,
            })
            .collect(),
        phase: RoomPhase::Running,
    }
}

#[test]
fn checked_single_cloud_publication_becomes_rules_seven_cpu_observation() {
    let mut state = super::tests::state(StormCloudConfig::Single);
    let _ = state.step(&[]).unwrap();
    let publication = publication(
        &state,
        StormCloudConfig::Single,
        ItemSet::empty(),
        EnabledItems::EMPTY,
    );
    let projected = project_recipient(&publication, Seat::One).unwrap();
    let observed = observe_cpu_recipient(&publication, Seat::One, WORLD).unwrap();

    assert_eq!(observed.parts().world, WORLD);
    assert_eq!(observed.parts().rules.version, 7);
    assert_eq!(observed.parts().tick.0, projected.state.physical_tick.get());
    let environment = observed.environment().unwrap();
    assert_eq!(environment.storm_cloud_config(), StormCloudConfig::Single);
    assert_eq!(
        environment.storm_cloud().unwrap().leg_remaining_ticks,
        projected.state.storm_cloud.unwrap().leg_remaining_ticks
    );
    assert_eq!(
        environment.actor_storm_slow_remaining_ticks(),
        &[Some(180), Some(0)]
    );
}

#[test]
fn disabled_environment_still_uses_rules_seven_cpu_metadata() {
    let state = super::tests::state(StormCloudConfig::Disabled);
    let publication = publication(
        &state,
        StormCloudConfig::Disabled,
        ItemSet::empty(),
        EnabledItems::EMPTY,
    );
    let observed = observe_cpu_recipient(&publication, Seat::Two, WORLD).unwrap();
    let environment = observed.environment().unwrap();

    assert_eq!(observed.parts().rules.version, 7);
    assert_eq!(environment.storm_cloud_config(), StormCloudConfig::Disabled);
    assert_eq!(environment.storm_cloud(), None);
    assert_eq!(
        environment.actor_storm_slow_remaining_ticks(),
        &[Some(0), Some(0)]
    );
}

#[test]
fn hidden_actor_has_neither_position_nor_storm_timer() {
    let enabled = EnabledItems::INVISIBILITY_CLOAK;
    let mut state = super::tests::state_with_enabled_items(StormCloudConfig::Single, enabled);
    state.apply_test_effect(SeatId::Two, DomainEffect::InvisibilityCloak, Tick(0));
    let publication = publication(
        &state,
        StormCloudConfig::Single,
        ItemSet::new([ItemId::InvisibilityCloak]).unwrap(),
        enabled,
    );
    let observed = observe_cpu_recipient(&publication, Seat::One, WORLD).unwrap();

    assert!(matches!(
        observed.parts().actors[1],
        ObservedActor::Hidden { .. }
    ));
    assert_eq!(
        observed
            .environment()
            .unwrap()
            .actor_storm_slow_remaining_ticks()[1],
        None
    );
}

#[test]
fn recipient_outside_checked_roster_is_rejected() {
    let state = super::tests::state(StormCloudConfig::Disabled);
    let publication = publication(
        &state,
        StormCloudConfig::Disabled,
        ItemSet::empty(),
        EnabledItems::EMPTY,
    );
    assert_eq!(
        observe_cpu_recipient(&publication, Seat::Three, WORLD),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn world_from_another_physical_round_is_rejected() {
    let state = super::tests::state(StormCloudConfig::Disabled);
    let publication = publication(
        &state,
        StormCloudConfig::Disabled,
        ItemSet::empty(),
        EnabledItems::EMPTY,
    );
    let wrong_round = WorldId {
        physical_round: 2,
        ..WORLD
    };

    assert_eq!(
        observe_cpu_recipient(&publication, Seat::One, wrong_round),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn mismatched_environment_cannot_form_a_cpu_publication() {
    let state = super::tests::state(StormCloudConfig::Disabled);
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = room(
        StormCloudConfig::Single,
        ItemSet::empty(),
        capture.0.world.physical_tick,
    );

    assert!(matches!(
        bind_expanded(
            &room,
            room.round_id,
            &capture,
            generation(EnabledItems::EMPTY),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    ));
}
