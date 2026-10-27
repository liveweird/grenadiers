use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::{
    self, ActorInit, EffectKind, EnabledItems, OilState, Position, RulesConfig, SeatId, SpeedLevel,
    State, World,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::{COUNTDOWN_TICKS, MatchChange, MatchLifecycle, RoomSettings};

use super::super::v8::{
    AdmissionProjection, BoundPublication, ExpandedCapture, GameSettings, ItemSet, MapId,
    PersonalAdmission, ProjectionError, ResumeToken, RoomMetadata, RoomView, RulesVersion,
};

pub(super) type ProjectRoom =
    fn(RoomMetadata, &MatchLifecycle) -> Result<RoomView, ProjectionError>;
pub(super) type Capture =
    fn(&State, &[game::StepReport]) -> Result<ExpandedCapture, ProjectionError>;
pub(super) type Bind = fn(
    &RoomView,
    Counter,
    &ExpandedCapture,
    Option<game::GenerationRecord>,
) -> Result<BoundPublication, ProjectionError>;
pub(super) type PreparePair = fn(&BoundPublication) -> Result<[String; 2], ProjectionError>;
pub(super) type PrepareAdmission =
    for<'a> fn(AdmissionProjection<'a>, PersonalAdmission) -> Result<String, ProjectionError>;

#[derive(Clone, Copy)]
pub(super) struct VersionApi {
    pub version: u16,
    pub project_room: ProjectRoom,
    pub capture: Capture,
    pub bind: Bind,
    pub prepare_pair: PreparePair,
    pub prepare_admission: PrepareAdmission,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FrameCase {
    pub case_id: String,
    pub version: u16,
    pub scenario: String,
    pub recipient: Option<Seat>,
    pub frame: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Corpus {
    pub cases: Vec<FrameCase>,
}

fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn wire_settings(enabled: EnabledItems) -> GameSettings {
    GameSettings::new(
        0,
        256,
        3,
        180,
        ItemSet::from_mask(super::super::v8::ItemMask::new(enabled.bits()).unwrap()),
    )
    .unwrap()
}

fn metadata(enabled: EnabledItems) -> RoomMetadata {
    RoomMetadata {
        settings: wire_settings(enabled),
        settings_revision: counter(1),
        settings_editor: None,
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        revision: counter(1),
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
    }
}

fn waiting_lifecycle(enabled: EnabledItems) -> MatchLifecycle {
    let mut lifecycle =
        MatchLifecycle::with_settings(RoomSettings::DEFAULT.with_enabled_items(enabled));
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle
}

fn running_lifecycle(enabled: EnabledItems) -> MatchLifecycle {
    let mut lifecycle = waiting_lifecycle(enabled);
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    for _ in 0..COUNTDOWN_TICKS - 1 {
        lifecycle.advance_idle_tick().unwrap();
    }
    assert!(matches!(
        lifecycle.advance_idle_tick().unwrap(),
        MatchChange::RoundStarted(_)
    ));
    lifecycle
}

fn actors(world: &World) -> [ActorInit; 2] {
    let spawns = world.arena().spawns().to_vec();
    SeatId::DUEL.map(|seat| ActorInit {
        id: game::EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: game::Direction::East,
        speed: SpeedLevel::Base,
    })
}

fn state_at(lifecycle: &MatchLifecycle, enabled: EnabledItems, world: World) -> State {
    State::new_at_tick_with_assignment(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        world.clone(),
        actors(&world),
        lifecycle.tick(),
        game::RoundAssignment::first(),
    )
    .unwrap()
}

pub(super) fn waiting_case(api: VersionApi, enabled: EnabledItems) -> FrameCase {
    let lifecycle = waiting_lifecycle(enabled);
    let room = (api.project_room)(metadata(enabled), &lifecycle).unwrap();
    let frame = (api.prepare_admission)(
        AdmissionProjection {
            room: &room,
            state_epoch: None,
            capture: None,
            generation: None,
        },
        PersonalAdmission {
            recipient: Seat::One,
            last_action_id: None,
            sequence: counter(1),
            resume_token: ResumeToken::new("c".repeat(64)).unwrap(),
        },
    )
    .unwrap();
    FrameCase {
        case_id: format!("v{}-waiting-admission", api.version),
        version: api.version,
        scenario: "waitingAdmission".into(),
        recipient: Some(Seat::One),
        frame,
    }
}

fn pair_cases(
    api: VersionApi,
    enabled: EnabledItems,
    scenario: &str,
    lifecycle: &MatchLifecycle,
    state: &State,
) -> Vec<FrameCase> {
    let room = (api.project_room)(metadata(enabled), lifecycle).unwrap();
    let capture = (api.capture)(state, &[]).unwrap();
    let generation = game::generate_expanded_arena([api.version as u8; 32], enabled).record();
    let publication = (api.bind)(&room, room.round_id, &capture, Some(generation)).unwrap();
    (api.prepare_pair)(&publication)
        .unwrap()
        .into_iter()
        .zip([Seat::One, Seat::Two])
        .map(|(frame, recipient)| FrameCase {
            case_id: format!("v{}-{scenario}-{}", api.version, seat_name(recipient)),
            version: api.version,
            scenario: scenario.into(),
            recipient: Some(recipient),
            frame,
        })
        .collect()
}

fn seat_name(seat: Seat) -> &'static str {
    match seat {
        Seat::One => "one",
        Seat::Two => "two",
    }
}

pub(super) fn privacy_pair(
    api: VersionApi,
    enabled: EnabledItems,
    scenario: &str,
) -> Vec<FrameCase> {
    let lifecycle = running_lifecycle(enabled);
    let mut state = state_at(&lifecycle, enabled, World::new(game::fixed_arena()));
    state.apply_test_effect(SeatId::Two, EffectKind::InvisibilityCloak, state.tick());
    pair_cases(api, enabled, scenario, &lifecycle, &state)
}

pub(super) fn frozen_pair(api: VersionApi) -> Vec<FrameCase> {
    let enabled = EnabledItems::STOPWATCH;
    let lifecycle = running_lifecycle(enabled);
    let mut state = state_at(&lifecycle, enabled, World::new(game::fixed_arena()));
    state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, state.tick());
    pair_cases(api, enabled, "runningFrozen", &lifecycle, &state)
}

pub(super) fn teleport_pairs(api: VersionApi) -> Vec<FrameCase> {
    let enabled = EnabledItems::from_bits(
        EnabledItems::TELEPORTER.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap();
    let mut lifecycle = running_lifecycle(enabled);
    let spawn = game::fixed_arena().spawns()[0];
    let world = World::with_pickups(
        game::fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawn, game::PickupKind::Teleporter)]),
    )
    .unwrap();
    let mut state = state_at(&lifecycle, enabled, world);
    let mut cases = pair_cases(api, enabled, "runningTeleportNull", &lifecycle, &state);
    state.step(&[]).unwrap();
    lifecycle.complete_running_tick(None).unwrap();
    state.apply_test_effect(SeatId::Two, EffectKind::InvisibilityCloak, state.tick());
    cases.extend(pair_cases(
        api,
        enabled,
        "runningTeleported",
        &lifecycle,
        &state,
    ));
    cases
}

pub(super) fn oil_pairs(api: VersionApi) -> Vec<FrameCase> {
    let enabled = EnabledItems::from_bits(
        EnabledItems::OIL_BARREL.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap();
    let lifecycle = running_lifecycle(enabled);
    let empty = state_at(&lifecycle, enabled, World::new(game::fixed_arena()));
    let mut cases = pair_cases(api, enabled, "runningOilEmpty", &lifecycle, &empty);
    let world = World::with_pickups_and_oil(
        game::fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (game::CellId::new(4, 1), OilState::Unlit),
            (game::CellId::new(3, 3), OilState::Burning),
            (game::CellId::new(5, 1), OilState::Concealed),
        ]),
    )
    .unwrap();
    let mut materialized = state_at(&lifecycle, enabled, world);
    materialized.apply_test_effect(
        SeatId::Two,
        EffectKind::InvisibilityCloak,
        materialized.tick(),
    );
    cases.extend(pair_cases(
        api,
        enabled,
        "runningOilMaterialized",
        &lifecycle,
        &materialized,
    ));
    cases
}
