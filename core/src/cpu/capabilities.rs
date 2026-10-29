use crate::cpu::{Observation, ObservedActor};
use crate::game::Equipment;

#[must_use]
pub(crate) fn can_pass_known_soft_blocks(observation: &Observation) -> bool {
    let parts = observation.parts();
    matches!(parts.rules.version, 5..=8)
        && parts.rules.settings.enabled_items().contains_wall_pass()
        && parts.actors.iter().any(|actor| {
            matches!(actor, ObservedActor::Visible(actor)
                if actor.seat == parts.recipient
                    && actor.alive
                    && actor.equipment == Some(Equipment::WallPass))
        })
}

#[cfg(test)]
mod tests {
    use crate::cpu::ObservedActor;
    use crate::cpu::policy::tests::observation;
    use crate::game::{CellId, EnabledItems, Equipment};

    use super::can_pass_known_soft_blocks;

    #[test]
    fn capability_requires_v5_enabled_living_visible_holder() {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        parts.rules.settings = parts
            .rules
            .settings
            .with_enabled_items(EnabledItems::WALL_PASS);
        let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
            unreachable!()
        };
        actor.equipment = Some(Equipment::WallPass);
        let capable = crate::cpu::Observation::new(parts).unwrap();
        assert!(can_pass_known_soft_blocks(&capable));

        let mut disabled = capable.clone().into_parts();
        disabled.rules.settings = disabled
            .rules
            .settings
            .with_enabled_items(EnabledItems::EMPTY);
        assert!(!can_pass_known_soft_blocks(
            &crate::cpu::Observation::new(disabled).unwrap()
        ));
        let mut dead = capable.into_parts();
        let ObservedActor::Visible(actor) = &mut dead.actors[1] else {
            unreachable!()
        };
        actor.alive = false;
        assert!(!can_pass_known_soft_blocks(
            &crate::cpu::Observation::new(dead).unwrap()
        ));
    }
}
