use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::routing::{
    ContinuationRequest, NominalLeg, PlantRequest, RouteLimits, plan_lair_ordinary_plant,
};
use crate::cpu::{MemoryKey, MovementIntent, WorkBudget};
use crate::game::{Direction, EntityId, InputAction, Position, SeatId, SeatInput, Tick};

use super::continuation_guard_tests::assert_guard_negatives;
use super::continuation_test_fixture::{CPU, observe, observe_for, state, step_until};
use super::{PlantEscapeOrigin, WORK_ALLOWANCE, certify_continuation};

#[test]
fn genuine_origin_certifies_and_executes_the_two_bomb_continuation() {
    let (mut state, settings) = state();
    state.step(&[]).unwrap();
    step_until(&mut state, Tick(184));
    let plant_basis = observe(&state, settings);
    let key = MemoryKey {
        world: plant_basis.parts().world,
        cpu_seat: CPU,
    };
    let rival_plant_basis = observe_for(&state, settings, SeatId::One);
    let rival_key = MemoryKey {
        world: rival_plant_basis.parts().world,
        cpu_seat: SeatId::One,
    };
    let plant = plan_lair_ordinary_plant(
        &plant_basis,
        &ObservedHazardMemory::new(key),
        PlantRequest {
            execution_tick: Tick(191),
            settled_stop_tick: Tick(184),
            first_movement: MovementIntent::Direction(Direction::South),
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .expect("ordinary short-fuse plant proof");
    let rival_plant = plan_lair_ordinary_plant(
        &rival_plant_basis,
        &ObservedHazardMemory::new(rival_key),
        PlantRequest {
            execution_tick: Tick(191),
            settled_stop_tick: Tick(184),
            first_movement: MovementIntent::Direction(Direction::South),
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .expect("mirrored ordinary short-fuse plant proof");
    assert_eq!(plant.placement().placed_at, Tick(191));
    assert_eq!(plant.placement().detonates_at, Tick(281));

    step_until(&mut state, Tick(190));
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: CPU,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    step_until(&mut state, Tick(196));
    let basis = observe(&state, settings);
    assert_eq!(
        state.actor(CPU).unwrap().position(),
        Position { x: 1536, y: 9728 }
    );
    assert_eq!(
        state.actor(SeatId::One).unwrap().position(),
        Position { x: 13824, y: 9728 }
    );
    assert!(
        state.world().bombs().values().all(|bomb| {
            bomb.placed_at() == Tick(191) && bomb.detonates_at() == Some(Tick(281))
        })
    );
    let own = state
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.owner() == EntityId(2))
        .unwrap();
    let rival_bomb = state
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.owner() == EntityId(1))
        .unwrap();
    assert_eq!(state.world().bombs().len(), 2);
    let origin = PlantEscapeOrigin::new(plant_basis, plant, own.id(), &basis)
        .expect("genuine stopped plant origin");
    let rival_basis = observe_for(&state, settings, SeatId::One);
    assert_eq!(rival_plant.placement().cell, rival_bomb.cell());
    assert!(super::compatibility::plant_actor_compatible(
        &rival_plant_basis,
        &rival_basis,
        &rival_plant,
        crate::cpu::routing::PlantPresence::Present(rival_bomb.id()),
    ));
    let rival_origin = PlantEscapeOrigin::new(
        rival_plant_basis,
        rival_plant,
        rival_bomb.id(),
        &rival_basis,
    )
    .expect("genuine mirrored stopped plant origin");
    let mut origin_work = WorkBudget::new(WORK_ALLOWANCE);
    super::continuation_planner::charge_origin_validation(&origin, &basis, &mut origin_work)
        .unwrap();
    let origin_units = origin_work.spent();
    assert!(origin_units > u32::try_from(basis.parts().actors.len()).unwrap());
    assert!(matches!(
        super::continuation_planner::charge_origin_validation(
            &origin,
            &basis,
            &mut WorkBudget::new(origin_units - 1),
        ),
        Err(crate::cpu::routing::RouteFailure::WorkExhausted)
    ));
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let legs = [
        NominalLeg::Move {
            direction: Direction::South,
            updates: 40,
        },
        NominalLeg::Stop { updates: 3 },
        NominalLeg::Move {
            direction: Direction::East,
            updates: 17,
        },
    ];
    let rival_legs = [
        NominalLeg::Move {
            direction: Direction::South,
            updates: 40,
        },
        NominalLeg::Stop { updates: 3 },
        NominalLeg::Move {
            direction: Direction::West,
            updates: 17,
        },
    ];
    let request = ContinuationRequest {
        first_update: Tick(203),
        settled_stop_tick: Tick(196),
        survive_until: Tick(312),
    };
    assert_guard_negatives(&basis, &hazards, request, &legs, &origin, key);
    let certificate = certify_continuation(&basis, &hazards, request, &legs, &origin, &mut work)
        .unwrap_or_else(|failure| {
            panic!(
                "continuation failed after {} work: {failure:?}",
                work.spent()
            )
        });
    let mut rival_work = WorkBudget::new(WORK_ALLOWANCE);
    let rival_hazards = forecast(
        &rival_basis,
        &ObservedHazardMemory::new(rival_key),
        None,
        ForecastLimits::default(),
        &mut rival_work,
    )
    .unwrap();
    let rival_certificate = certify_continuation(
        &rival_basis,
        &rival_hazards,
        request,
        &rival_legs,
        &rival_origin,
        &mut rival_work,
    )
    .unwrap_or_else(|failure| {
        panic!(
            "mirrored continuation failed after {} work: {failure:?}",
            rival_work.spent()
        )
    });
    assert!(rival_work.spent() <= WORK_ALLOWANCE);
    assert!(work.spent() <= WORK_ALLOWANCE);
    eprintln!("continuation work={}", work.spent());
    let exact_work = work.spent();
    let mut exact = WorkBudget::new(exact_work);
    let exact_forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut exact,
    )
    .unwrap();
    assert!(
        certify_continuation(&basis, &exact_forecast, request, &legs, &origin, &mut exact,).is_ok()
    );
    let mut one_short = WorkBudget::new(exact_work - 1);
    let short_forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut one_short,
    )
    .unwrap();
    assert!(matches!(
        certify_continuation(
            &basis,
            &short_forecast,
            request,
            &legs,
            &origin,
            &mut one_short,
        ),
        Err(crate::cpu::routing::RouteFailure::WorkExhausted)
    ));
    assert_eq!(certificate.first_update(), Tick(203));
    assert_eq!(certificate.survive_until(), Tick(312));
    let checkpoints = certificate.checkpoints_for_test();
    assert_eq!(checkpoints.len(), 60);
    assert_eq!(checkpoints[39].tick, Tick(242));
    assert_eq!(checkpoints[39].position, Position { x: 1536, y: 11776 });
    assert_eq!(checkpoints[0].residual, 12);
    assert_eq!(checkpoints[42].tick, Tick(245));
    assert_eq!(checkpoints[42].position, Position { x: 1536, y: 11776 });
    assert_eq!(checkpoints[42].residual, 0);
    assert_eq!(checkpoints[59].tick, Tick(262));
    assert_eq!(checkpoints[59].position, Position { x: 2406, y: 11776 });
    assert_eq!(checkpoints[59].residual, 24);
    assert!(
        checkpoints
            .iter()
            .all(|checkpoint| checkpoint.pending.is_none())
    );
    assert!(checkpoints.iter().all(|checkpoint| !checkpoint.reversed));
    assert_eq!(certificate.assumptions_for_test().len(), 2);
    let branches = certificate.stop_branch_shape_for_test();
    assert_eq!(branches.len(), 60);
    assert!(
        branches
            .iter()
            .all(|(publication, count, last)| { *count == 13 && last.0 == publication.0 + 13 })
    );
    assert_eq!(branches[41], (Tick(244), 13, Tick(257)));

    let unsafe_stop = [
        NominalLeg::Move {
            direction: Direction::South,
            updates: 40,
        },
        NominalLeg::Stop { updates: 21 },
        NominalLeg::Move {
            direction: Direction::East,
            updates: 17,
        },
    ];
    assert!(
        super::continuation_planner::certify_nominal_without_stop_branches(
            &basis,
            &hazards,
            request,
            &unsafe_stop,
            &origin,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .is_ok(),
        "the nominal late escape must remain safe so only its fallback branch rejects"
    );
    assert!(
        matches!(
            certify_continuation(
                &basis,
                &hazards,
                request,
                &unsafe_stop,
                &origin,
                &mut WorkBudget::new(WORK_ALLOWANCE),
            ),
            Err(crate::cpu::routing::RouteFailure::NoRoute)
        ),
        "continuous escape may clear the blast, but its unsafe early Stop branch must reject"
    );
    let reentry = [
        NominalLeg::Move {
            direction: Direction::South,
            updates: 40,
        },
        NominalLeg::Move {
            direction: Direction::North,
            updates: 40,
        },
    ];
    assert!(
        matches!(
            certify_continuation(
                &basis,
                &hazards,
                request,
                &reentry,
                &origin,
                &mut WorkBudget::new(WORK_ALLOWANCE),
            ),
            Err(crate::cpu::routing::RouteFailure::NoRoute)
        ),
        "leaving the planted cell permanently consumes collision grace"
    );

    let rival_checkpoints = rival_certificate.checkpoints_for_test();
    assert_eq!(rival_checkpoints.len(), checkpoints.len());
    for (checkpoint, rival_checkpoint) in checkpoints.into_iter().zip(rival_checkpoints) {
        step_until(&mut state, Tick(checkpoint.tick.0 - 1));
        let action = match checkpoint.movement {
            MovementIntent::Direction(direction) => InputAction::Press(direction),
            MovementIntent::Stop => InputAction::Clear,
        };
        let rival_action = match rival_checkpoint.movement {
            MovementIntent::Direction(direction) => InputAction::Press(direction),
            MovementIntent::Stop => InputAction::Clear,
        };
        state
            .step(&[
                SeatInput {
                    seat: SeatId::One,
                    action: rival_action,
                },
                SeatInput { seat: CPU, action },
            ])
            .unwrap();
        let actor = state.actor(CPU).unwrap();
        assert_eq!(actor.position(), checkpoint.position);
        assert_eq!(actor.heading(), checkpoint.heading);
        let rival = state.actor(SeatId::One).unwrap();
        assert_eq!(rival.position(), rival_checkpoint.position);
        assert_eq!(rival.heading(), rival_checkpoint.heading);
    }
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Clear,
            },
            SeatInput {
                seat: CPU,
                action: InputAction::Clear,
            },
        ])
        .unwrap();
    step_until(&mut state, Tick(312));
    assert!(state.actor(CPU).unwrap().alive());
    assert!(state.actor(SeatId::One).unwrap().alive());
    assert_eq!(
        state.actor(CPU).unwrap().position(),
        Position { x: 2406, y: 11776 }
    );
    assert_eq!(
        state.actor(SeatId::One).unwrap().position(),
        Position { x: 12954, y: 11776 }
    );
    assert!(state.world().bombs().is_empty());
    assert!(state.public_view().flames.is_empty());
}
