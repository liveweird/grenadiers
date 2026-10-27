use grenadiers_core::game::{CellId, EntityId, Tick};

use super::ObservedBombPlacement;

pub(super) fn cpu_event_ticks(frame: &str, cpu_id: EntityId) -> (Option<Tick>, Option<Tick>) {
    let Some(events) = event_values(frame) else {
        return (None, None);
    };
    let bomb = events.iter().find_map(|event| {
        (event["kind"]["type"] == "bombPlaced"
            && event["kind"]["bomb"]["owner"].as_u64() == Some(u64::from(cpu_id.0)))
        .then(|| event["tick"].as_u64().map(Tick))
        .flatten()
    });
    let pickup = events.iter().find_map(|event| {
        (event["kind"]["type"] == "pickupCollected" && event["kind"]["seat"] == "two")
            .then(|| event["tick"].as_u64().map(Tick))
            .flatten()
    });
    (bomb, pickup)
}

pub(super) fn bomb_placement(frame: &str, owner: EntityId) -> Option<ObservedBombPlacement> {
    event_values(frame)?.iter().find_map(|event| {
        if event["kind"]["type"] != "bombPlaced"
            || event["kind"]["bomb"]["owner"].as_u64() != Some(u64::from(owner.0))
        {
            return None;
        }
        let tick = Tick(event["tick"].as_u64()?);
        let column = u16::try_from(event["kind"]["bomb"]["cell"]["column"].as_u64()?).ok()?;
        let row = u16::try_from(event["kind"]["bomb"]["cell"]["row"].as_u64()?).ok()?;
        Some(ObservedBombPlacement {
            tick,
            cell: CellId::new(column, row),
        })
    })
}

fn event_values(frame: &str) -> Option<Vec<serde_json::Value>> {
    serde_json::from_str::<serde_json::Value>(frame).ok()?["events"]
        .as_array()
        .cloned()
}
