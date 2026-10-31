use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::survival::plan_respawn;
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, WorkBudget, WorldId};
use crate::game::{
    ActorInit, ArenaGenerationConfig, Direction, EnabledItems, EntityId, PlayerCount, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig,
    Tick, World, generate_multiplayer_arena,
};
use crate::room::{BoardSize, RespawnLairConfig, RoomSettings};

use super::rules_v9_episode_tests::observe_for;

fn generated_state(seed: u8) -> State {
    let assignment = RoundAssignment::first();
    let generated = generate_multiplayer_arena(
        [seed; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: PlayerCount::Four,
            board: BoardSize::FifteenByThirteen,
            assignment,
        },
    )
    .unwrap();
    let spawns = assignment.spawns(generated.arena());
    let actors = SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(u32::try_from(seat.index() + 1).unwrap()),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let (world, _) = World::from_generated(generated).unwrap();
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v9(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::EMPTY),
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
        ),
        world,
        actors,
        Tick(0),
        assignment,
        SimulationSeed::new([seed.wrapping_add(1); 32]),
    )
    .unwrap()
}

#[test]
fn generated_all_off_four_player_openings_give_every_cpu_a_checked_action() {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::EMPTY);
    for seed in 0..=255 {
        let state = generated_state(seed);
        for seat in [SeatId::Two, SeatId::Three, SeatId::Four] {
            let observation = observe_for(&state, settings, seat);
            let key = MemoryKey {
                world: WorldId {
                    room_instance: 91,
                    physical_round: 1,
                },
                cpu_seat: seat,
            };
            let mut history = ObservedHazardMemory::new(key);
            history.observe(&observation).unwrap();
            let mut survival_work = WorkBudget::new(65_536);
            let plan = plan_respawn(
                &observation,
                &UtilityMemory::new(key),
                &history,
                observation.parts().tick,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut survival_work,
            )
            .unwrap();
            let intent = plan.survival.control.intent;
            assert!(
                intent.movement != MovementIntent::Stop || intent.manual.is_some(),
                "seed={seed} seat={seat:?} intent={intent:?} work={}",
                survival_work.spent()
            );
            assert!(
                plan.survival.lease.is_some() || plan.survival.execution.is_some(),
                "seed={seed} seat={seat:?} emitted an unchecked action"
            );
            assert!(survival_work.spent() <= 65_536);
        }
    }
}
