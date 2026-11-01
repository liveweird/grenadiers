use grenadiers_core::{
    cpu::{
        BombKnowledge, ManualIntent, MovementIntent, ObservedActor, ObservedBombTrigger, WorldId,
        policy::{StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason, StrategicTarget},
        survival::WORK_ALLOWANCE,
    },
    game::{BlastKind, CellId, Direction, EntityId, Position, SeatId, Tick},
    room::{CpuPolicy, RespawnLairConfig, RespawnLairSettings},
};

use super::{
    early_farm_owner_interaction_support::run as run_interaction,
    early_farm_owner_support::run as run_single,
};

#[test]
fn other_cpu_activity_reproduces_cpu_two_passivity_with_a_guarded_queued_basis() {
    let single = run_single(47, 600);
    let interaction = run_interaction(600);
    eprintln!(
        "paired single={:?} interaction_plants={:?} cpu2_initial_work={:?} jobs={}/{} basis={:?} admissions={:?}",
        single.first_bomb,
        interaction.first_plants,
        &interaction.pump_batches[..3],
        interaction.worker.delivered,
        interaction.worker.dequeued,
        interaction.basis_probe,
        interaction.admissions,
    );
    eprintln!("EVIDENCE_FULL_BASIS {:#?}", interaction.basis_probe.capture);
    assert_eq!(single.first_bomb, Some(Tick(232)));
    assert_eq!(
        interaction.initial_positions,
        [
            Position {
                x: 13_824,
                y: 1_536
            },
            Position {
                x: 1_536,
                y: 11_776
            },
            Position {
                x: 13_824,
                y: 11_776
            },
        ]
    );
    let initial = interaction
        .cpu_two_samples
        .iter()
        .find_map(|sample| sample.intent)
        .unwrap();
    assert_eq!(
        (initial.target, initial.approach_cell),
        (
            StrategicTarget::Block {
                cell: CellId::new(11, 2)
            },
            Some(CellId::new(11, 1)),
        )
    );
    assert_eq!(
        interaction.first_plants,
        [None, Some(Tick(205)), Some(Tick(205))]
    );
    assert_eq!(
        interaction.pump_batches[..3],
        [
            vec![41_742, 41_361, 41_742],
            vec![34_401, 12_275, 12_682],
            vec![4_938, 11_499, 11_499],
        ]
    );
    let basis = &interaction.basis_probe;
    assert_eq!(basis.policy, CpuPolicy::UtilityV51);
    // This exact queued basis matches the private Teleporter counter-pressure
    // guard's public predicates. The no-plant result records support
    // suppression; it does not establish that a complete safe plant existed.
    assert_eq!(
        (basis.tick, basis.settled_stop_tick),
        (Tick(234), Tick(234))
    );
    assert_eq!(
        basis.position,
        Position {
            x: 11_727,
            y: 1_536
        }
    );
    assert_eq!(basis.position.cell(), Some(CellId::new(11, 1)));
    assert!(basis.teleporter_enabled);
    assert_eq!(basis.visible_bombs, 2);
    assert_eq!(basis.visible_flames, 0);
    assert_eq!(basis.blocks, 48);
    assert!(basis.running_updates > 0);
    assert!(!basis.exposed_teleporter);
    assert!(!basis.had_origin);
    assert!(basis.settled_stop_tick <= basis.tick);
    assert_eq!(
        basis.planner_intent.movement,
        MovementIntent::Direction(Direction::North)
    );
    assert_eq!(basis.planner_intent.manual, None::<ManualIntent>);
    assert!(!basis.planner_execution);
    assert_eq!(basis.planner_work, 10_443);
    assert_complete_transcription_basis(basis);
    let first_stall = interaction
        .cpu_two_samples
        .iter()
        .find(|sample| {
            sample
                .intent
                .is_some_and(|intent| intent.last_switch_reason == StrategicSwitchReason::Stalled)
        })
        .unwrap();
    assert_eq!(first_stall.tick, Tick(235));
    assert_eq!(
        first_stall.position,
        Position {
            x: 11_776,
            y: 1_534
        }
    );
    assert_eq!(
        first_stall.intent.unwrap().target,
        StrategicTarget::Block {
            cell: CellId::new(11, 2)
        }
    );
    assert!(interaction.cpu_two_samples.iter().any(|sample| {
        sample.intent.is_some_and(|intent| {
            intent.target
                == StrategicTarget::Block {
                    cell: CellId::new(9, 1),
                }
                && intent.approach_cell == Some(CellId::new(10, 1))
        })
    }));
    assert!(
        interaction
            .cpu_two_samples
            .iter()
            .any(|sample| { sample.controller.contains("pending=Some") })
    );
    assert!(interaction.cpu_two_samples.iter().all(|sample| {
        sample.controller.contains("origin=false")
            && sample.controller.contains("route=false")
            && sample.controller.contains("fence=false")
    }));
    assert_eq!(
        interaction.cpu_two_samples.last().unwrap().position,
        Position {
            x: 11_776,
            y: 1_344
        }
    );
    assert_eq!(interaction.worker.dequeued, interaction.worker.delivered);
    assert_eq!(interaction.worker.dequeued, 263);
    assert_eq!(
        (interaction.worker.failed, interaction.worker.cancelled),
        (0, 0)
    );
    assert!(
        interaction
            .worker
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    assert!(
        interaction
            .admissions
            .iter()
            .all(|admission| *admission == Default::default())
    );
}

fn assert_complete_transcription_basis(
    basis: &super::early_farm_owner_interaction_support::BasisProbe,
) {
    let capture = &basis.capture;
    let parts = capture.observation.parts();
    assert_eq!(
        (parts.world, parts.tick, parts.recipient),
        (
            WorldId {
                room_instance: 5_461,
                physical_round: 1,
            },
            Tick(234),
            SeatId::Two,
        )
    );
    assert_eq!(
        (parts.rules.running_updates, parts.rules.remaining_ticks),
        (54, 10_746)
    );
    assert_eq!(capture.settled_stop_tick, Tick(234));
    assert!(!capture.had_origin);
    assert_eq!(capture.hazard_memory.key(), capture.memory.key());

    let actors = parts
        .actors
        .iter()
        .map(|actor| match actor {
            ObservedActor::Visible(actor) => {
                assert!(actor.alive);
                assert!(actor.effects.is_empty());
                assert_eq!(actor.damage_grace_remaining_ticks, 0);
                assert!(!actor.voluntary_actions_frozen);
                (
                    actor.id,
                    actor.seat,
                    actor.position,
                    actor.heading,
                    actor.live_bomb_count,
                )
            }
            ObservedActor::Hidden { .. } => panic!("all four actors must be visible"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actors,
        vec![
            (
                EntityId(1),
                SeatId::One,
                Position { x: 1_536, y: 1_536 },
                Direction::East,
                0
            ),
            (
                EntityId(2),
                SeatId::Two,
                Position {
                    x: 11_727,
                    y: 1_536
                },
                Direction::West,
                0
            ),
            (
                EntityId(3),
                SeatId::Three,
                Position {
                    x: 1_638,
                    y: 11_776
                },
                Direction::East,
                1
            ),
            (
                EntityId(4),
                SeatId::Four,
                Position {
                    x: 13_722,
                    y: 11_776
                },
                Direction::West,
                1
            ),
        ]
    );

    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        panic!("the queued basis must carry complete visible bomb facts");
    };
    assert_eq!(bombs.len(), 2);
    for (bomb, id, owner, cell) in [
        (&bombs[0], EntityId(0), EntityId(3), CellId::new(1, 10)),
        (&bombs[1], EntityId(5), EntityId(4), CellId::new(13, 10)),
    ] {
        assert_eq!((bomb.id, bomb.owner, bomb.cell), (id, owner, cell));
        assert_eq!(
            (bomb.placed_at, bomb.range, bomb.blast),
            (Tick(205), 2, BlastKind::Standard)
        );
        assert_eq!(
            bomb.trigger,
            ObservedBombTrigger::Timed {
                remaining_ticks: 151,
                paused: false,
            }
        );
    }
    assert!(parts.flames.is_empty());
    assert!(parts.pickups.is_empty());

    let lairs = capture.observation.respawn_lairs().unwrap();
    assert_eq!(
        lairs.config(),
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT)
    );
    assert_eq!(lairs.mobs(), []);
    assert_eq!(
        lairs
            .lairs()
            .iter()
            .map(|lair| (lair.cell, lair.next_spawn_at, lair.living_mobs))
            .collect::<Vec<_>>(),
        vec![
            (CellId::new(6, 6), Some(Tick(780)), 0),
            (CellId::new(8, 6), Some(Tick(780)), 0),
        ]
    );

    let intent = capture.memory.strategic_intent().unwrap();
    assert_eq!(
        (intent.objective, intent.phase),
        (StrategicObjectiveKind::Farm, StrategicPhase::Approach)
    );
    assert_eq!(
        (intent.committed_until, intent.progress_deadline),
        (Tick(303), Tick(390))
    );
    assert_eq!(
        (intent.observation_tick, intent.last_distance),
        (Tick(210), 1)
    );
    assert_eq!(capture.memory.route()[0], Some(CellId::new(11, 1)));
    assert_eq!(capture.memory.refuge(), Some(CellId::new(11, 1)));
}
