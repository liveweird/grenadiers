use crate::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedEffect, ObservedEnvironment, ObservedOil, ObservedRespawnLairs, ObservedSpikePits,
    UnknownBombFacts, WorkBudget,
};
use crate::game::{
    BlastKind, CellId, EffectKind, EntityId, Equipment, LairStatusView, MobView, OilBarrelState,
    Position, SeatId, SpikePitPhase, SpikePitView, StormCloudConfig, Tick, lair_layout_v1,
};
use crate::room::{
    BoardSize, RespawnLairConfig, RespawnLairSettings, SpikePitConfig, SpikePitSettings,
};

use super::can_pause_easy_v68;

fn baseline() -> Observation {
    let original = super::policy::tests::observation(CellId::new(2, 2));
    let mut parts = original.into_parts();
    parts.rules.version = 10;
    parts.rules.punch_activation_policy = crate::game::PunchActivationPolicy::ContactV2;
    let ObservedActor::Visible(mut rival) = parts.actors[1].clone() else {
        unreachable!()
    };
    rival.id = EntityId(1);
    rival.seat = SeatId::One;
    rival.position = Position::center(CellId::new(12, 10));
    parts.actors[0] = ObservedActor::Visible(rival);
    construct(parts, None, None)
}

fn construct(
    parts: ObservationParts,
    lairs: Option<ObservedRespawnLairs>,
    pits: Option<ObservedSpikePits>,
) -> Observation {
    let timers = parts
        .actors
        .iter()
        .map(|actor| matches!(actor, ObservedActor::Visible(_)).then_some(0))
        .collect();
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, timers),
        lairs,
        pits,
    )
    .unwrap()
}

fn construct_with_environment(
    parts: ObservationParts,
    environment: ObservedEnvironment,
) -> Observation {
    Observation::new_rules_v10(parts, environment, None, None).unwrap()
}

fn recipient(parts: &mut ObservationParts) -> &mut crate::cpu::VisibleActor {
    parts
        .actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
            _ => None,
        })
        .unwrap()
}

fn pits(phase: SpikePitPhase, remaining_ticks: u64) -> ObservedSpikePits {
    let mut cells = [
        CellId::new(5, 5),
        CellId::new(9, 5),
        CellId::new(5, 7),
        CellId::new(9, 7),
    ];
    cells.sort_unstable();
    ObservedSpikePits::new(
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        cells
            .into_iter()
            .map(|cell| SpikePitView {
                cell,
                phase,
                remaining_ticks,
            })
            .collect(),
    )
}

fn with_lairs(
    source: Observation,
    cpu: Position,
    next_spawn_at: Option<Tick>,
    mobs: Vec<MobView>,
) -> Observation {
    let mut parts = source.into_parts();
    recipient(&mut parts).position = cpu;
    let cells = lair_layout_v1(BoardSize::FifteenByThirteen);
    for cell in cells {
        let index = parts.arena.index(cell).unwrap();
        parts.arena.terrain[index] = crate::game::Terrain::Wall;
        parts.arena.blocks.retain(|block| *block != cell);
    }
    let counts = cells.map(|cell| {
        u8::try_from(
            mobs.iter()
                .filter(|mob| cells[usize::from(mob.lair)] == cell)
                .count(),
        )
        .unwrap()
    });
    let lairs = ObservedRespawnLairs::new(
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
        cells
            .into_iter()
            .zip(counts)
            .map(|(cell, living_mobs)| LairStatusView {
                cell,
                next_spawn_at: (living_mobs == 0).then_some(next_spawn_at).flatten(),
                living_mobs,
            })
            .collect(),
        mobs,
    );
    construct(parts, Some(lairs), None)
}

fn allows(observation: &Observation, through: Tick, limit: u32) -> (bool, u32) {
    let mut work = WorkBudget::new(limit);
    let allowed = can_pause_easy_v68(observation, through, &mut work);
    (allowed, work.spent())
}

#[test]
fn clear_scene_is_bounded_and_charges_exact_required_work() {
    let observed = baseline();
    let through = Tick(observed.parts().tick.0 + 12);
    let (allowed, required) = allows(&observed, through, 65_536);
    assert!(allowed);
    assert!(required > 0);
    assert_eq!(allows(&observed, through, required), (true, required));
    assert!(!allows(&observed, through, required - 1).0);
    assert!(!allows(&observed, Tick(through.0 + 1), 65_536).0);
}

#[test]
fn active_cloud_position_and_slow_timer_expiry_do_not_move_a_stationary_actor() {
    let observed = baseline();
    let through = Tick(observed.parts().tick.0 + 12);
    for (cloud_cell, recipient_timer, rival_timer) in
        [(CellId::new(12, 10), 0, 0), (CellId::new(2, 2), 1, 40)]
    {
        let environment = ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(crate::game::StormCloudView {
                position: Position::center(cloud_cell),
                direction: crate::game::Direction::East,
                leg_remaining_ticks: 1,
            }),
            vec![Some(rival_timer), Some(recipient_timer)],
        );
        let cloudy = construct_with_environment(observed.clone().into_parts(), environment);
        assert!(allows(&cloudy, through, 65_536).0);
    }
}

#[test]
fn distant_pits_are_supported_but_recipient_footprint_contact_is_not() {
    let observed = baseline();
    let parts = observed.into_parts();
    let distant = construct(parts.clone(), None, Some(pits(SpikePitPhase::Idle, 0)));
    assert!(allows(&distant, Tick(distant.parts().tick.0 + 12), 65_536).0);

    let mut contacted_parts = parts;
    recipient(&mut contacted_parts).position = Position::center(CellId::new(5, 5));
    let contacted = construct(contacted_parts, None, Some(pits(SpikePitPhase::Raised, 1)));
    assert!(!allows(&contacted, Tick(contacted.parts().tick.0 + 12), 65_536).0);
}

#[test]
fn bombs_fire_forced_control_and_uncertain_or_displacing_rivals_fail_closed() {
    let observed = baseline();
    let through = Tick(observed.parts().tick.0 + 12);

    let mut unknown = observed.clone().into_parts();
    unknown.bombs = BombKnowledge::Unavailable;
    assert!(!allows(&construct(unknown, None, None), through, 65_536).0);

    let mut bomb = observed.clone().into_parts();
    let owner = recipient(&mut bomb);
    owner.live_bomb_count = 1;
    bomb.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: Tick(9),
        cell: CellId::new(4, 4),
        owner: owner.id,
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 20,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    assert!(!allows(&construct(bomb, None, None), through, 65_536).0);

    for kind in [EffectKind::Diarrhea, EffectKind::ReverseControls] {
        let mut affected = observed.clone().into_parts();
        recipient(&mut affected).effects.push(ObservedEffect {
            kind,
            remaining_ticks: 10,
        });
        assert!(!allows(&construct(affected, None, None), through, 65_536).0);
    }
    let mut frozen = observed.clone().into_parts();
    recipient(&mut frozen).voluntary_actions_frozen = true;
    assert!(!allows(&construct(frozen, None, None), through, 65_536).0);

    let mut hidden = observed.clone().into_parts();
    hidden.actors[0] = ObservedActor::Hidden {
        id: EntityId(1),
        seat: SeatId::One,
        alive: true,
    };
    assert!(!allows(&construct(hidden, None, None), through, 65_536).0);

    let mut shove = observed.clone().into_parts();
    let ObservedActor::Visible(rival) = &mut shove.actors[0] else {
        unreachable!()
    };
    rival.equipment = Some(Equipment::Bracelet);
    assert!(!allows(&construct(shove, None, None), through, 65_536).0);

    let mut fire = observed.into_parts();
    fire.oil.push(ObservedOil {
        cell: CellId::new(4, 4),
        state: OilBarrelState::Burning,
    });
    assert!(!allows(&construct(fire, None, None), through, 65_536).0);
}

#[test]
fn mob_approach_and_lair_birth_inside_the_hold_reject_the_pause() {
    let observed = baseline();
    let through = Tick(observed.parts().tick.0 + 12);
    let near_lair = Position::center(CellId::new(6, 7));
    let birth = with_lairs(observed.clone(), near_lair, Some(Tick(15)), Vec::new());
    assert!(!allows(&birth, through, 65_536).0);

    let cpu = Position::center(CellId::new(2, 2));
    let mob = MobView {
        id: EntityId(70),
        lair: 0,
        position: Position {
            x: cpu.x + 900,
            y: cpu.y,
        },
        heading: crate::game::Direction::West,
        frozen: false,
    };
    let approach = with_lairs(observed.clone(), cpu, Some(Tick(600)), vec![mob]);
    assert!(!allows(&approach, through, 65_536).0);

    let distant = with_lairs(observed, cpu, Some(Tick(600)), Vec::new());
    assert!(allows(&distant, through, 65_536).0);
}
