use crate::cpu::{
    BombKnowledge, ObservationError, ObservationParts, ObservedActor, ObservedArena,
    ObservedEnvironment, ObservedLairs, ObservedRules, UnknownActorFacts, VisibleActor, WorldId,
};
use crate::game::geometry::MovementSegment;
use crate::game::{
    CellId, Direction, EntityId, LairView, MobView, Position, SeatId, SpeedLevel, StormCloudConfig,
    Terrain, Tick, lair_layout_v1,
};
use crate::room::{BoardSize, LairConfig, LairSettings, RoomSettings};

use super::*;

pub(crate) fn observation(mobs: Vec<MobView>, next_spawn_at: Tick) -> Observation {
    let board = BoardSize::FifteenByThirteen;
    let cells = lair_layout_v1(board);
    let mut terrain = vec![Terrain::Floor; board.cell_count()];
    for cell in cells {
        terrain[usize::from(cell.row) * 15 + usize::from(cell.column)] = Terrain::Wall;
    }
    let actors = vec![
        ObservedActor::Hidden {
            id: EntityId(1),
            seat: SeatId::One,
            alive: true,
        },
        ObservedActor::Visible(VisibleActor {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
            alive: true,
            bomb_capacity: 1,
            bomb_range: 2,
            live_bomb_count: 0,
            equipment: None,
            weapon: None,
            effects: Vec::new(),
            damage_grace_remaining_ticks: 0,
            voluntary_actions_frozen: false,
            last_teleport_tick: None,
            unknown: UnknownActorFacts::ALL,
        }),
    ];
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 1,
            physical_round: 1,
        },
        tick: Tick(100),
        recipient: SeatId::Two,
        rules: ObservedRules {
            version: 8,
            tick_rate: 60,
            running_updates: 100,
            remaining_ticks: 100,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: 15,
            height: 13,
            terrain,
            blocks: Vec::new(),
        },
        actors,
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    };
    let counts = [
        mobs.iter().filter(|mob| mob.lair == 0).count() as u8,
        mobs.iter().filter(|mob| mob.lair == 1).count() as u8,
    ];
    let lairs = cells
        .into_iter()
        .enumerate()
        .map(|(index, cell)| LairView {
            cell,
            next_spawn_at,
            living_mobs: counts[index],
        })
        .collect();
    Observation::new_rules_v8(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![None, Some(0)]),
        ObservedLairs::new(
            LairConfig::Enabled(LairSettings::new(5, 3).unwrap()),
            lairs,
            mobs,
        ),
    )
    .unwrap()
}

pub(crate) fn mob(id: u32, position: Position) -> MobView {
    MobView {
        id: EntityId(id),
        lair: 0,
        position,
        heading: Direction::East,
        frozen: false,
    }
}

#[test]
fn rules_v8_overlay_checks_layout_counts_ids_positions_and_due_zero() {
    let valid = observation(
        vec![mob(10, Position::center(CellId::new(4, 4)))],
        Tick(100),
    );
    assert_eq!(valid.lairs().unwrap().lairs()[0].next_spawn_at, Tick(100));

    let environment = valid.environment().unwrap().clone();
    let overlay = valid.lairs().unwrap();
    let mut mobs = overlay.mobs().to_vec();
    mobs[0].id = EntityId(2);
    let duplicate = ObservedLairs::new(overlay.config(), overlay.lairs().to_vec(), mobs);
    assert_eq!(
        Observation::new_rules_v8(valid.clone().into_parts(), environment.clone(), duplicate),
        Err(ObservationError::InvalidActors)
    );

    let mut lairs = overlay.lairs().to_vec();
    lairs[0].cell = CellId::new(5, 5);
    let wrong_layout = ObservedLairs::new(overlay.config(), lairs, overlay.mobs().to_vec());
    assert_eq!(
        Observation::new_rules_v8(
            valid.clone().into_parts(),
            environment.clone(),
            wrong_layout
        ),
        Err(ObservationError::InvalidActors)
    );

    let mut lairs = overlay.lairs().to_vec();
    lairs[0].living_mobs = 0;
    let wrong_count = ObservedLairs::new(overlay.config(), lairs, overlay.mobs().to_vec());
    assert_eq!(
        Observation::new_rules_v8(valid.clone().into_parts(), environment.clone(), wrong_count),
        Err(ObservationError::InvalidActors)
    );

    let mut lairs = overlay.lairs().to_vec();
    lairs[1].next_spawn_at = Tick(101);
    let unequal_timers = ObservedLairs::new(overlay.config(), lairs, overlay.mobs().to_vec());
    assert_eq!(
        Observation::new_rules_v8(valid.into_parts(), environment, unequal_timers),
        Err(ObservationError::InvalidActors)
    );
}

#[test]
fn disabled_overlay_requires_empty_facts_and_old_constructors_stay_closed() {
    let valid = observation(Vec::new(), Tick(200));
    let parts = valid.clone().into_parts();
    let environment = valid.environment().unwrap().clone();
    assert!(Observation::new(parts.clone()).is_err());
    assert!(Observation::new_rules_v7(parts.clone(), environment.clone()).is_err());
    assert!(
        Observation::new_rules_v8(
            parts.clone(),
            environment.clone(),
            ObservedLairs::new(LairConfig::Disabled, Vec::new(), Vec::new())
        )
        .is_ok()
    );
    assert_eq!(
        Observation::new_rules_v8(
            parts,
            environment,
            ObservedLairs::new(
                LairConfig::Disabled,
                valid.lairs().unwrap().lairs().to_vec(),
                Vec::new()
            )
        ),
        Err(ObservationError::InvalidActors)
    );
}

#[test]
fn envelope_uses_unknown_residual_and_actual_timed_sweep() {
    let origin = Position::center(CellId::new(4, 4));
    let observed = observation(vec![mob(10, origin)], Tick(200));
    let envelope = MobContactEnvelope::new(&observed, &mut WorkBudget::new(100)).unwrap();
    let contact_edge = Position {
        x: origin.x + 2 * HALF_FOOTPRINT - 1 + 26,
        y: origin.y,
    };
    assert_eq!(
        envelope.sweep_is_clear(
            Tick(101),
            &[MovementSegment::new(contact_edge, contact_edge)],
            &mut WorkBudget::new(100)
        ),
        Ok(false)
    );
    let clear = Position {
        x: contact_edge.x + 1,
        ..contact_edge
    };
    assert_eq!(
        envelope.sweep_is_clear(
            Tick(101),
            &[MovementSegment::new(clear, clear)],
            &mut WorkBudget::new(100)
        ),
        Ok(true)
    );
    let crossing = MovementSegment::new(
        Position {
            x: origin.x - 1_000,
            y: origin.y,
        },
        Position {
            x: origin.x + 1_000,
            y: origin.y,
        },
    );
    assert_eq!(
        envelope.sweep_is_clear(Tick(101), &[crossing], &mut WorkBudget::new(100)),
        Ok(false)
    );
}

#[test]
fn containment_allows_motion_death_and_predicted_birth_but_rejects_escape_and_expiry() {
    let origin = Position::center(CellId::new(4, 4));
    let basis = observation(vec![mob(10, origin)], Tick(101));
    let envelope = MobContactEnvelope::new(&basis, &mut WorkBudget::new(100)).unwrap();

    let overlay = basis.lairs().unwrap();
    let mut mobs = overlay.mobs().to_vec();
    mobs[0].position.x += 26;
    let moved = ObservedLairs::new(overlay.config(), overlay.lairs().to_vec(), mobs);
    let mut parts = basis.clone().into_parts();
    parts.tick = Tick(101);
    let current =
        Observation::new_rules_v8(parts, basis.environment().unwrap().clone(), moved).unwrap();
    assert_eq!(
        envelope.contains(&current, Tick(110), &mut WorkBudget::new(100)),
        Ok(true)
    );

    let mut born_mobs = current.lairs().unwrap().mobs().to_vec();
    born_mobs.push(MobView {
        id: EntityId(11),
        lair: 0,
        position: Position::center(CellId::new(6, 5)),
        heading: Direction::South,
        frozen: true,
    });
    let born_overlay = ObservedLairs::new(
        current.lairs().unwrap().config(),
        vec![
            LairView {
                living_mobs: 2,
                ..current.lairs().unwrap().lairs()[0]
            },
            current.lairs().unwrap().lairs()[1],
        ],
        born_mobs,
    );
    let born = Observation::new_rules_v8(
        current.clone().into_parts(),
        current.environment().unwrap().clone(),
        born_overlay,
    )
    .unwrap();
    assert_eq!(
        envelope.contains(&born, Tick(110), &mut WorkBudget::new(100)),
        Ok(true)
    );

    let overlay = current.lairs().unwrap();
    let mut mobs = overlay.mobs().to_vec();
    mobs[0].position.x += 1;
    let escaped = ObservedLairs::new(overlay.config(), overlay.lairs().to_vec(), mobs);
    let impossible = Observation::new_rules_v8(
        current.clone().into_parts(),
        current.environment().unwrap().clone(),
        escaped,
    )
    .unwrap();
    assert_eq!(
        envelope.contains(&impossible, Tick(110), &mut WorkBudget::new(100)),
        Ok(false)
    );
    assert_eq!(
        envelope.contains(&current, Tick(101), &mut WorkBudget::new(100)),
        Ok(false)
    );
    for (room_instance, physical_round) in [(2, 1), (1, 2)] {
        let other_world = current
            .clone()
            .map_parts(|parts| {
                parts.world = WorldId {
                    room_instance,
                    physical_round,
                };
            })
            .unwrap();
        assert_eq!(
            envelope.contains(&other_world, Tick(110), &mut WorkBudget::new(100)),
            Ok(false)
        );
    }
}

#[test]
fn urgent_helper_fails_closed_when_work_is_exhausted() {
    let observed = observation(Vec::new(), Tick(101));
    assert!(has_urgent_lair_threat(
        &observed,
        Tick(112),
        &mut WorkBudget::new(0)
    ));
}
