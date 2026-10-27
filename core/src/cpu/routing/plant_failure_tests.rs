use crate::cpu::forecast::{ForecastFailure, ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedEffect, UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, Direction, EffectKind, EntityId, SpecialWeapon, Tick};

use super::plant_tests::request;
use super::tests::{key, observation};
use super::{PlantFailure, RouteFailure, plan_ordinary_plant};

#[test]
fn invalid_schedule_capacity_weapon_and_work_fail_closed() {
    let observed = observation(Vec::new(), Vec::new());
    let memory = ObservedHazardMemory::new(key());
    let mut invalid = request(MovementIntent::Stop);
    invalid.execution_tick = Tick(6);
    assert_eq!(
        plan_ordinary_plant(&observed, &memory, invalid, &mut WorkBudget::new(65_536)),
        Err(PlantFailure::InvalidRequest)
    );
    assert!(matches!(
        plan_ordinary_plant(
            &observed,
            &memory,
            request(MovementIntent::Stop),
            &mut WorkBudget::new(1)
        ),
        Err(PlantFailure::Forecast(_))
    ));

    let mut parts = observed.into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.weapon = Some(SpecialWeapon::Piercing);
    let armed = Observation::new(parts).unwrap();
    assert_eq!(
        plan_ordinary_plant(
            &armed,
            &memory,
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536)
        ),
        Err(PlantFailure::Unsupported)
    );

    let mut full = armed.into_parts();
    let ObservedActor::Visible(actor) = &mut full.actors[1] else {
        unreachable!()
    };
    actor.weapon = None;
    actor.live_bomb_count = actor.bomb_capacity;
    let full = Observation::new(full).unwrap();
    assert_eq!(
        plan_ordinary_plant(
            &full,
            &memory,
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536)
        ),
        Err(PlantFailure::Unsupported)
    );
}

#[test]
fn visible_live_own_remote_declines_ordinary_placement_without_forecasting() {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.bomb_capacity = 2;
    actor.live_bomb_count = 1;
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(91),
        placed_at: Tick(0),
        cell: crate::game::CellId::new(5, 1),
        owner: actor.id,
        range: 3,
        trigger: ObservedBombTrigger::Remote,
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    let observed = Observation::new(parts).unwrap();
    let mut work = WorkBudget::new(65_536);

    assert_eq!(
        plan_ordinary_plant(
            &observed,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Direction(Direction::East)),
            &mut work,
        ),
        Err(PlantFailure::Unsupported)
    );
    assert_eq!(work.spent(), 0);
}

#[test]
fn absent_shielding_danger_and_partial_work_never_return_a_certificate() {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.arena.blocks.push(crate::game::CellId::new(4, 2));
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = crate::game::Position::center(crate::game::CellId::new(4, 1));
    actor.heading = Direction::West;
    actor.effects = vec![ObservedEffect {
        kind: EffectKind::ShortFlame,
        remaining_ticks: 100,
    }];
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: Tick(0),
        cell: crate::game::CellId::new(7, 1),
        owner: EntityId(1),
        range: 6,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    let shielding = Observation::new(parts).unwrap();
    assert!(
        plan_ordinary_plant(
            &shielding,
            &ObservedHazardMemory::new(key()),
            request(MovementIntent::Direction(Direction::West)),
            &mut WorkBudget::new(65_536),
        )
        .is_err()
    );

    let clear = observation(Vec::new(), Vec::new());
    let memory = ObservedHazardMemory::new(key());
    let mut first = WorkBudget::new(65_536);
    forecast(&clear, &memory, None, ForecastLimits::default(), &mut first).unwrap();
    let mut during_second = WorkBudget::new(first.spent() + 1);
    assert_eq!(
        plan_ordinary_plant(
            &clear,
            &memory,
            request(MovementIntent::Direction(Direction::East)),
            &mut during_second,
        ),
        Err(PlantFailure::Forecast(ForecastFailure::WorkExhausted))
    );

    let mut complete = WorkBudget::new(65_536);
    plan_ordinary_plant(
        &clear,
        &memory,
        request(MovementIntent::Direction(Direction::East)),
        &mut complete,
    )
    .unwrap();
    let mut during_finish = WorkBudget::new(complete.spent() - 1);
    assert_eq!(
        plan_ordinary_plant(
            &clear,
            &memory,
            request(MovementIntent::Direction(Direction::East)),
            &mut during_finish,
        ),
        Err(PlantFailure::Route(RouteFailure::WorkExhausted))
    );
}
