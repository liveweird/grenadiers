use grenadiers_core::game::{EntityId, Tick};
use serde_json::json;

use super::ObservedBombPlacement;
use super::events::{bomb_placement, cpu_event_ticks};

#[test]
fn physical_metrics_ignore_vetoed_inputs_and_other_actors_events() {
    let no_physical_event = json!({ "events": [] }).to_string();
    assert_eq!(
        cpu_event_ticks(&no_physical_event, EntityId(20)),
        (None, None)
    );

    let other_actor_events = json!({ "events": [
        { "tick": 4, "kind": { "type": "bombPlaced", "bomb": { "owner": 10 } } },
        { "tick": 5, "kind": { "type": "pickupCollected", "seat": "one" } }
    ] })
    .to_string();
    assert_eq!(
        cpu_event_ticks(&other_actor_events, EntityId(20)),
        (None, None)
    );
}

#[test]
fn physical_metrics_use_cpu_recipient_placement_and_pickup_events() {
    let cpu_events = json!({ "events": [
        { "tick": 7, "kind": { "type": "bombPlaced", "bomb": { "owner": 20 } } },
        { "tick": 9, "kind": { "type": "pickupCollected", "seat": "two" } }
    ] })
    .to_string();
    assert_eq!(
        cpu_event_ticks(&cpu_events, EntityId(20)),
        (Some(Tick(7)), Some(Tick(9)))
    );
}

#[test]
fn human_placement_records_physical_tick_and_cell() {
    let frame = json!({ "events": [
        { "tick": 22, "kind": { "type": "bombPlaced", "bomb": {
            "owner": 10, "cell": { "column": 2, "row": 1 }
        } } }
    ] })
    .to_string();
    assert_eq!(
        bomb_placement(&frame, EntityId(10)),
        Some(ObservedBombPlacement {
            tick: Tick(22),
            cell: grenadiers_core::game::CellId::new(2, 1),
        })
    );
}
