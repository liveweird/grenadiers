use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    BombKnowledge, CpuRng, Intent, MemoryKey, MovementIntent, Observation, WorkBudget,
};
use crate::game::{
    self, BombTrigger, CellId, Direction, EffectKind, EntityId, Equipment, InputAction, OilState,
    PickupKind, SeatId, SeatInput, Tick, World,
};
use crate::protocol::{Cell, Seat};
use crate::room::RoomSettings;

use super::cpu_observation_tests::{WORLD, bind_capture, enabled, state};
use super::*;

fn fixture_decision(observation: &Observation, rng: &mut CpuRng) -> Intent {
    let directions = [
        game::Direction::North,
        game::Direction::East,
        game::Direction::South,
        game::Direction::West,
    ];
    assert!(matches!(
        observation.parts().bombs,
        BombKnowledge::Unavailable
    ));
    Intent::moving(directions[rng.bounded_u32(4).unwrap() as usize])
}

#[test]
fn concealed_differences_are_observation_and_decision_inert_for_both_seats() {
    let enabled = enabled(&[
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::OilBarrel,
        ItemId::Shield,
        ItemId::Heart,
    ]);
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled);
    let pickup_cell = CellId::new(2, 2);
    let oil_cell = CellId::new(3, 2);
    let arena = game::test_support::open_border_arena(BTreeSet::from([pickup_cell, oil_cell]));
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::from([(pickup_cell, game::PickupKind::Shield)]),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil_cell, OilState::Concealed)]),
    )
    .unwrap();
    let mut state = state(settings, world);
    for seat in SeatId::ALL {
        state.apply_test_effect(seat, EffectKind::InvisibilityCloak, Tick(0));
        state.apply_test_effect(seat, EffectKind::InvisibleBombs, Tick(0));
    }
    for (id, cell, owner) in [
        (20, CellId::new(4, 3), EntityId(1)),
        (21, CellId::new(5, 3), EntityId(2)),
    ] {
        state
            .insert_test_bomb(
                EntityId(id),
                cell,
                owner,
                2,
                Tick(0),
                BombTrigger::Timed {
                    detonates_at: Tick(180),
                },
            )
            .unwrap();
    }
    let original = capture_expanded(&state, &[]).unwrap();

    for recipient in [Seat::One, Seat::Two] {
        let mut changed = original.clone();
        changed.world.0.private_pickup_cells = vec![Cell::new(3, 2).unwrap()];
        changed.world.0.oil[0].cell = Cell::new(2, 2).unwrap();
        swap_private_bombs(&mut changed);
        let rival = usize::from(recipient == Seat::One);
        changed.world.0.actors[rival].position =
            crate::protocol::Position::new(7_680, 6_656).unwrap();
        changed.world.0.actors[rival].damage_grace_remaining_ticks =
            DamageGraceTicks::new(12).unwrap();

        let first =
            observe_cpu_recipient(&bind_capture(&original, settings), recipient, WORLD).unwrap();
        let second =
            observe_cpu_recipient(&bind_capture(&changed, settings), recipient, WORLD).unwrap();
        assert_eq!(first, second);

        let key = MemoryKey {
            world: WORLD,
            cpu_seat: first.parts().recipient,
        };
        let mut first_rng = CpuRng::from_seed(key, [44; 32]);
        let mut second_rng = CpuRng::from_seed(key, [44; 32]);
        let first_intent = fixture_decision(&first, &mut first_rng);
        assert_eq!(first_intent, fixture_decision(&second, &mut second_rng));
        assert!(matches!(
            first_intent.movement,
            MovementIntent::Direction(_)
        ));
    }
}

#[derive(Clone, Copy)]
enum ConcealedUnderBlock {
    Empty,
    Pickup,
    Oil,
}

fn stopped_inside_intact_block(kind: ConcealedUnderBlock) -> (crate::game::State, RoomSettings) {
    let start = CellId::new(1, 1);
    let block = CellId::new(3, 1);
    let settings = RoomSettings::DEFAULT.with_enabled_items(enabled(&[
        ItemId::WallPass,
        ItemId::Shield,
        ItemId::OilBarrel,
    ]));
    let arena = game::test_support::open_border_arena(BTreeSet::from([block]));
    let concealed = matches!(kind, ConcealedUnderBlock::Pickup)
        .then_some((block, PickupKind::Shield))
        .into_iter()
        .collect();
    let oil = matches!(kind, ConcealedUnderBlock::Oil)
        .then_some((block, OilState::Concealed))
        .into_iter()
        .collect();
    let world = World::with_pickups_and_oil(
        arena,
        concealed,
        BTreeMap::new(),
        BTreeMap::from([(start, PickupKind::WallPass)]),
        oil,
    )
    .unwrap();
    let mut state = state(settings, world);

    state.step(&[]).unwrap();
    assert_eq!(
        state.actors()[SeatId::One.index()].inventory().equipment(),
        Some(Equipment::WallPass)
    );
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    for _ in 0..31 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Release(Direction::East),
        }])
        .unwrap();

    assert_eq!(
        state.actors()[SeatId::One.index()].position().cell(),
        Some(block)
    );
    assert!(state.world().arena().blocks().contains(&block));
    assert!(state.world().exposed_pickups().is_empty());
    (state, settings)
}

#[test]
fn wall_pass_crossing_an_intact_block_does_not_observe_or_plan_from_concealed_contents() {
    let mut observations = Vec::new();
    let mut plans = Vec::new();
    let key = MemoryKey {
        world: WORLD,
        cpu_seat: SeatId::One,
    };

    for kind in [
        ConcealedUnderBlock::Empty,
        ConcealedUnderBlock::Pickup,
        ConcealedUnderBlock::Oil,
    ] {
        let (state, settings) = stopped_inside_intact_block(kind);
        let observation = observe_cpu_recipient(
            &bind_capture(&capture_expanded(&state, &[]).unwrap(), settings),
            Seat::One,
            WORLD,
        )
        .unwrap();
        let mut rng = CpuRng::from_seed(key, [87; 32]);
        let plan = crate::cpu::policy::plan(
            &observation,
            &UtilityMemory::new(key),
            &mut rng,
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        observations.push(observation);
        plans.push(plan);
    }

    assert_eq!(observations[0], observations[1]);
    assert_eq!(observations[0], observations[2]);
    assert_eq!(plans[0], plans[1]);
    assert_eq!(plans[0], plans[2]);
    assert_ne!(plans[0].intent, Intent::STOP);
}

fn swap_private_bombs(capture: &mut ExpandedCapture) {
    let first = capture.world.0.bombs[0];
    let second = capture.world.0.bombs[1];
    capture.world.0.bombs[0].cell = second.cell;
    capture.world.0.bombs[0].owner = second.owner;
    capture.world.0.bombs[1].cell = first.cell;
    capture.world.0.bombs[1].owner = first.owner;
    for bomb in capture.world.0.bombs.iter().copied() {
        capture.world.0.kick_bombs.get_mut(&bomb.id).unwrap().bomb = bomb;
        capture.world.0.punch_bombs.get_mut(&bomb.id).unwrap().bomb = bomb;
    }
}
