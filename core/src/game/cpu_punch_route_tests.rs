use std::collections::BTreeSet;

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::routing::{PunchRequest, RouteLimits, RouteStep, plan_punch};
use crate::cpu::{MemoryKey, Observation, WorkBudget, WorldId};
use crate::protocol::v11::{
    ConnectionStatus, GameSettings, ItemMask, ItemSet, MapId, RoomPhase, RoomView, RoundCount,
    RulesVersion, Scores, SeatStatus, Wins,
};
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::super::{
    ActorInit, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, Tick, World,
};

const CPU: SeatId = SeatId::Two;
const SOURCE_ID: EntityId = EntityId(90);
const SOURCE: CellId = CellId::new(8, 7);
const WORLD: WorldId = WorldId {
    room_instance: 96,
    physical_round: 4,
};

fn settings() -> RoomSettings {
    RoomSettings::new(SpeedLevel::Four, 256, 3, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::PUNCH)
}

fn base_state() -> State {
    let arena = crate::game::test_support::open_border_arena(BTreeSet::new());
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Four,
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(CellId::new(7, 7)),
            heading: Direction::East,
            speed: SpeedLevel::Four,
        },
    ];
    let mut state = State::new(
        RulesConfig::expanded_v5(settings()),
        World::new(arena),
        actors,
    )
    .unwrap();
    state.replace_actor_equipment(CPU, Equipment::Punch);
    state
}

fn paired_states() -> (State, State) {
    let mut ready = base_state();
    for _ in 0..4 {
        ready.step(&[]).unwrap();
    }
    ready
        .insert_test_bomb(
            SOURCE_ID,
            SOURCE,
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(180),
            },
        )
        .unwrap();
    let mut cooling = ready.clone();
    cooling.actors[CPU.index()].last_successful_punch_tick = Some(Tick(1));
    (ready, cooling)
}

fn observation(state: &State) -> Observation {
    let settings = settings();
    let capture = crate::protocol::v11::capture_expanded(state, &[]).unwrap();
    let items = ItemSet::from_mask(ItemMask::new(settings.enabled_items().bits()).unwrap());
    let room = RoomView {
        settings: GameSettings::new(
            4,
            settings.turn_assist_units(),
            settings.wins_to_finish(),
            u16::try_from(settings.bomb_fuse_ticks()).unwrap(),
            items,
        )
        .unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: ItemSet::from_mask(ItemMask::new(EnabledItems::ALL.bits()).unwrap()),
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

fn scheduled_inputs(
    certificate: &crate::cpu::routing::PunchCertificate,
    next: Tick,
) -> Vec<SeatInput> {
    let lane = certificate
        .route()
        .lane(certificate.execution_tick())
        .unwrap();
    let mut inputs = Vec::new();
    for (index, step) in certificate.route().steps().iter().enumerate() {
        match step {
            RouteStep::Move { direction, .. } => {
                if lane.step_start(index) == Some(next) {
                    inputs.push(SeatInput {
                        seat: CPU,
                        action: InputAction::Press(*direction),
                    });
                }
                if lane.stop_update(index) == Some(next) {
                    inputs.push(SeatInput {
                        seat: CPU,
                        action: InputAction::Clear,
                    });
                }
            }
            RouteStep::Wait { .. } if index == 0 && lane.step_start(index) == Some(next) => {
                inputs.push(SeatInput {
                    seat: CPU,
                    action: InputAction::Clear,
                });
            }
            RouteStep::Wait { .. } => {}
        }
    }
    if next == certificate.execution_tick() {
        inputs.push(SeatInput {
            seat: CPU,
            action: InputAction::UseEquipment,
        });
    }
    inputs
}

#[test]
fn one_common_punch_escape_survives_ready_relocation_and_cooldown_no_op() {
    let (mut ready, mut cooling) = paired_states();
    let basis = observation(&ready);
    assert_eq!(observation(&cooling), basis);
    let key = MemoryKey {
        world: WORLD,
        cpu_seat: CPU,
    };
    let certificate = plan_punch(
        &basis,
        &ObservedHazardMemory::new(key),
        PunchRequest {
            execution_tick: Tick(11),
            settled_stop_tick: Tick(4),
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();

    while ready.tick() < Tick(10) {
        ready.step(&[]).unwrap();
        cooling.step(&[]).unwrap();
    }
    assert!(certificate.can_admit(&basis, &observation(&ready), Tick(10)));
    assert!(certificate.can_admit(&basis, &observation(&cooling), Tick(10)));

    while ready.tick().0 + 1 < certificate.route().survive_until().0 {
        let next = Tick(ready.tick().0 + 1);
        let inputs = scheduled_inputs(&certificate, next);
        ready.step(&inputs).unwrap();
        cooling.step(&inputs).unwrap();
        if next == certificate.execution_tick() {
            assert_eq!(ready.world().bombs()[&SOURCE_ID].cell(), CellId::new(11, 7));
            assert_eq!(cooling.world().bombs()[&SOURCE_ID].cell(), SOURCE);
        }
        let expected = certificate
            .route()
            .lane(certificate.execution_tick())
            .unwrap()
            .pose_at(next);
        assert_eq!(
            expected,
            Some((
                ready.actors()[CPU.index()].position(),
                ready.actors()[CPU.index()].heading()
            ))
        );
        assert_eq!(
            expected,
            Some((
                cooling.actors()[CPU.index()].position(),
                cooling.actors()[CPU.index()].heading(),
            ))
        );
    }
    ready.step(&[]).unwrap();
    cooling.step(&[]).unwrap();
    assert!(ready.actors()[CPU.index()].alive());
    assert!(cooling.actors()[CPU.index()].alive());
}
