use crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    Observation, ObservedActor, ObservedEnvironment, ObservedRespawnLairs, ObservedSpikePits,
    WorkBudget,
};
use crate::game::{
    CellId, Direction, EntityId, MobView, Position, SpikePitPhase, StormCloudConfig, Tick,
};
use crate::room::{SpikePitConfig, SpikePitSettings};

use super::teleporter_execution_tests::{at, decision, key, plan, step};
use super::{
    CommittedTeleporterStep, PreparedTeleporterStep, TELEPORTER_ADMISSION_ALLOWANCE,
    TeleporterStepStatus,
};

#[test]
fn stopped_contact_accepts_expected_death_and_predicted_birth_but_rejects_displacement() {
    let basis = enabled_basis();
    let death = evolved_lairs(&basis, |lairs, mobs| {
        let removed = mobs.remove(0);
        let lair = &mut lairs[usize::from(removed.lair)];
        lair.living_mobs = 0;
        lair.next_spawn_at = Some(Tick(6_123 + 600));
    });
    assert_completed(committed(&basis), &death);

    let displaced = evolved_lairs(&basis, |_, mobs| {
        mobs[0].position.x += 1_000;
    });
    assert_invalidated(committed(&basis), &displaced);

    let birth_basis = modify_lairs(basis, |lairs, mobs| {
        let removed = mobs.remove(0);
        let lair = &mut lairs[usize::from(removed.lair)];
        lair.living_mobs = 0;
        lair.next_spawn_at = Some(Tick(6_122));
    });
    let born = evolved_lairs(&birth_basis, |lairs, mobs| {
        let lair = &mut lairs[0];
        let cell = lair.cell;
        lair.living_mobs = 1;
        lair.next_spawn_at = None;
        mobs.push(MobView {
            id: EntityId(999),
            lair: 0,
            position: Position::center(CellId::new(cell.column, cell.row - 1)),
            heading: Direction::North,
            frozen: false,
        });
    });
    assert_completed(committed(&birth_basis), &born);
}

#[test]
fn admission_accepts_distant_pit_phase_change_but_rejects_settings_or_layout_change() {
    let basis = enabled_basis();
    let warning = modify_pits(at(&basis, 1), |pits| {
        pits[0].phase = SpikePitPhase::Warning;
        pits[0].remaining_ticks = 36;
    });
    assert!(admit(&basis, &warning).is_ok());

    let changed_settings = rebuild(
        at(&basis, 1),
        basis.respawn_lairs().cloned(),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::new(6, 60).unwrap()),
            basis.spike_pits().unwrap().pits().to_vec(),
        )),
    );
    assert!(admit(&basis, &changed_settings).is_err());

    let missing_layout = rebuild(at(&basis, 1), basis.respawn_lairs().cloned(), None);
    assert!(admit(&basis, &missing_layout).is_err());
}

fn enabled_basis() -> Observation {
    let captured = captured_tick_6121();
    let environment = ObservedEnvironment::new(
        StormCloudConfig::Disabled,
        None,
        vec![Some(0); captured.parts().actors.len()],
    );
    let respawn = captured.respawn_lairs().cloned();
    let pits = captured.spike_pits().cloned();
    Observation::new_rules_v10(captured.into_parts(), environment, respawn, pits).unwrap()
}

fn modify_lairs(
    observation: Observation,
    change: impl FnOnce(&mut Vec<crate::game::LairStatusView>, &mut Vec<MobView>),
) -> Observation {
    let original = observation.respawn_lairs().unwrap();
    let mut lairs = original.lairs().to_vec();
    let mut mobs = original.mobs().to_vec();
    change(&mut lairs, &mut mobs);
    let respawn = ObservedRespawnLairs::new(original.config(), lairs, mobs);
    let pits = observation.spike_pits().cloned();
    rebuild(observation, Some(respawn), pits)
}

fn evolved_lairs(
    basis: &Observation,
    change: impl FnOnce(&mut Vec<crate::game::LairStatusView>, &mut Vec<MobView>),
) -> Observation {
    let original = basis.respawn_lairs().unwrap();
    let mut lairs = original.lairs().to_vec();
    let mut mobs = original.mobs().to_vec();
    change(&mut lairs, &mut mobs);
    let respawn = ObservedRespawnLairs::new(original.config(), lairs, mobs);
    let environment = basis.environment().unwrap().clone();
    let pits = basis.spike_pits().cloned();
    let mut parts = basis.clone().into_parts();
    parts.tick = Tick(parts.tick.0 + 2);
    parts.rules.running_updates += 2;
    parts.rules.remaining_ticks -= 2;
    let ObservedActor::Visible(root) = &mut parts.actors[2] else {
        unreachable!()
    };
    root.heading = Direction::West;
    Observation::new_rules_v10(parts, environment, Some(respawn), pits).unwrap()
}

fn modify_pits(
    observation: Observation,
    change: impl FnOnce(&mut Vec<crate::game::SpikePitView>),
) -> Observation {
    let original = observation.spike_pits().unwrap();
    let mut pits = original.pits().to_vec();
    change(&mut pits);
    let overlay = ObservedSpikePits::new(original.config(), pits);
    let respawn = observation.respawn_lairs().cloned();
    rebuild(observation, respawn, Some(overlay))
}

fn rebuild(
    observation: Observation,
    respawn: Option<ObservedRespawnLairs>,
    pits: Option<ObservedSpikePits>,
) -> Observation {
    let environment = observation.environment().unwrap().clone();
    Observation::new_rules_v10(observation.into_parts(), environment, respawn, pits).unwrap()
}

fn admit(
    basis: &Observation,
    current: &Observation,
) -> Result<PreparedTeleporterStep, crate::cpu::routing::RouteFailure> {
    PreparedTeleporterStep::admit(
        plan(step(basis, Tick(6_111)).unwrap()),
        basis.clone(),
        current,
        Tick(6_111),
        decision(basis),
        current.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    )
}

fn committed(basis: &Observation) -> CommittedTeleporterStep {
    let memory = UtilityMemory::new(key(basis));
    let mut prepared = PreparedTeleporterStep::admit(
        plan(step(basis, Tick(6_111)).unwrap()),
        basis.clone(),
        basis,
        Tick(6_111),
        decision(basis),
        basis.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    )
    .unwrap();
    let movement = prepared.command().unwrap();
    let mut committed = prepared.commit(movement).unwrap();
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), memory.key(), 7)
    else {
        unreachable!()
    };
    assert!(committed.record_stop(stop));
    committed
}

fn assert_completed(mut committed: CommittedTeleporterStep, current: &Observation) {
    assert!(matches!(
        committed.observe_stopped(
            current,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        ),
        TeleporterStepStatus::Completed(_)
    ));
}

fn assert_invalidated(mut committed: CommittedTeleporterStep, current: &Observation) {
    assert!(matches!(
        committed.observe_stopped(
            current,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        ),
        TeleporterStepStatus::Invalidated
    ));
}
