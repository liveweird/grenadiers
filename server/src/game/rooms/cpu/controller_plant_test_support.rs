use grenadiers_core::cpu::{Observation, ObservedActor};
use grenadiers_core::game::{SeatId, Tick};

pub(super) fn advanced(basis: &Observation, tick: Tick) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    for effect in &mut cpu.effects {
        effect.remaining_ticks = effect.remaining_ticks.saturating_sub(elapsed);
    }
    cpu.effects.retain(|effect| effect.remaining_ticks > 0);
    Observation::new(parts).unwrap()
}
